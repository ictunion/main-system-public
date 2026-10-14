use axum::Router;
use axum::extract::{FromRequestParts, Query, State};
use axum::http::StatusCode;
use axum::http::header::CONTENT_TYPE;
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
    file_type: FileType,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FileInfo {
    pub(crate) id: Id<File>,
    pub(crate) name: String,
    pub(crate) file_type: FileType,
    pub(crate) created_at: DateTime<Utc>,
}

impl IntoResponse for File {
    fn into_response(self) -> axum::response::Response {
        ([(CONTENT_TYPE, self.file_type.mime_type())], self.data).into_response()
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
    oid_provider.require_role(&token, Role::ViewApplication)?;

    let file: File = read_file(&db_pool, id).await?;
    Ok(file)
}

async fn read_file(pool: &DbPool, id: Id<File>) -> sqlx::Result<File> {
    sqlx::query_as!(
        File,
        r#"SELECT data AS "data!", file_type AS "file_type: FileType" FROM files WHERE id = $1"#,
        id as _
    )
    .fetch_one(pool)
    .await
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/{id}", axum::routing::get(get))
}
