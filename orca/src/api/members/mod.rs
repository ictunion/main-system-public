use chrono::{DateTime, NaiveDate, Utc};
use rocket::serde::json::Json;
use rocket::{delete, get, patch, post, routes, Route, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

pub mod query;

use super::ApiError;
use crate::api::files::FileInfo;
use crate::api::Response;
use crate::data::{Id, Member, MemberNumber, RegistrationRequest, Workplace};
use crate::db::DbPool;
use crate::server::oid::{self, JwtToken, Provider, RealmManagementRole, Role};
use crate::validation::Validated;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Summary {
    id: Id<Member>,
    member_number: MemberNumber,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    left_at: Option<DateTime<Utc>>,
    company_names: Vec<Option<String>>,
    created_at: DateTime<Utc>,
}

#[get("/")]
async fn list_all<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[get("/past")]
async fn list_past<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_past_summaries()
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(summaries))
}

#[get("/new")]
async fn list_new<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_new_summaries()
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(summaries))
}

#[get("/current")]
async fn list_current<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_current_summaries()
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(summaries))
}

#[derive(Deserialize, Validate)]
pub struct NewMember {
    member_number: Option<MemberNumber>,
    first_name: Option<String>,
    last_name: Option<String>,
    date_of_birth: Option<NaiveDate>,
    #[validate(required)]
    #[validate(email)]
    email: Option<String>,
    phone_number: Option<String>,
    city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    language: String,
}

#[post("/", format = "json", data = "<new_member>")]
async fn create_member<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    new_member: Validated<Json<NewMember>>,
) -> Response<Json<Summary>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;
    let member = new_member.into_inner();

    // ensure member number
    let member_number = match member.member_number {
        Some(num) => num,
        None => {
            let (new_num,) = query::get_next_member_number().fetch_one(&mut *tx).await?;
            new_num
        }
    };

    // Create new member
    let summary = query::create_member(member_number, &member)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(summary))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Detail {
    id: Id<Member>,
    member_number: MemberNumber,
    first_name: Option<String>,
    last_name: Option<String>,
    date_of_birth: Option<NaiveDate>,
    email: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    address: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    language: Option<String>,
    application_id: Option<Id<RegistrationRequest>>,
    left_at: Option<DateTime<Utc>>,
    onboarding_finished_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    workplace_id: Option<Id<Workplace>>,
}

#[get("/<id>")]
async fn detail<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Detail>> {
    oid_provider.require_any_role(&token, &[Role::ListMembers, Role::ViewMember])?;

    let detail = query::detail(id).fetch_one(db_pool.inner()).await?;

    Ok(Json(detail))
}

#[derive(Debug, sqlx::FromRow)]
pub struct MemberStatusData {
    sub: Option<Uuid>,
    left_at: Option<DateTime<Utc>>,
}

#[patch("/<id>/accept")]
async fn accept<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Detail>> {
    oid_provider.require_realm_role(&token, RealmManagementRole::ManageUsers)?;

    let status = query::get_status_data(id)
        .fetch_one(db_pool.inner())
        .await?;

    if status.sub.is_some() {
        return Err(ApiError::data_conflict(
            "Member is accepted already".to_string(),
        ));
    }

    if status.left_at.is_some() {
        return Err(ApiError::data_conflict(
            "Past members can't be activated".to_string(),
        ));
    }

    let user = query::get_new_oid_user(id)
        .fetch_one(db_pool.inner())
        .await?;

    let uuid = oid_provider.create_user(&token, &user).await?;

    let detail = query::assign_member_oid_sub(id, uuid)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[get("/<id>/files")]
async fn list_files<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Vec<FileInfo>>> {
    oid_provider.require_any_role(&token, &[Role::ListMembers, Role::ViewApplication])?;

    let files = query::list_member_files(id)
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(files))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Occupation {
    id: Id<Occupation>,
    company_name: Option<String>,
    position: Option<String>,
    created_at: DateTime<Utc>,
}

#[get("/<id>/occupations")]
async fn list_occupations<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Vec<Occupation>>> {
    oid_provider.require_any_role(&token, &[Role::ListMembers, Role::ViewMember])?;

    let occupations = query::list_occupations(id)
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(occupations))
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Note {
    note: Option<String>,
}

#[patch("/<id>/note", format = "json", data = "<note>")]
async fn update_note<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
    note: Json<Note>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::update_member_note(id, &note)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[derive(Debug, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct UpdateMember {
    first_name: Option<String>,
    last_name: Option<String>,
    date_of_birth: Option<NaiveDate>,
    #[validate(required)]
    #[validate(email)]
    email: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    address: Option<String>,
    city: Option<String>,
    postal_code: Option<String>,
    language: String,
}

#[patch("/<id>", format = "json", data = "<data>")]
async fn update_member<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
    data: Validated<Json<UpdateMember>>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let result = query::update_member(id, data.into_inner().into_inner())
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(result))
}

#[delete("/<id>", format = "json")]
async fn remove_member<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;

    let status = query::get_status_data(id).fetch_one(&mut *tx).await?;

    // Remove from keycloak if paired
    if let Some(uuid) = status.sub {
        oid_provider.inner().remove_user(&token, uuid).await?;
    }

    if status.left_at.is_some() {
        return Err(ApiError::data_conflict(format!(
            "Id {} is no longer a member of organization",
            id
        )));
    }

    // Mark in database
    let detail = query::remove_member(id).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[get("/<id>/list_candidate_users")]
async fn list_candidate_users<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
) -> Response<Json<Vec<oid::User>>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::detail(id).fetch_one(db_pool.inner()).await?;

    match detail.email {
        Some(email) => Ok(Json(oid_provider.get_matching_users(&token, email).await?)),
        None => Ok(Json(Vec::new())),
    }
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct PairRequest {
    sub: Uuid,
}

#[patch("/<id>/pair_oid", format = "json", data = "<data>")]
async fn pair_oid<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<Member>,
    data: Json<PairRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::assign_member_oid_sub(id, data.sub)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_all,
        list_past,
        list_new,
        list_current,
        create_member,
        list_files,
        list_occupations,
        detail,
        accept,
        update_note,
        update_member,
        remove_member,
        list_candidate_users,
        pair_oid,
    ]
}
