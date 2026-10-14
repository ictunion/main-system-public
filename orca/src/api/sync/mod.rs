use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::NaiveDate;
use serde::Serialize;
use std::sync::Arc;

mod query;

use super::Response;
use crate::api::AppState;
use crate::config::Config;
use crate::data::MemberNumber;
use crate::db::DbPool;
use crate::server::oid::JwtToken;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BankSyncMember {
    sub: Option<Uuid>,
    member_number: MemberNumber,
    fee_start_date: Option<NaiveDate>,
    /// Date the member's fee liability ends (the day they left). Null while the
    /// member is still active. Symmetric with `fee_start_date`.
    fee_stop_date: Option<NaiveDate>,
    active: bool,
    /// Keycloak executive committee group id of the member's workplace. Null if
    /// the member isn't assigned to a workplace, or their workplace has no
    /// executive committee group configured. If a member is assigned to more
    /// than one workplace, the one first by name is used.
    workplace_executive_committee_sub: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct BankSyncMembersResponse {
    members: Vec<BankSyncMember>,
}

/// Constant-time compare, so response timing can't leak `sync_token`.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

async fn list_bank_members(
    State(db_pool): State<DbPool>,
    State(config): State<Arc<Config>>,
    token: JwtToken,
) -> Response<Json<BankSyncMembersResponse>> {
    let expected_token = config
        .bank_sync_token
        .as_deref()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !constant_time_eq(token.as_str().as_bytes(), expected_token.as_bytes()) {
        return Err(StatusCode::UNAUTHORIZED.into());
    }

    let members = query::list_bank_sync_members(&db_pool).await?;

    Ok(Json(BankSyncMembersResponse { members }))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/bank/members", get(list_bank_members))
}
