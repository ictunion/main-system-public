use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{delete, get, post, routes, Route, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::SuccessResponse;
use crate::api::members::Summary;
use crate::api::Response;
use crate::data::{Id, Workplace};
use crate::db::DbPool;
use crate::server::oid::{JwtToken, Provider, Role};
use crate::validation::Validated;

pub mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkplaceSummary {
    id: Id<Workplace>,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
}

#[get("/")]
async fn list_all<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    // We should restrict this only for Admins/Board (people with ManageWorkplaces), until we have permissions to ViewMember separated by workplace
    // If we allowed access to this EP to anyone with ListWorkplaces, reps could see members from other workplaces
    // We still need to be careful when sharing links to specific workplaces, because every rep will have combination of ListWorkplaces and ViewMember, which allow them to open link to list of members of any workplace
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;

    Ok(Json(summaries))
}

#[get("/<workplace_id>")]
async fn detail<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    workplace_id: Id<Workplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_any_role(&token, &[Role::ListWorkplaces])?;

    let detail = query::detail(workplace_id)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewWorkplace {
    #[validate(required)]
    name: Option<String>,
    #[validate(required)]
    email: Option<String>,
}

#[post("/", format = "json", data = "<new_workplace>")]
async fn create_workplace<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    new_workplace: Validated<Json<NewWorkplace>>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    // Create new workplace
    let workplace = query::create_workplace(&new_workplace.into_inner().into_inner())
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(workplace))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewWorkplaceMember {
    member_id: Uuid,
}

#[post("/<workplace_id>", format = "json", data = "<new_workplace_member>")]
async fn assign_member_to_workplace<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    workplace_id: Id<Workplace>,
    new_workplace_member: Json<NewWorkplaceMember>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    // Create new connection
    query::create_connection_between_member_and_workplace(
        workplace_id,
        new_workplace_member.into_inner().member_id,
    )
    .execute(db_pool.inner())
    .await?;

    Ok(SuccessResponse::Accepted)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovedWorkplaceMember {
    member_id: Uuid,
}

#[delete(
    "/<workplace_id>",
    format = "json",
    data = "<removed_workplace_member>"
)]
async fn remove_member_from_workplace<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    workplace_id: Id<Workplace>,
    removed_workplace_member: Json<RemovedWorkplaceMember>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    // Remove existing connection
    query::remove_connection_between_member_and_workplace(
        workplace_id,
        removed_workplace_member.into_inner().member_id,
    )
    .execute(db_pool.inner())
    .await?;

    Ok(SuccessResponse::Accepted)
}

#[get("/<workplace_id>/members")]
async fn get_all_workplace_members<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    workplace_id: Id<Workplace>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListWorkplaces)?;
    oid_provider.require_role(&token, Role::ViewMember)?;

    let summaries = query::get_all_workplace_members(workplace_id)
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_all,
        detail,
        create_workplace,
        assign_member_to_workplace,
        remove_member_from_workplace,
        get_all_workplace_members,
    ]
}
