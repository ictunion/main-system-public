use rocket::{Route, State};

use super::{ApiError, Response};
use crate::data::{Id, Member};
use crate::db::{DbPool, QueryAs};
use crate::server::keycloak::{JwtClaims, JwtToken, Keycloak};
use rocket::serde::{json::Json, Serialize};

#[derive(Debug, Serialize)]
struct SessionInfo {
    token_claims: JwtClaims,
    member_id: Option<Id<Member>>,
}

#[get("/current", format = "json")]
async fn current<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<SessionInfo>> {
    let token_data = keycloak.inner().decode_jwt(token)?;

    let member_id = get_user_id(&token_data.claims)
        .fetch_optional(db_pool.inner())
        .await?
        .map(|(member_id,)| member_id);

    let session_info = SessionInfo {
        token_claims: token_data.claims,
        member_id,
    };

    Ok(Json(session_info))
}

#[post("/current/pair-by-email", format = "json")]
async fn pair_by_email<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<SessionInfo>> {
    // TODO: shouw we require some role for this action?
    // in a way we're already trusting token so maybe we can also just
    // let any member assing themselves.
    let token_data = keycloak.inner().decode_jwt(token)?;

    let member_id = get_user_id(&token_data.claims)
        .fetch_optional(db_pool.inner())
        .await?
        .map(|(member_id,)| member_id);

    if member_id.is_none() {
        let (member_id,) = set_pairing_by_email(&token_data.claims)
            .fetch_one(db_pool.inner())
            .await?;

        let session_info = SessionInfo {
            token_claims: token_data.claims,
            member_id: Some(member_id),
        };

        Ok(Json(session_info))
    } else {
        Err(ApiError::data_conflict(
            "Member id is already assigned".to_string(),
        ))
    }
}

fn get_user_id(claims: &JwtClaims) -> QueryAs<'_, (Id<Member>,)> {
    sqlx::query_as(
        "
SELECT id
FROM members
WHERE sub = $1
",
    )
    .bind(claims.sub)
}

fn set_pairing_by_email(claims: &JwtClaims) -> QueryAs<'_, (Id<Member>,)> {
    sqlx::query_as(
        "
UPDATE members
SET   sub = $1
    , onboarding_finished_at = NOW()
WHERE email = $2
    AND left_at IS NULL
RETURNING id
",
    )
    .bind(claims.sub)
    .bind(&claims.email)
}

pub fn routes() -> Vec<Route> {
    routes![current, pair_by_email]
}
