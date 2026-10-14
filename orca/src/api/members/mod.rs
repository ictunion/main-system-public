use std::collections::HashMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

pub mod query;

use super::{ApiError, AppState, SuccessResponse};
use crate::api::Response;
use crate::api::extract::{ApiJson, ApiPath};
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
    pub(crate) is_representative: Option<bool>,
    pub(crate) sub: Option<Uuid>,
}

async fn list_all(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_summaries(&db_pool).await?;
    Ok(Json(summaries))
}

async fn list_past(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_past_summaries(&db_pool).await?;
    Ok(Json(summaries))
}

async fn list_new(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_new_summaries(&db_pool).await?;
    Ok(Json(summaries))
}

async fn list_current(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<MemberSummary>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let summaries = query::list_current_summaries(&db_pool).await?;
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

async fn create_member(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    new_member: Validated<NewMember>,
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
pub struct EmailInfo {
    subject: String,
    body: String,
    variables: HashMap<String, String>,
}

const WRAPPER_CS: &str = include_str!("../../../email_templates/member_email_cs.mjml");
const WRAPPER_EN: &str = include_str!("../../../email_templates/member_email_en.mjml");

async fn send_email(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    State(queue): State<QueueSender>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
    ApiJson(email_info): ApiJson<EmailInfo>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let member_detail = query::detail(&db_pool, id).await?;

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
    is_representative: Option<bool>,
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
/// scope is refused with the same error as a caller holding no role at all.
async fn member_read_scope(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
    id: Id<Member>,
) -> Result<MemberReadScope, ApiError> {
    let staff_roles = [Role::ListMembers];

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

async fn detail(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<MemberDetail>> {
    let scope = member_read_scope(&db_pool, &oid_provider, &token, id).await?;

    let detail = query::detail(&db_pool, id).await?;

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

async fn accept(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_realm_role(&token, RealmManagementRole::ManageUsers)?;

    let status = query::get_status_data(&db_pool, id).await?;

    if status.onboarding_finished_at.is_some() {
        return Err(ApiError::data_conflict("Member is accepted already"));
    }

    if status.left_at.is_some() {
        return Err(ApiError::data_conflict("Past members can't be activated"));
    }

    let detail = query::set_onboarding_finished(&db_pool, id).await?;

    Ok(Json(detail))
}

async fn list_files(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<Vec<FileInfo>>> {
    oid_provider.require_role(&token, Role::ListMembers)?;

    let files = query::list_member_files(&db_pool, id).await?;

    Ok(Json(files))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Occupation {
    id: Id<Occupation>,
    company_name: Option<String>,
    position: Option<String>,
    source: String,
    created_at: DateTime<Utc>,
}

/// Known values for `occupations.source`. The column is plain `TEXT`, not a DB
/// enum, so this doesn't constrain what can be stored -- it just documents the
/// values our code actually writes.
#[derive(Debug, Clone, Copy)]
pub enum OccupationSource {
    /// Submitted by the applicant as part of their registration.
    Application,
    /// Added by staff on the member detail page in Melon Head. Not constructed
    /// server-side -- Melon Head sends the literal string in the request body
    /// -- kept here so the value is documented alongside `Application`.
    #[allow(
        dead_code,
        reason = "documents a value Melon Head sends but orca never constructs"
    )]
    Orca,
}

impl OccupationSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::Orca => "orca",
        }
    }
}

async fn list_occupations(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<Vec<Occupation>>> {
    // Staff see the full employment history; a workplace executive sees only the
    // member's current occupation -- enough to know where a colleague works now,
    // without a record of previous employers.
    let scope = member_read_scope(&db_pool, &oid_provider, &token, id).await?;

    let occupations = match scope {
        MemberReadScope::Full => query::list_occupations(&db_pool, id).await?,
        MemberReadScope::WorkplaceExecutive => query::latest_occupation(&db_pool, id).await?,
    };

    Ok(Json(occupations))
}

#[derive(Debug, Deserialize, Validate)]
pub struct NewOccupation {
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    company_name: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    position: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "super::validate_non_empty"))]
    source: Option<String>,
}

async fn create_occupation(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
    data: Validated<NewOccupation>,
) -> Response<Json<Occupation>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let data = data.into_inner();
    let occupation = query::create_occupation(
        &db_pool,
        id,
        data.company_name.as_deref().expect("validated"),
        data.position.as_deref().expect("validated"),
        data.source.as_deref().expect("validated"),
    )
    .await?;

    Ok(Json(occupation))
}

async fn delete_occupation(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((id, occupation_id)): ApiPath<(Id<Member>, Id<Occupation>)>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    query::delete_occupation(&db_pool, id, occupation_id).await?;

    Ok(SuccessResponse::Accepted)
}

#[derive(Debug, Deserialize)]
pub struct Note {
    note: Option<String>,
}

async fn update_note(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
    ApiJson(note): ApiJson<Note>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::update_member_note(&db_pool, id, &note).await?;

    Ok(Json(detail))
}

#[derive(Debug, Deserialize, Validate)]
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

async fn update_member(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
    data: Validated<UpdateMember>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let result = query::update_member(&db_pool, id, data.into_inner()).await?;

    Ok(Json(result))
}

async fn remove_member(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let mut tx = db_pool.begin().await?;

    let status = query::get_status_data(&mut *tx, id).await?;

    // Remove from keycloak if paired
    if let Some(uuid) = status.sub {
        oid_provider.remove_user(&token, uuid).await?;
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

async fn list_candidate_users(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<Json<Vec<User>>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::detail(&db_pool, id).await?;

    match detail.email {
        Some(email) => Ok(Json(oid_provider.get_matching_users(&token, email).await?)),
        None => Ok(Json(Vec::new())),
    }
}

#[derive(Debug, Deserialize)]
struct PairRequest {
    sub: Uuid,
}

async fn create_oid_account(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    State(queue): State<QueueSender>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let status = query::get_status_data(&db_pool, id).await?;

    if status.sub().is_some() {
        return Err(ApiError::data_conflict("Member already has an OID account"));
    }

    queue
        .send(Command::NewMemberCreated(
            id,
            Some(token.as_str().to_owned()),
        ))
        .await?;

    Ok(SuccessResponse::Accepted)
}

async fn add_to_oid_group(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((id, group_id)): ApiPath<(Id<Member>, String)>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let group_id = Uuid::parse_str(&group_id).map_err(|_err| StatusCode::BAD_REQUEST)?;

    let status = query::get_status_data(&db_pool, id).await?;
    let sub = status
        .sub()
        .ok_or_else(|| ApiError::data_conflict("Member has no OID account"))?;

    oid_provider
        .connect_keycloak_user_and_group(&token, sub, group_id)
        .await?;

    Ok(SuccessResponse::Accepted)
}

async fn pair_oid(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<Member>>,
    ApiJson(data): ApiJson<PairRequest>,
) -> Response<Json<MemberDetail>> {
    oid_provider.require_role(&token, Role::ManageMembers)?;

    let detail = query::assign_member_oid_sub(&db_pool, id, data.sub).await?;

    Ok(Json(detail))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_all).post(create_member))
        .route("/past", get(list_past))
        .route("/new", get(list_new))
        .route("/current", get(list_current))
        .route("/{id}/send_email", post(send_email))
        .route(
            "/{id}",
            get(detail).patch(update_member).delete(remove_member),
        )
        .route("/{id}/accept", patch(accept))
        .route("/{id}/files", get(list_files))
        .route(
            "/{id}/occupations",
            get(list_occupations).post(create_occupation),
        )
        .route(
            "/{id}/occupations/{occupation_id}",
            delete(delete_occupation),
        )
        .route("/{id}/note", patch(update_note))
        .route("/{id}/list_candidate_users", get(list_candidate_users))
        .route("/{id}/create_oid_account", post(create_oid_account))
        .route("/{id}/oidc_groups/{group_id}", put(add_to_oid_group))
        .route("/{id}/pair_oid", patch(pair_oid))
}
