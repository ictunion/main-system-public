use super::{ApiError, SuccessResponse};
use crate::api::Response;
use crate::api::members::Summary;
use crate::api::members::query::get_status_data;
use crate::data::{Id, Member, Workplace};
use crate::db::DbPool;
use crate::server::oid::{JwtToken, Provider, Role};
use crate::validation::Validated;
use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{Route, State, delete, get, patch, post, put, routes};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

pub mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct WorkplaceSummary {
    id: Id<Workplace>,
    name: String,
    email: String,
    created_at: DateTime<Utc>,
    keycloak_group_id: Uuid,
    keycloak_executive_group_id: Option<Uuid>,
    announced_at: Option<DateTime<Utc>>,
    established_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    newsletter_id: Option<i32>,
    member_count: i64,
}

#[get("/")]
async fn list_all(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    // We should restrict this only for Admins/Board (people with ManageWorkplaces), until we have permissions to ViewMember separated by workplace
    // If we allowed access to this EP to anyone with ListWorkplaces, reps could see members from other workplaces
    // We still need to be careful when sharing links to specific workplaces, because every rep will have combination of ListWorkplaces and ViewMember, which allow them to open link to list of members of any workplace
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_summaries(db_pool.inner()).await?;

    Ok(Json(summaries))
}

#[get("/active")]
async fn list_active(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_active_summaries(db_pool.inner()).await?;

    Ok(Json(summaries))
}

#[get("/inactive")]
async fn list_inactive(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<WorkplaceSummary>>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let summaries = query::list_inactive_summaries(db_pool.inner()).await?;

    Ok(Json(summaries))
}

#[get("/<workplace_id>")]
async fn detail(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_any_role(&token, &[Role::ListWorkplaces])?;

    let detail = query::detail(db_pool.inner(), workplace_id).await?;

    Ok(Json(detail))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewWorkplace {
    #[validate(required)]
    name: Option<String>,
    #[validate(required)]
    email: Option<String>,
    #[validate(required)]
    keycloak_group_id: Option<Uuid>,
    #[validate(required)]
    keycloak_executive_group_id: Option<Uuid>,
    newsletter_id: Option<i32>,
}

#[post("/", format = "json", data = "<new_workplace>")]
async fn create_workplace(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    new_workplace: Validated<Json<NewWorkplace>>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    // Create new workplace
    let workplace =
        query::create_workplace(db_pool.inner(), &new_workplace.into_inner().into_inner()).await?;

    Ok(Json(workplace))
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkplace {
    newsletter_id: Option<i32>,
}

#[patch("/<workplace_id>", format = "json", data = "<update>")]
async fn update(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
    update: Json<UpdateWorkplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace =
        query::update_workplace(db_pool.inner(), workplace_id, &update.into_inner()).await?;

    Ok(Json(workplace))
}

#[patch("/<workplace_id>/establish")]
async fn establish(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::establish_workplace(db_pool.inner(), workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be established in its current state")
        })?;

    Ok(Json(workplace))
}

#[patch("/<workplace_id>/announce")]
async fn announce(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::announce_workplace(db_pool.inner(), workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be announced in its current state")
        })?;

    Ok(Json(workplace))
}

#[patch("/<workplace_id>/cancel")]
async fn cancel(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
) -> Response<Json<WorkplaceSummary>> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let workplace = query::cancel_workplace(db_pool.inner(), workplace_id)
        .await?
        .ok_or_else(|| {
            ApiError::data_conflict("Workplace cannot be cancelled in its current state")
        })?;

    Ok(Json(workplace))
}

#[put("/<workplace_id>/members/<member_id>")]
async fn assign_member_to_workplace(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let result = query::create_connection_between_member_and_workplace(workplace_id, member_id)
        .execute(db_pool.inner())
        .await?;

    let workplace_details = query::detail(db_pool.inner(), workplace_id).await?;

    let user_data = get_status_data(member_id)
        .fetch_one(db_pool.inner())
        .await?;

    let keycloak_id = user_data.sub().ok_or_else(|| {
        ApiError::keycloak_push(&format!("keycloak ID for user {member_id} not assigned"))
    })?;

    oid_provider
        .connect_keycloak_user_and_group(&token, keycloak_id, workplace_details.keycloak_group_id)
        .await?;

    if result.rows_affected() == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Created)
    }
}

#[delete("/<workplace_id>/members/<member_id>")]
async fn remove_member_from_workplace(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
    member_id: Id<Member>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageWorkplaces)?;

    let result = query::remove_connection_between_member_and_workplace(workplace_id, member_id)
        .execute(db_pool.inner())
        .await?;

    let workplace_details = query::detail(db_pool.inner(), workplace_id).await?;

    let user_data = get_status_data(member_id)
        .fetch_one(db_pool.inner())
        .await?;

    let keycloak_id = user_data.sub().ok_or_else(|| {
        ApiError::keycloak_push(&format!("keycloak ID for user {member_id} not assigned"))
    })?;

    oid_provider
        .remove_keycloak_user_from_group(&token, keycloak_id, workplace_details.keycloak_group_id)
        .await?;

    if result.rows_affected() == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Accepted)
    }
}

#[get("/<workplace_id>/members")]
async fn get_all_workplace_members(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    workplace_id: Id<Workplace>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListWorkplaces)?;
    oid_provider.require_role(&token, Role::ViewMember)?;

    let summaries = query::get_all_workplace_members(workplace_id)
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[expect(clippy::redundant_type_annotations, reason = "rocket macro expansion")]
pub fn routes() -> Vec<Route> {
    routes![
        list_all,
        list_active,
        list_inactive,
        detail,
        update,
        establish,
        announce,
        cancel,
        create_workplace,
        assign_member_to_workplace,
        remove_member_from_workplace,
        get_all_workplace_members,
    ]
}
