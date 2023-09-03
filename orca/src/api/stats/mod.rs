use rocket::serde::json::Json;
use rocket::{serde::Serialize, Route, State};

use super::Response;
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak};

mod query;

#[derive(Debug, Serialize)]
struct BasicStats {
    unverified: i64,
    accepted: i64,
    rejected: i64,
    processing: i64,
}

#[get("/basic")]
async fn basic_stats<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<BasicStats>> {
    // Every authenticated user is able to see stats
    keycloak.inner().decode_jwt(token)?;

    let (unverified,) = query::count_unverified().fetch_one(db_pool.inner()).await?;

    let (accepted,) = query::count_accepted().fetch_one(db_pool.inner()).await?;

    let (rejected,) = query::count_rejected().fetch_one(db_pool.inner()).await?;

    let (processing,) = query::count_processing().fetch_one(db_pool.inner()).await?;

    Ok(Json(BasicStats {
        unverified,
        accepted,
        rejected,
        processing,
    }))
}

pub fn routes() -> Vec<Route> {
    routes![basic_stats]
}
