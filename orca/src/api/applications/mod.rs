use chrono::{DateTime, Utc};
use log::error;
use rocket::serde::json::Json;
use rocket::{delete, get, patch, post, routes, Route, State};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::api::files::FileInfo;
use crate::api::members;
use crate::api::Response;
use crate::data::{Id, Member, MemberNumber, RegistrationRequest};
use crate::db::{self, DbPool};
use crate::processing::{Command, QueueSender};
use crate::server::oid::{JwtToken, Provider, Role};
use crate::server::IpAddress;

use super::{ApiError, SuccessResponse};

mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Summary {
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
}

#[get("/")]
async fn list<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<Summary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_summaries().fetch_all(db_pool.inner()).await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UnverifiedSummary {
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
}

#[get("/unverified")]
async fn list_unverified<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<UnverifiedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_unverified_summaries()
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AcceptedSummary {
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
}

#[get("/accepted")]
async fn list_accepted<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<AcceptedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_accepted_summaries()
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RejectedSummary {
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
}

#[get("/rejected")]
async fn list_rejected<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<RejectedSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_rejected_summaries()
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProcessingSummary {
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
}

#[get("/processing")]
async fn list_processing<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<ProcessingSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_processing_summaries()
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct InvalidSummary {
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
}

#[get("/invalid")]
async fn list_invalid<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<InvalidSummary>>> {
    oid_provider.require_role(&token, Role::ListApplications)?;

    let summaries = query::list_invalid_summaries()
        .fetch_all(db_pool.inner())
        .await?;

    Ok(Json(summaries))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Detail {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    date_of_birth: Option<Date>,
    phone_number: Option<String>,
    note: Option<String>,
    city: Option<String>,
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
}

#[get("/<id>")]
async fn detail<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_any_role(&token, &[Role::ViewApplication])?;

    let detail = query::get_application(id)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

#[derive(Debug, sqlx::FromRow)]
pub struct ApplicationStatusData {
    id: Id<RegistrationRequest>,
    #[allow(dead_code)]
    // we don't need it now but it seems like a good idea to collect such an important info
    created_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    rejected_at: Option<DateTime<Utc>>,
    invalidated_at: Option<DateTime<Utc>>,
    accepted_at: Option<DateTime<Utc>>,
    member_id: Option<Id<Member>>,
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

#[derive(Debug)]
pub enum ApplicationStatus {
    WaitingForConfirmation,
    InProcessing,
    Rejected,
    Accepted,
    Invalid,
}

impl ApplicationStatus {
    pub fn assert_waiting_for_confirmation(&self) -> Result<(), ApiError> {
        match self {
            Self::WaitingForConfirmation => Ok(()),
            _ => {
                let message = format!(
                    "Application status must be `{:?}` but is `{:?}`.",
                    ApplicationStatus::WaitingForConfirmation,
                    self
                );

                Err(ApiError::data_conflict(message))
            }
        }
    }

    pub fn assert_in_proceesing(&self) -> Result<(), ApiError> {
        match self {
            Self::InProcessing => Ok(()),

            _ => {
                let message = format!(
                    "Application status must be `{:?}` but is `{:?}`.",
                    ApplicationStatus::InProcessing,
                    self
                );

                Err(ApiError::data_conflict(message))
            }
        }
    }

    pub fn assert_waiting_or_in_processing_or(&self) -> Result<(), ApiError> {
        match self {
            Self::InProcessing => Ok(()),
            Self::WaitingForConfirmation => Ok(()),
            _ => {
                let message = format!(
                    "Application status must be `{:?}` or `{:?} but is `{:?}`.",
                    ApplicationStatus::WaitingForConfirmation,
                    ApplicationStatus::InProcessing,
                    self
                );

                Err(ApiError::data_conflict(message))
            }
        }
    }

    pub fn assert_rejected(&self) -> Result<(), ApiError> {
        match self {
            Self::Rejected => Ok(()),
            _ => {
                let message = format!("Application status must be `Rejected` but is `{:?}`.", self);

                Err(ApiError::data_conflict(message))
            }
        }
    }

    pub fn assert_invalidated(&self) -> Result<(), ApiError> {
        match self {
            Self::Invalid => Ok(()),
            _ => {
                let message = format!("Application status must be `Invalid` but is `{:?}`.", self);

                Err(ApiError::data_conflict(message))
            }
        }
    }
}

#[delete("/<id>")]
async fn reject<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is in processing
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_waiting_or_in_processing_or()?;

    let detail = query::reject_application(id).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[patch("/<id>/invalidate")]
async fn invalidate<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let mut tx = db_pool.inner().begin().await?;

    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_waiting_or_in_processing_or()?;

    let detail = query::invalidate_application(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[patch("/<id>/unreject")]
async fn unreject<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_rejected()?;

    let detail = query::unreject_application(id).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[patch("/<id>/uninvalidate")]
async fn uninvalidate<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_invalidated()?;

    let detail = query::uninvalidate_application(id)
        .fetch_one(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct NewMember {
    member_number: Option<MemberNumber>,
}

#[post("/<id>/accept", format = "json", data = "<new_member>")]
async fn accept<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
    new_member: Json<NewMember>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that application status is in processing
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_in_proceesing()?;

    // Ensure member number for new member
    let member_number = match new_member.member_number {
        Some(v) => v,
        None => {
            let (new_num,) = members::query::get_next_member_number()
                .fetch_one(&mut *tx)
                .await?;
            new_num
        }
    };

    // Inserting new member data
    let result = query::create_new_member(id, member_number)
        .fetch_one(&mut *tx)
        .await;

    // Gracefully handle colision of member numbers
    if db::fail_duplicated(&result) {
        let message = format!(
            "Member number `{}` is already used for different member.",
            &member_number
        );

        return Err(ApiError::data_conflict(message));
    }

    let (member_id,) = result?;

    // Populate all the relations for new member
    query::attach_files_to_member(id, member_id)
        .execute(&mut *tx)
        .await?;
    query::attach_occupation(id, member_id)
        .execute(&mut *tx)
        .await?;

    // Since we return just member_id from the insert query
    // let's just do an extra query for application detail
    let detail = query::get_application(id).fetch_one(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[get("/<id>/files")]
async fn list_files<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Vec<FileInfo>>> {
    oid_provider.require_role(&token, Role::ViewApplication)?;

    let files = query::list_application_files(id)
        .fetch_all(db_pool.inner())
        .await?;
    Ok(Json(files))
}

#[patch("/<id>/verify")]
async fn verify<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    queue: &State<QueueSender>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is in processing
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_waiting_for_confirmation()?;

    let detail = query::verify_application(id).fetch_one(&mut *tx).await?;

    // Trigger event to send a notification out
    queue
        .inner()
        .send(Command::RegistrationRequestVerified(id))
        .await?;

    tx.commit().await?;

    Ok(Json(detail))
}

#[post("/<id>/resend-email")]
async fn resend_email<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    queue: &State<QueueSender>,
    id: Id<RegistrationRequest>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    query::get_application_status_data(id)
        .fetch_one(db_pool.inner())
        .await?
        .to_status()
        .assert_waiting_for_confirmation()?;

    queue
        .inner()
        .send(Command::ResentRegistrationEmail(id))
        .await?;

    Ok(SuccessResponse::Accepted)
}

#[delete("/<id>/hard")]
async fn hard_delete<'r>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<SuccessResponse> {
    oid_provider.require_role(&token, Role::SuperPowers)?;

    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is rejected
    query::get_application_status_data(id)
        .fetch_one(&mut *tx)
        .await?
        .to_status()
        .assert_invalidated()?;

    query::dangerous_hard_delete_application_data(id)
        .execute(&mut *tx)
        .await?;
    query::dangerous_hard_delete_application(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(SuccessResponse::Accepted)
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
    id: Id<RegistrationRequest>,
    note: Json<Note>,
) -> Response<Json<Detail>> {
    oid_provider.require_role(&token, Role::ResolveApplications)?;

    let detail = query::update_registration_note(id, &note)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

pub fn routes() -> Vec<Route> {
    routes![
        list,
        list_unverified,
        list_processing,
        list_accepted,
        list_invalid,
        resend_email,
        list_rejected,
        detail,
        list_files,
        reject,
        invalidate,
        unreject,
        uninvalidate,
        verify,
        accept,
        hard_delete,
        update_note,
    ]
}
