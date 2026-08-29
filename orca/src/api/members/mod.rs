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
use crate::api::workplaces;
use crate::data::{Id, Member, MemberNumber};
use crate::db::DbPool;
use crate::processing::{Command, QueueSender};
use crate::server::oid::{JwtToken, Provider, RealmManagementRole, Role, User};
use crate::validation::Validated;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct MemberSummary {
    pub(crate) id: Id<Member>,
    pub(crate) member_number: MemberNumber,
    pub(crate) first_name: Option<String>,
    pub(crate) last_name: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) phone_number: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) city: Option<String>,
    pub(crate) language: Option<String>,
    pub(crate) left_at: Option<DateTime<Utc>>,
    pub(crate) company_names: Vec<Option<String>>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) workplace_ids: Vec<Uuid>,
    pub(crate) sub: Option<Uuid>,
}

#[get("/")]
async fn list_all(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_summaries(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[get("/past")]
async fn list_past(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_past_summaries(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[get("/new")]
async fn list_new(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_new_summaries(db_pool.inner()).await?;
    Ok(Json(summaries))
}

#[get("/current")]
async fn list_current(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_current_summaries(db_pool.inner()).await?;
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
) -> Response<Json<MemberSummary>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;
    let member = new_member.into_inner();

    // ensure member number
    let member_number = if let Some(num) = member.member_number {
        num
    } else {
        query::get_next_member_number(&mut *tx).await?
    };

    // Create new member
    let summary = query::create_member(&mut *tx, member_number, &member).await?;

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
    let member_detail = query::detail(db_pool.inner(), id).await?;

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
pub struct MemberDetail {
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
    application_id: Option<Uuid>,
    left_at: Option<DateTime<Utc>>,
    onboarding_finished_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    workplace_id: Option<Uuid>,
    sub: Option<Uuid>,
}

impl MemberDetail {
    /// Drop what a workplace executive committee has no organising need for.
    ///
    /// Done here rather than in the UI so that hiding these fields on screen is
    /// not load-bearing: the reduced client never receives them at all. Mirrors
    /// `WorkplaceMemberSummary` in `api::workplaces`, which is the list view of
    /// the same people.
    fn redact_for_workplace_executive(mut self) -> Self {
        self.note = None;
        self.date_of_birth = None;
        self.address = None;
        self.city = None;
        self.postal_code = None;
        self
    }
}

/// How much of a member record the caller is entitled to see.
#[derive(Debug, Clone, Copy, PartialEq)]
enum MemberReadScope {
    /// Staff: the whole record.
    Full,
    /// Workplace executive committee, and this member is in one of their
    /// workplaces: the reduced projection only.
    WorkplaceExecutive,
}

/// Decide whether the caller may read this one member, and how much of them.
///
/// Staff roles read anyone. An executive committee member reads only people in
/// the workplaces they sit on the committee of; that set is resolved from
/// Keycloak per request and never taken from the URL. A member outside their
/// scope is refused with the same error as a caller holding no role at all, so
/// the endpoint cannot be used to probe which member UUIDs exist.
async fn member_read_scope(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken<'_>,
    id: Id<Member>,
) -> Result<MemberReadScope, ApiError> {
    let staff_roles = [Role::ListMembers, Role::ViewMember];

    let Err(staff_denial) = oid_provider.require_any_role(token, &staff_roles) else {
        return Ok(MemberReadScope::Full);
    };

    if oid_provider
        .require_role(token, Role::ListOwnWorkplaceMembers)
        .is_ok()
    {
        let workplace_ids =
            workplaces::executive_workplace_id_list(db_pool, oid_provider, token).await?;

        if !workplace_ids.is_empty()
            && query::is_member_of_workplaces(db_pool, id, &workplace_ids).await?
        {
            return Ok(MemberReadScope::WorkplaceExecutive);
        }
    }

    Err(staff_denial.into())
}

#[get("/<id>")]
async fn detail(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
) -> Response<Json<MemberDetail>> {
    let scope = member_read_scope(db_pool.inner(), oid_provider.inner(), &token, id).await?;

    let detail = query::detail(db_pool.inner(), id).await?;

    Ok(Json(match scope {
        MemberReadScope::Full => detail,
        MemberReadScope::WorkplaceExecutive => detail.redact_for_workplace_executive(),
    }))
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
) -> Response<Json<MemberDetail>> {
    oid_provider.require_realm_role(&token, RealmManagementRole::ManageUsers)?;

    let status = query::get_status_data(db_pool.inner(), id).await?;

    if status.onboarding_finished_at.is_some() {
        return Err(ApiError::data_conflict("Member is accepted already"));
    }

    if status.left_at.is_some() {
        return Err(ApiError::data_conflict("Past members can't be activated"));
    }

    let detail = query::set_onboarding_finished(db_pool.inner(), id).await?;

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

    let files = query::list_member_files(db_pool.inner(), id).await?;

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
    // Company and position are exactly what a workplace rep needs, so this is
    // scoped the same way as the member detail rather than being staff-only.
    member_read_scope(db_pool.inner(), oid_provider.inner(), &token, id).await?;

    let occupations = query::list_occupations(db_pool.inner(), id).await?;

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
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::update_member_note(db_pool.inner(), id, &note).await?;

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
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let result = query::update_member(db_pool.inner(), id, data.into_inner().into_inner()).await?;

    Ok(Json(result))
}

#[delete("/<id>", format = "json")]
async fn remove_member(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'_>,
    id: Id<Member>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;

    let status = query::get_status_data(&mut *tx, id).await?;

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
    super::workplaces::query::remove_member_workplace_associations(&mut *tx, id).await?;
    let detail = query::remove_member(&mut *tx, id).await?;

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

    let detail = query::detail(db_pool.inner(), id).await?;

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

    let status = query::get_status_data(db_pool.inner(), id).await?;

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

    let status = query::get_status_data(db_pool.inner(), id).await?;
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
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::assign_member_oid_sub(db_pool.inner(), id, data.sub).await?;

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
