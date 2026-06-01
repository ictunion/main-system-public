use rocket::serde::json::Json;
use rocket::{Route, State, get, routes};
use serde::Serialize;
use uuid::Uuid;

use crate::api::Response;
use crate::server::oid::{JwtToken, Provider, Role};

#[derive(Debug, Serialize)]
pub struct OidGroupMember {
    id: Uuid,
}

#[get("/<group_id>")]
async fn list_oid_group_members(
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    group_id: &str,
) -> Response<Json<Vec<OidGroupMember>>> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let group_id = Uuid::parse_str(group_id).map_err(|_err| rocket::http::Status::BadRequest)?;
    let ids = oid_provider.get_group_members(&token, group_id).await?;
    let members = ids.into_iter().map(|id| OidGroupMember { id }).collect();
    Ok(Json(members))
}

#[expect(clippy::redundant_type_annotations, reason = "rocket macro expansion")]
pub fn routes() -> Vec<Route> {
    routes![list_oid_group_members]
}
