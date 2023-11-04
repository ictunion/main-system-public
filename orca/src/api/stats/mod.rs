use rocket::serde::json::Json;
use rocket::{serde::Serialize, Route, State};

use super::Response;
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak};

mod query;

#[derive(Debug, Serialize)]
struct ApplicationsBasicStats {
    unverified: i64,
    accepted: i64,
    rejected: i64,
    processing: i64,
    invalid: i64,
}

#[get("/applications/basic")]
async fn applications_basic_stats<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<ApplicationsBasicStats>> {
    // Every authenticated user is able to see stats
    keycloak.inner().decode_jwt(token)?;

    let (unverified,) = query::count_unverified_applications()
        .fetch_one(db_pool.inner())
        .await?;

    let (accepted,) = query::count_accepted_applications()
        .fetch_one(db_pool.inner())
        .await?;

    let (rejected,) = query::count_rejected_applications()
        .fetch_one(db_pool.inner())
        .await?;

    let (processing,) = query::count_processing_applications()
        .fetch_one(db_pool.inner())
        .await?;

    let (invalid,) = query::count_invalid_applications()
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(ApplicationsBasicStats {
        unverified,
        accepted,
        rejected,
        processing,
        invalid,
    }))
}

#[derive(Debug, Serialize)]
struct MembersBasicStats {
    new: i64,
    current: i64,
    past: i64,
}

#[get("/members/basic")]
async fn members_basic_stats<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<MembersBasicStats>> {
    // Every authenticated user is able to see stats
    keycloak.inner().decode_jwt(token)?;

    let (new,) = query::count_new_members()
        .fetch_one(db_pool.inner())
        .await?;

    let (current,) = query::count_current_members()
        .fetch_one(db_pool.inner())
        .await?;

    let (past,) = query::count_past_members()
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(MembersBasicStats { new, current, past }))
}

pub fn routes() -> Vec<Route> {
    routes![applications_basic_stats, members_basic_stats]
}
