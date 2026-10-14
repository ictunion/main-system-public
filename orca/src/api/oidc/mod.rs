use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::extract::ApiPath;
use crate::api::{ApiError, AppState, Response};
use crate::server::oid::{self, JwtToken, Provider, Role};

#[derive(Debug, Serialize)]
pub struct OidGroupMember {
    id: Uuid,
}

async fn list_oid_group_members(
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(group_id): ApiPath<String>,
) -> Response<Json<Vec<OidGroupMember>>> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let group_id = Uuid::parse_str(&group_id).map_err(|_err| ApiError::invalid_group_id())?;
    let ids = oid_provider
        .get_group_members(&token, group_id)
        .await
        .map_err(|err| match err {
            oid::Error::Proxy(s) if s.as_u16() == 404 => ApiError::oidc_group_not_found(),
            other => other.into(),
        })?;
    let members = ids.into_iter().map(|id| OidGroupMember { id }).collect();
    Ok(Json(members))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new().route("/{group_id}", get(list_oid_group_members))
}
