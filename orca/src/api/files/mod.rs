use axum::Router;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::http::request::Parts;
use axum::response::IntoResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::extract::ApiPath;
use crate::api::{ApiError, AppState, Response};
use crate::data::{FileType, Id};
use crate::db::DbPool;
use crate::server::oid::{JwtToken, Provider, Role};

#[derive(Debug, sqlx::FromRow)]
pub struct File {
    // This is read only and using Rc<[u8]> rather than Vec<u8> removes some overhead
    data: Vec<u8>,
    name: String,
    kind: FileType,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FileInfo {
    pub(crate) id: Id<File>,
    pub(crate) name: String,
    pub(crate) file_type: FileType,
    pub(crate) created_at: DateTime<Utc>,
}

impl File {
    /// `inline` keeps opening the file in the browser,
    /// `filename` is what the browser offers when saving it: name and creation date (UTC).
    fn content_disposition(&self) -> String {
        format!(
            "inline; filename=\"{}_{}.{}\"",
            header_safe_name(&self.name),
            self.created_at.format("%Y-%m-%d"),
            self.kind.extension()
        )
    }
}

/// File names are our own identifiers (`entrustment`, `registration`...), but anything
/// outside a plain ASCII set is replaced so it can't break the header value.
fn header_safe_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

impl IntoResponse for File {
    fn into_response(self) -> axum::response::Response {
        let content_disposition = self.content_disposition();
        (
            [
                (CONTENT_TYPE, self.kind.mime_type().to_string()),
                (CONTENT_DISPOSITION, content_disposition),
            ],
            self.data,
        )
            .into_response()
    }
}

#[derive(Debug, Deserialize)]
struct QueryWithToken {
    token: Option<UrlToken>,
}

/// JWT token from the `?token=` query parameter, because files are opened
/// as plain links which can't send an `Authorization` header.
/// A missing token is a 401.
#[derive(Debug, Deserialize)]
#[serde(from = "String")]
struct UrlToken(JwtToken);

impl From<String> for UrlToken {
    fn from(token: String) -> Self {
        UrlToken(JwtToken::new(token))
    }
}

impl<S: Send + Sync> FromRequestParts<S> for UrlToken {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(query) = Query::<QueryWithToken>::from_request_parts(parts, state)
            .await
            .map_err(|_rejection| ApiError::Status(StatusCode::UNAUTHORIZED))?;

        query
            .token
            .ok_or(ApiError::Status(StatusCode::UNAUTHORIZED))
    }
}

async fn get(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    UrlToken(token): UrlToken,
    ApiPath(id): ApiPath<Id<File>>,
) -> Response<File> {
    // Files belong to applications, members or workplaces. Any role that can
    // list one of those file lists may download; the file owner isn't checked.
    oid_provider.require_any_role(
        &token,
        &[
            Role::ViewApplication,
            Role::ListMembers,
            Role::ListWorkplaces,
        ],
    )?;

    let file: File = read_file(&db_pool, id).await?;
    Ok(file)
}

async fn read_file(pool: &DbPool, id: Id<File>) -> sqlx::Result<File> {
    sqlx::query_as!(
        File,
        r#"
SELECT data AS "data!"
    , name
    , file_type AS "kind: FileType"
    , created_at
FROM files
WHERE id = $1
"#,
        id as _
    )
    .fetch_one(pool)
    .await
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/{id}", axum::routing::get(get))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_disposition_keeps_file_name() {
        let file = File {
            data: Vec::new(),
            name: "acme-cz_workplace_announcement".to_string(),
            kind: FileType::Pdf,
            created_at: DateTime::from_timestamp(1_791_590_400, 0).unwrap(),
        };
        assert_eq!(
            file.content_disposition(),
            "inline; filename=\"acme-cz_workplace_announcement_2026-10-10.pdf\""
        );
    }

    #[test]
    fn header_safe_name_replaces_unsafe_chars() {
        assert_eq!(header_safe_name("přihláška \"1\""), "p_ihl__ka__1_");
    }
}
