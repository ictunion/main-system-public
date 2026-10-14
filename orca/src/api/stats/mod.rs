use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;

mod query;

use super::Response;
use crate::api::AppState;
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

    let unverified = query::count_unverified_applications(&db_pool).await?;
    let accepted = query::count_accepted_applications(&db_pool).await?;
    let rejected = query::count_rejected_applications(&db_pool).await?;
    let processing = query::count_processing_applications(&db_pool).await?;
    let invalid = query::count_invalid_applications(&db_pool).await?;

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
}

async fn workplaces_basic_stats(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<WorkplacesBasicStats>> {
    // Every authenticated user is able to see stats
    oid_provider.decode_jwt(&token)?;

    let current = query::count_current_workplaces(&db_pool).await?;
    let past = query::count_past_workplaces(&db_pool).await?;

    Ok(Json(WorkplacesBasicStats { current, past }))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/applications/basic", get(applications_basic_stats))
        .route("/members/basic", get(members_basic_stats))
        .route("/workplaces/basic", get(workplaces_basic_stats))
}
