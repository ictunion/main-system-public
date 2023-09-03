use rocket::{Route, State};

use super::Response;
use crate::db::DbPool;
use crate::server::keycloak::{JwtClaims, JwtToken, Keycloak};
use jsonwebtoken::TokenData;
use rocket::serde::{json::Json, Serialize};

#[derive(Debug, Serialize)]
struct SessionInfo {
    token_claims: JwtClaims,
}

impl SessionInfo {
    fn new(token: TokenData<JwtClaims>) -> Self {
        Self {
            token_claims: token.claims,
        }
    }
}

#[get("/current", format = "json")]
async fn current<'r>(
    _db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<SessionInfo>> {
    let token_data = keycloak.inner().decode_jwt(token)?;

    let session_info = SessionInfo::new(token_data);

    Ok(Json(session_info))
}

pub fn routes() -> Vec<Route> {
    routes![current]
}
