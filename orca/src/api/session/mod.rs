use rocket::serde::{Serialize, json::Json};
use rocket::{Route, State, get, post, routes};

use super::{ApiError, Response};
use crate::data::{Id, Member};
use crate::db::DbPool;
use crate::server::oid::{JwtClaims, JwtToken, Provider};

#[derive(Debug, Serialize)]
struct SessionInfo {
    token_claims: JwtClaims,
    member_id: Option<Id<Member>>,
}

#[get("/current", format = "json")]
async fn current(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<SessionInfo>> {
    let token_data = oid_provider.inner().decode_jwt(&token)?;

    let member_id = get_user_id(db_pool.inner(), &token_data.claims).await?;

    let session_info = SessionInfo {
        token_claims: token_data.claims,
        member_id,
    };

    Ok(Json(session_info))
}

#[post("/current/pair-by-email", format = "json")]
async fn pair_by_email(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<SessionInfo>> {
    // TODO: should we require some role for this action?
    // in a way we're already trusting token so maybe we can also just
    // let any member assing themselves.
    let token_data = oid_provider.inner().decode_jwt(&token)?;

    let member_id = get_user_id(db_pool.inner(), &token_data.claims).await?;

    if member_id.is_none() {
        let member_id = set_pairing_by_email(db_pool.inner(), &token_data.claims).await?;

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
#[expect(clippy::redundant_type_annotations, reason = "rocket macro expansion")]
pub fn routes() -> Vec<Route> {
    routes![current, pair_by_email]
}
