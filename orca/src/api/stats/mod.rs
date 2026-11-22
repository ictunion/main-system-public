use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

mod query;

use super::Response;
use crate::api::{AppState, applications};
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

async fn applications_basic_stats(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<ApplicationsBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.decode_jwt(&token)?;

    Ok(Json(count_applications(&db_pool, None).await?))
}

/// Same counts, limited to applications assigned to the caller's executive
/// workplaces (tab counts of the rep "Workplace Applications" page).
async fn applications_workplace_basic_stats(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<ApplicationsBasicStats>> {
    let ids = applications::own_workplace_ids(&db_pool, &oid_provider, &token).await?;

    Ok(Json(count_applications(&db_pool, Some(&ids)).await?))
}

async fn count_applications(
    db_pool: &DbPool,
    workplace_ids: Option<&[Uuid]>,
) -> sqlx::Result<ApplicationsBasicStats> {
    Ok(ApplicationsBasicStats {
        unverified: query::count_unverified_applications(db_pool, workplace_ids).await?,
        accepted: query::count_accepted_applications(db_pool, workplace_ids).await?,
        rejected: query::count_rejected_applications(db_pool, workplace_ids).await?,
        processing: query::count_processing_applications(db_pool, workplace_ids).await?,
        invalid: query::count_invalid_applications(db_pool, workplace_ids).await?,
    })
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

async fn members_basic_stats(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<MembersBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.decode_jwt(&token)?;

    let new = query::count_new_members(&db_pool).await?;
    let current = query::count_current_members(&db_pool).await?;
    let past = query::count_past_members(&db_pool).await?;
    let workplace = query::count_workplace_members(&db_pool).await?;
    let sectoral = query::count_sectoral_members(&db_pool).await?;

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
    active: i64,
    in_preparation: i64,
}

async fn workplaces_basic_stats(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<WorkplacesBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.decode_jwt(&token)?;

    let counts = query::count_workplaces(&db_pool).await?;

    Ok(Json(WorkplacesBasicStats {
        current: counts.active + counts.in_preparation,
        past: counts.past,
        active: counts.active,
        in_preparation: counts.in_preparation,
    }))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/applications/basic", get(applications_basic_stats))
        .route(
            "/applications/workplace/basic",
            get(applications_workplace_basic_stats),
        )
        .route("/members/basic", get(members_basic_stats))
        .route("/workplaces/basic", get(workplaces_basic_stats))
}
