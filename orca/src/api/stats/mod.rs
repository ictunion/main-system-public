use rocket::serde::json::Json;
use rocket::{Route, State, get, routes, serde::Serialize};

mod query;

use super::Response;
use crate::db::DbPool;
use crate::server::oid::{JwtToken, Provider};

#[derive(Debug, Serialize)]
struct ApplicationsBasicStats {
    unverified: i64,
    accepted: i64,
    rejected: i64,
    processing: i64,
    invalid: i64,
}

#[get("/applications/basic")]
async fn applications_basic_stats(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<ApplicationsBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.inner().decode_jwt(&token)?;

    let unverified = query::count_unverified_applications(db_pool.inner()).await?;
    let accepted = query::count_accepted_applications(db_pool.inner()).await?;
    let rejected = query::count_rejected_applications(db_pool.inner()).await?;
    let processing = query::count_processing_applications(db_pool.inner()).await?;
    let invalid = query::count_invalid_applications(db_pool.inner()).await?;

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
    /// Members with `left_at IS NULL` connected to at least one workplace.
    workplace: i64,
    /// Members with `left_at IS NULL` connected to no workplace.
    sectoral: i64,
}

#[get("/members/basic")]
async fn members_basic_stats(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<MembersBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.inner().decode_jwt(&token)?;

    let new = query::count_new_members(db_pool.inner()).await?;
    let current = query::count_current_members(db_pool.inner()).await?;
    let past = query::count_past_members(db_pool.inner()).await?;
    let workplace = query::count_workplace_members(db_pool.inner()).await?;
    let sectoral = query::count_sectoral_members(db_pool.inner()).await?;

    Ok(Json(MembersBasicStats {
        new,
        current,
        past,
        workplace,
        sectoral,
    }))
}

#[derive(Debug, Serialize)]
struct WorkplacesBasicStats {
    current: i64,
    past: i64,
}

#[get("/workplaces/basic")]
async fn workplaces_basic_stats(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<WorkplacesBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.inner().decode_jwt(&token)?;

    let current = query::count_current_workplaces(db_pool.inner()).await?;
    let past = query::count_past_workplaces(db_pool.inner()).await?;

    Ok(Json(WorkplacesBasicStats { current, past }))
}

#[expect(clippy::redundant_type_annotations, reason = "rocket macro expansion")]
pub fn routes() -> Vec<Route> {
    routes![
        applications_basic_stats,
        members_basic_stats,
        workplaces_basic_stats
    ]
}
