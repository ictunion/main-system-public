use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use handlebars::Handlebars;
use rocket::serde::json::Json;
use rocket::{Route, State, delete, get, patch, post, put, routes};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

pub mod query;

use super::ApiError;
use super::SuccessResponse;
use crate::api::Response;
use crate::api::files::FileInfo;
use crate::data::{Id, Member, MemberNumber, RegistrationRequest, Workplace};
use crate::db::DbPool;
use crate::processing::{Command, QueueSender};
use crate::server::oid::{JwtToken, Provider, RealmManagementRole, Role, User};
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
    language: Option<String>,
    left_at: Option<DateTime<Utc>>,
    company_names: Vec<Option<String>>,
    created_at: DateTime<Utc>,
    workplace_ids: Vec<Uuid>,
    sub: Option<Uuid>,
}

#[get("/")]
async fn list_all(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[get("/past")]
async fn list_past(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_past_summaries()
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(summaries))
}

#[get("/new")]
async fn list_new(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_new_summaries()
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(summaries))
}

#[get("/current")]
async fn list_current(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
async fn create_member(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    new_member: Validated<Json<NewMember>>,
) -> Response<Json<Summary>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;
    let member = new_member.into_inner();

    // ensure member number
    let member_number = if let Some(num) = member.member_number {
        num
    } else {
        let (new_num,) = query::get_next_member_number().fetch_one(&mut *tx).await?;
        new_num
    };

    // Create new member
    let summary = query::create_member(member_number, &member)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(summary))
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct EmailInfo {
    subject: String,
    body: String,
    variables: HashMap<String, String>,
}

const WRAPPER_CS: &str = include_str!("../../../email_templates/member_email_cs.mjml");
const WRAPPER_EN: &str = include_str!("../../../email_templates/member_email_en.mjml");

#[post("/<id>/send_email", format = "json", data = "<request_email_info>")]
async fn send_email(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    queue: &State<QueueSender>,
    token: JwtToken<'_>,
    request_email_info: Json<EmailInfo>,
    id: Id<Member>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let email_info = request_email_info.into_inner();
    let member_detail = query::detail(id).fetch_one(db_pool.inner()).await?;

    let wrapper = match member_detail.language.as_deref() {
        Some("cs") => WRAPPER_CS,
        _ => WRAPPER_EN,
    };
    let message_mjml = Handlebars::new()
        .render_template(
            &wrapper.replace("{body}", &email_info.body),
            &email_info.variables,
        )
        .unwrap();

    let full_name = format!(
        "{} {}",
        member_detail.first_name.as_deref().unwrap_or(""),
        member_detail.last_name.as_deref().unwrap_or("")
    );
    let email = member_detail.email.as_deref().unwrap_or("").to_string();

    queue
        .inner()
        .send(Command::SendEmailAsTreasurer(
            full_name,
            email_info.subject,
            email,
            message_mjml,
        ))
        .await?;

    Ok(SuccessResponse::Accepted)
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
    sub: Option<Uuid>,
}

#[get("/<id>")]
async fn detail(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
    onboarding_finished_at: Option<DateTime<Utc>>,
}

impl MemberStatusData {
    pub(crate) fn sub(&self) -> Option<Uuid> {
        self.sub
    }
}

#[patch("/<id>/accept")]
async fn accept(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
) -> Response<Json<Detail>> {
    oid_provider.require_realm_role(&token, RealmManagementRole::ManageUsers)?;

    let status = query::get_status_data(id)
        .fetch_one(db_pool.inner())
        .await?;

    if status.onboarding_finished_at.is_some() {
        return Err(ApiError::data_conflict("Member is accepted already"));
    }

    if status.left_at.is_some() {
        return Err(ApiError::data_conflict("Past members can't be activated"));
    }

    let detail = query::set_onboarding_finished(id)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[get("/<id>/files")]
async fn list_files(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
async fn list_occupations(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
async fn update_note(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
async fn update_member(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
async fn remove_member(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
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
        return Err(ApiError::data_conflict(&format!(
            "Id {id} is no longer a member of organization"
        )));
    }

    // Mark in database and remove workplace associations
    super::workplaces::query::remove_member_workplace_associations(id)
        .execute(&mut *tx)
        .await?;
    let detail = query::remove_member(id).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[get("/<id>/list_candidate_users")]
async fn list_candidate_users(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
) -> Response<Json<Vec<User>>> {
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

#[post("/<id>/create_oid_account")]
async fn create_oid_account(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    queue: &State<QueueSender>,
    id: Id<Member>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let status = query::get_status_data(id)
        .fetch_one(db_pool.inner())
        .await?;

    if status.sub().is_some() {
        return Err(ApiError::data_conflict("Member already has an OID account"));
    }

    queue
        .inner()
        .send(Command::NewMemberCreated(
            id,
            Some(token.as_str().to_owned()),
        ))
        .await?;

    Ok(SuccessResponse::Accepted)
}

#[put("/<id>/oidc_groups/<group_id>")]
async fn add_to_oid_group(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
    group_id: &str,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let group_id = Uuid::parse_str(group_id).map_err(|_err| rocket::http::Status::BadRequest)?;

    let status = query::get_status_data(id)
        .fetch_one(db_pool.inner())
        .await?;
    let sub = status
        .sub()
        .ok_or_else(|| ApiError::data_conflict("Member has no OID account"))?;

    oid_provider
        .connect_keycloak_user_and_group(&token, sub, group_id)
        .await?;

    Ok(SuccessResponse::Accepted)
}

#[patch("/<id>/pair_oid", format = "json", data = "<data>")]
async fn pair_oid(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
    data: Json<PairRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::assign_member_oid_sub(id, data.sub)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[expect(clippy::redundant_type_annotations, reason = "rocket macro expansion")]
pub fn routes() -> Vec<Route> {
    routes![
        list_all,
        list_past,
        list_new,
        list_current,
        create_member,
        send_email,
        list_files,
        list_occupations,
        detail,
        accept,
        update_note,
        update_member,
        remove_member,
        list_candidate_users,
        add_to_oid_group,
        pair_oid,
        create_oid_account,
    ]
}
