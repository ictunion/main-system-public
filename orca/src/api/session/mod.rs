use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;

use super::{ApiError, Response};
use crate::api::AppState;
use crate::data::{Id, Member};
use crate::db::DbPool;
use crate::server::oid::{JwtClaims, JwtToken, Provider};

#[derive(Debug, Serialize)]
struct SessionInfo {
    token_claims: JwtClaims,
    member_id: Option<Id<Member>>,
}

async fn current(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<SessionInfo>> {
    let token_data = oid_provider.decode_jwt(&token)?;

    let member_id = get_user_id(&db_pool, &token_data.claims).await?;

    let session_info = SessionInfo {
        token_claims: token_data.claims,
        member_id,
    };

    Ok(Json(session_info))
}

async fn pair_by_email(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<SessionInfo>> {
    // TODO: should we require some role for this action?
    // in a way we're already trusting token so maybe we can also just
    // let any member assing themselves.
    let token_data = oid_provider.decode_jwt(&token)?;

    let member_id = get_user_id(&db_pool, &token_data.claims).await?;

    if member_id.is_none() {
        let member_id = set_pairing_by_email(&db_pool, &token_data.claims).await?;

        let session_info = SessionInfo {
            token_claims: token_data.claims,
            member_id: Some(member_id),
        };

        Ok(Json(session_info))
    } else {
        Err(ApiError::data_conflict("Member id is already assigned"))
    }
}

async fn get_user_id(pool: &DbPool, claims: &JwtClaims) -> sqlx::Result<Option<Id<Member>>> {
    sqlx::query_scalar!("SELECT id FROM members WHERE sub = $1", claims.sub)
        .fetch_optional(pool)
        .await
        .map(|opt| opt.map(Id::from))
}

async fn set_pairing_by_email(pool: &DbPool, claims: &JwtClaims) -> sqlx::Result<Id<Member>> {
    sqlx::query_scalar!(
        "UPDATE members SET sub = $1, onboarding_finished_at = NOW() WHERE email = $2 AND left_at IS NULL RETURNING id",
        claims.sub,
        claims.email,
    )
    .fetch_one(pool)
    .await
    .map(Id::from)
}
pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/current", get(current))
        .route("/current/pair-by-email", post(pair_by_email))
}
