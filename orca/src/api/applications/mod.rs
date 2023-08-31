use rocket::{Route, State};

use super::Response;
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak};

#[get("/test", format = "json")]
async fn test_api<'r>(
    _db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<String> {
    let res = keycloak.inner().decode_jwt(token);

    Ok(format!("{:?}", res))
}

pub fn routes() -> Vec<Route> {
    routes![test_api]
}
