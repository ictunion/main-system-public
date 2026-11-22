use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::Response;
use crate::api::files::FileInfo;
use crate::api::members;
use crate::api::workplaces::{self, WorkplaceSummary};
use crate::data::{Id, Member, MemberNumber, RegistrationRequest, Workplace};
use crate::db::{self, DbPool};
use crate::newsletter::Newsletter;
use crate::processing::{Command, QueueSender};
use crate::server::IpAddress;
use crate::server::oid::{JwtToken, Provider, Role};

use super::{ApiError, AppState, SuccessResponse};
use crate::api::extract::{ApiJson, ApiPath};

mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationUnverifiedSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    verification_sent_at: Option<DateTime<Utc>>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list_unverified(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationUnverifiedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_unverified_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationAcceptedSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    accepted_at: DateTime<Utc>,
    member_id: Id<Member>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list_accepted(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationAcceptedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_accepted_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationRejectedSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    rejected_at: DateTime<Utc>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list_rejected(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationRejectedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_rejected_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationProcessingSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    confirmed_at: DateTime<Utc>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list_processing(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationProcessingSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_processing_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationInvalidSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    invalidated_at: DateTime<Utc>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn list_invalid(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationInvalidSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_invalid_summaries(&db_pool, None).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApplicationDetail {
    id: Id<RegistrationRequest>,
    pub(crate) email: Option<String>,
    pub(crate) first_name: Option<String>,
    pub(crate) last_name: Option<String>,
    date_of_birth: Option<NaiveDate>,
    phone_number: Option<String>,
    note: Option<String>,
    pub(crate) city: Option<String>,
    address: Option<String>,
    postal_code: Option<String>,
    occupation: Option<String>,
    company_name: Option<String>,
    verification_sent_at: Option<DateTime<Utc>>,
    confirmed_at: Option<DateTime<Utc>>,
    registration_ip: Option<IpAddress>,
    registration_local: String,
    registration_user_agent: Option<String>,
    registration_source: Option<String>,
    rejected_at: Option<DateTime<Utc>>,
    invalidated_at: Option<DateTime<Utc>>,
    accepted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    pub(crate) language: Option<String>,
}

impl ApplicationDetail {
    /// Drop what a workplace executive committee has no need for. Same fields
    /// as `MemberDetail::redact_for_workplace_executive`, plus the request
    /// fingerprint. Done server-side so the reduced client never receives them.
    fn redact_for_workplace_executive(mut self) -> Self {
        self.note = None;
        self.date_of_birth = None;
        self.address = None;
        self.city = None;
        self.postal_code = None;
        self.registration_ip = None;
        self.registration_user_agent = None;
        self
    }
}

/// How much of an application the caller may see and do.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ApplicationScope {
    /// Admins: the whole record, every action their roles allow.
    Admin,
    /// Workplace executive committee, and the application is assigned to one
    /// of their workplaces: redacted view, approve/reject only.
    WorkplaceExecutive,
}

/// Decide whether the caller may act on this one application.
///
/// Any of `admin_roles` passes unrestricted. Otherwise the caller needs
/// `scoped_role` and the application must be assigned to a workplace whose
/// executive committee they sit on (resolved from Keycloak, never from the
/// URL). Out of scope is refused with the same error as holding no role.
async fn application_scope(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
    id: Id<RegistrationRequest>,
    admin_roles: &[Role],
    scoped_role: Role,
) -> Result<ApplicationScope, ApiError> {
    let Err(admin_denial) = oid_provider.require_any_role(token, admin_roles) else {
        return Ok(ApplicationScope::Admin);
    };

    if oid_provider.require_role(token, scoped_role).is_ok() {
        let workplace_ids =
            workplaces::executive_workplace_id_list(db_pool, oid_provider, token).await?;

        if !workplace_ids.is_empty()
            && query::is_application_of_workplaces(db_pool, id, &workplace_ids).await?
        {
            return Ok(ApplicationScope::WorkplaceExecutive);
        }
    }

    Err(admin_denial.into())
}

async fn detail(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    let scope = application_scope(
        &db_pool,
        &oid_provider,
        &token,
        id,
        &[Role::ViewApplication],
        Role::ListOwnWorkplaceApplications,
    )
    .await?;

    let detail = query::get_application(&db_pool, id).await?;

    Ok(Json(match scope {
        ApplicationScope::Admin => detail,
        ApplicationScope::WorkplaceExecutive => detail.redact_for_workplace_executive(),
    }))
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApplicationStatusData {
    id: Id<RegistrationRequest>,
    #[expect(
        dead_code,
        reason = "we don't need it now but it seems like a good idea to collect such an important info"
    )]
    created_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
    invalidated_at: Option<DateTime<Utc>>,
    accepted_at: Option<DateTime<Utc>>,
    member_id: Option<Uuid>,
}

impl ApplicationStatusData {
    pub fn to_status(&self) -> ApplicationStatus {
        if self.accepted_at.is_some() {
            if self.member_id.is_some() {
                return ApplicationStatus::Accepted;
            }
            error!(
                "Broken invariant for status of application with id {}",
                self.id
            );
        }

        if self.rejected_at.is_some() {
            return ApplicationStatus::Rejected;
        }

        if self.invalidated_at.is_some() {
            return ApplicationStatus::Invalid;
        }

        if self.confirmed_at.is_some() {
            return ApplicationStatus::InProcessing;
        }

        ApplicationStatus::WaitingForConfirmation
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationStatus {
    WaitingForConfirmation,
    InProcessing,
    Rejected,
    Accepted,
    Invalid,
}

impl ApplicationStatus {
    fn assert_status(self, status: Self) -> Result<(), ApiError> {
        if self == status {
            return Ok(());
        }
        let message = format!("Application status must be `{status:?}` but is `{self:?}`.",);

        Err(ApiError::data_conflict(&message))
    }

    pub fn assert_waiting_for_confirmation(self) -> Result<(), ApiError> {
        self.assert_status(Self::WaitingForConfirmation)
    }

    pub fn assert_in_proceesing(self) -> Result<(), ApiError> {
        self.assert_status(Self::InProcessing)
    }

    pub fn assert_waiting_or_in_processing_or(self) -> Result<(), ApiError> {
        if self.assert_in_proceesing().is_ok() || self.assert_waiting_for_confirmation().is_ok() {
            return Ok(());
        }

        let message = format!(
            "Application status must be `{:?}` or `{:?} but is `{:?}`.",
            ApplicationStatus::WaitingForConfirmation,
            ApplicationStatus::InProcessing,
            self
        );

        Err(ApiError::data_conflict(&message))
    }

    pub fn assert_rejected(self) -> Result<(), ApiError> {
        self.assert_status(Self::Rejected)
    }

    pub fn assert_invalidated(self) -> Result<(), ApiError> {
        self.assert_status(Self::Invalid)
    }

    /// Workplace assignment is frozen once the applicant became a member; from
    /// then on the member's own workplace (`members_workplaces`) is what counts.
    pub fn assert_not_accepted(self) -> Result<(), ApiError> {
        if self == Self::Accepted {
            return Err(ApiError::data_conflict(
                "Workplace of an accepted application can't be changed.",
            ));
        }
        Ok(())
    }
}

async fn reject(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    let scope = application_scope(
        &db_pool,
        &oid_provider,
        &token,
        id,
        &[Role::ResolveApplications],
        Role::ResolveOwnWorkplaceApplications,
    )
    .await?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.begin().await?;

    // check that the application status is in processing
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_waiting_or_in_processing_or()?;

    let detail = query::reject_application(&mut *tx, id).await?;
    query::clear_application_approval(&mut *tx, id).await?;

    tx.commit().await?;

    Ok(Json(match scope {
        ApplicationScope::Admin => detail,
        ApplicationScope::WorkplaceExecutive => detail.redact_for_workplace_executive(),
    }))
}

/// Workplace executive committee recommends the application for acceptance.
/// Staff still perform the accept (it needs Keycloak admin rights), with or
/// without this approval.
async fn approve(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<SuccessResponse> {
    // No staff role: approving is the workplace's say, staff just accept
    application_scope(
        &db_pool,
        &oid_provider,
        &token,
        id,
        &[],
        Role::ResolveOwnWorkplaceApplications,
    )
    .await?;

    let mut tx = db_pool.begin().await?;

    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_in_proceesing()?;

    query::approve_application(&mut *tx, id).await?;

    tx.commit().await?;

    // 202 with a body: melon-head's JSON client treats an empty 204 as an error
    Ok(SuccessResponse::Accepted)
}

async fn invalidate(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let mut tx = db_pool.begin().await?;

    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_waiting_or_in_processing_or()?;

    let detail = query::invalidate_application(&mut *tx, id).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

async fn unreject(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_rejected()?;

    let detail = query::unreject_application(&mut *tx, id).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

async fn uninvalidate(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let mut tx = db_pool.begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_invalidated()?;

    let detail = query::uninvalidate_application(&mut *tx, id).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
struct NewMember {
    member_number: Option<MemberNumber>,
}

async fn accept(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    State(queue): State<QueueSender>,
    State(newsletter): State<Newsletter>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
    ApiJson(new_member): ApiJson<NewMember>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.begin().await?;

    // check that application status is in processing
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_in_proceesing()?;

    // Ensure member number for new member
    let member_number: MemberNumber = if let Some(v) = new_member.member_number {
        v
    } else {
        members::query::get_next_member_number(&mut *tx).await?
    };

    // Inserting new member data
    let result = query::create_new_member(&mut *tx, id, member_number).await;

    // Gracefully handle colision of member numbers
    if db::fail_duplicated(&result) {
        let message = format!(
            "Member number `{}` is already used for different member.",
            &member_number
        );

        return Err(ApiError::data_conflict(&message));
    }

    let member_id = result?;

    // Populate all the relations for new member
    query::attach_files_to_member(&mut *tx, id, member_id).await?;
    query::attach_occupation(
        &mut *tx,
        id,
        member_id,
        members::OccupationSource::Application.as_str(),
    )
    .await?;

    // Application could be assigned to workplace, we need to copy this assignment
    // also to the newly created member
    query::copy_workplace_to_member(&mut *tx, id, member_id).await?;

    // Since we return just member_id from the insert query
    // let's just do an extra query for application detail
    let detail: ApplicationDetail = query::get_application(&mut *tx, id).await?;

    tx.commit().await?;

    newsletter.create_member(&db_pool, &token, member_id).await;

    queue
        .send(Command::NewMemberCreated(
            member_id,
            Some(token.as_str().to_owned()),
        ))
        .await?;

    queue.send(Command::SendNotificationToTreasurer).await?;

    Ok(Json(detail))
}

async fn list_files(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<Vec<FileInfo>>> {
    oid_provider.require_role(&token, Role::ViewApplication)?;

    let files = query::list_application_files(&db_pool, id).await?;
    Ok(Json(files))
}

async fn verify(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = db_pool.begin().await?;

    // check that the application status is in processing
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_waiting_for_confirmation()?;

    let detail = query::verify_application(&mut *tx, id).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

async fn resend_email(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    State(queue): State<QueueSender>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    query::get_application_status_data(&db_pool, id)
        .await?
        .to_status()
        .assert_waiting_for_confirmation()?;

    queue.send(Command::ResentRegistrationEmail(id)).await?;

    Ok(SuccessResponse::Accepted)
}

async fn hard_delete(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let mut tx = db_pool.begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_invalidated()?;

    query::dangerous_hard_delete_application_data(&mut *tx, id).await?;
    query::dangerous_hard_delete_application(&mut *tx, id).await?;

    tx.commit().await?;

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
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
    ApiJson(note): ApiJson<Note>,
) -> Response<Json<ApplicationDetail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let detail = query::update_registration_note(&db_pool, id, &note).await?;

    Ok(Json(detail))
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApplicationWorkplaceRow {
    workplace_id: Id<Workplace>,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApplicationWorkplace {
    workplace: WorkplaceSummary,
    approved_by_reps_at: Option<DateTime<Utc>>,
}

async fn workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<Json<ApplicationWorkplace>> {
    application_scope(
        &db_pool,
        &oid_provider,
        &token,
        id,
        &[Role::ViewApplication],
        Role::ListOwnWorkplaceApplications,
    )
    .await?;

    let Some(row) = query::get_application_workplace(&db_pool, id).await? else {
        return Err(ApiError::Status(StatusCode::NOT_FOUND));
    };

    let workplace = workplaces::query::detail(&db_pool, row.workplace_id).await?;

    Ok(Json(ApplicationWorkplace {
        workplace,
        approved_by_reps_at: row.approved_by_reps_at,
    }))
}

async fn assign_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath((id, workplace_id)): ApiPath<(Id<RegistrationRequest>, Id<Workplace>)>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // 404 for an unknown workplace instead of a 500 from the FK violation
    workplaces::query::detail(&db_pool, workplace_id).await?;

    let mut tx = db_pool.begin().await?;

    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_not_accepted()?;

    let rows_affected = query::assign_workplace(&mut *tx, id, workplace_id).await?;

    tx.commit().await?;

    if rows_affected == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Created)
    }
}

async fn unassign_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
    ApiPath(id): ApiPath<Id<RegistrationRequest>>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let mut tx = db_pool.begin().await?;

    query::get_application_status_data(&mut *tx, id)
        .await?
        .to_status()
        .assert_not_accepted()?;

    let rows_affected = query::unassign_workplace(&mut *tx, id).await?;

    tx.commit().await?;

    if rows_affected == 0 {
        Ok(SuccessResponse::NoContent)
    } else {
        Ok(SuccessResponse::Accepted)
    }
}

/// Workplaces whose applications the caller may list as an executive; empty
/// (so every listing is empty) when they sit on no committee. No workplace ID
/// is taken from the URL, same as `/workplaces/mine/members`.
pub(crate) async fn own_workplace_ids(
    db_pool: &DbPool,
    oid_provider: &Provider,
    token: &JwtToken,
) -> Response<Vec<Uuid>> {
    oid_provider.require_role(token, Role::ListOwnWorkplaceApplications)?;

    workplaces::executive_workplace_id_list(db_pool, oid_provider, token).await
}

async fn list_workplace(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(query::list_summaries(&db_pool, Some(&ids)).await?))
}

async fn list_workplace_unverified(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationUnverifiedSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(
        query::list_unverified_summaries(&db_pool, Some(&ids)).await?,
    ))
}

async fn list_workplace_accepted(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationAcceptedSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(
        query::list_accepted_summaries(&db_pool, Some(&ids)).await?,
    ))
}

async fn list_workplace_rejected(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationRejectedSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(
        query::list_rejected_summaries(&db_pool, Some(&ids)).await?,
    ))
}

async fn list_workplace_processing(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationProcessingSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(
        query::list_processing_summaries(&db_pool, Some(&ids)).await?,
    ))
}

async fn list_workplace_invalid(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
    token: JwtToken,
) -> Response<Json<Vec<ApplicationInvalidSummary>>> {
    let ids = own_workplace_ids(&db_pool, &oid_provider, &token).await?;
    Ok(Json(
        query::list_invalid_summaries(&db_pool, Some(&ids)).await?,
    ))
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/unverified", get(list_unverified))
        .route("/accepted", get(list_accepted))
        .route("/rejected", get(list_rejected))
        .route("/processing", get(list_processing))
        .route("/invalid", get(list_invalid))
        .route("/workplace", get(list_workplace))
        .route("/workplace/unverified", get(list_workplace_unverified))
        .route("/workplace/accepted", get(list_workplace_accepted))
        .route("/workplace/rejected", get(list_workplace_rejected))
        .route("/workplace/processing", get(list_workplace_processing))
        .route("/workplace/invalid", get(list_workplace_invalid))
        .route("/{id}", get(detail).delete(reject))
        .route("/{id}/invalidate", patch(invalidate))
        .route("/{id}/unreject", patch(unreject))
        .route("/{id}/uninvalidate", patch(uninvalidate))
        .route("/{id}/accept", post(accept))
        .route("/{id}/approve", patch(approve))
        .route("/{id}/files", get(list_files))
        .route("/{id}/verify", patch(verify))
        .route("/{id}/resend-email", post(resend_email))
        .route("/{id}/hard", delete(hard_delete))
        .route("/{id}/note", patch(update_note))
        .route("/{id}/workplace", get(workplace).delete(unassign_workplace))
        .route("/{id}/workplace/{workplace_id}", put(assign_workplace))
}
