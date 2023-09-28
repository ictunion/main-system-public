use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{Route, State};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::api::Response;
use crate::data::{Id, Member, MemberNumber, RegistrationRequest};
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak, Role};
use crate::server::IpAddress;

use self::query::get_application_files;

use super::ApiError;

mod query;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct UnverifiedSummary {
    id: Id<RegistrationRequest>,
    email: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    phone_number: Option<String>,
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    verification_sent_at: Option<DateTime<Utc>>,
}

#[get("/unverified")]
async fn list_unverified<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<UnverifiedSummary>>> {
    keycloak.require_role(token, Role::ListApplications)?;

    let summaries = query::get_unverified_summaries()
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
    city: Option<String>,
    company_name: Option<String>,
    registration_local: String,
    created_at: DateTime<Utc>,
    confirmed_at: DateTime<Utc>,
}

#[get("/processing")]
async fn list_processing<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
) -> Response<Json<Vec<ProcessingSummary>>> {
    keycloak.require_role(token, Role::ListApplications)?;

    let summaries = query::get_processing_summaries()
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
    accepted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct File {
    id: Id<crate::api::files::File>,
    name: String,
    file_type: String,
    created_at: DateTime<Utc>,
}

#[get("/<id>")]
async fn detail<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    keycloak.require_any_role(token, &[Role::ViewApplication])?;

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
    accepted_at: Option<DateTime<Utc>>,
    member_id: Option<Id<Member>>,
}

impl ApplicationStatusData {
    pub fn to_status(&self) -> ApplicationStatus {
        if let Some(accepted_at) = self.accepted_at {
            if let Some(member_id) = self.member_id {
                return ApplicationStatus::Accepted(member_id, accepted_at);
            } else {
                error!(
                    "Broken invariant for status of application with id {}",
                    self.id
                );
            }
        }

        if let Some(rejected_at) = self.rejected_at {
            return ApplicationStatus::Rejected(rejected_at);
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
    Rejected(DateTime<Utc>),
    Accepted(Id<Member>, DateTime<Utc>),
}

impl ApplicationStatus {
    pub fn assert_in_proceesing(&self) -> Result<(), ApiError> {
        match self {
            Self::InProcessing => Ok(()),
            _ => Err(ApiError::DataConflict(format!(
                "Application status must be `InProcessing` but is {:?}",
                self
            ))),
        }
    }
}

#[delete("/<id>")]
async fn reject<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    keycloak.require_role(token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that the application status is in processing
    query::get_application_status_data(id)
        .fetch_one(&mut tx)
        .await?
        .to_status()
        .assert_in_proceesing()?;

    let detail = query::reject_application(id).fetch_one(&mut tx).await?;

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
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
    new_member: Json<NewMember>,
) -> Response<Json<Id<Member>>> {
    keycloak.require_role(token, Role::ResolveApplications)?;

    // transaction suppose to rollback on drop automatically
    // so we don't need to explicitely clean it
    let mut tx = db_pool.inner().begin().await?;

    // check that application status is in processing
    query::get_application_status_data(id)
        .fetch_one(&mut tx)
        .await?
        .to_status()
        .assert_in_proceesing()?;

    // Ensure member number for new member
    let member_number = match new_member.member_number {
        Some(v) => v,
        None => {
            let (new_num,) = query::get_next_member_number().fetch_one(&mut tx).await?;
            new_num
        }
    };

    // Inserting new member data
    let (member_id,) = query::create_new_member(id, member_number)
        .fetch_one(&mut tx)
        .await?;

    // Populate all the relations for new member
    query::attach_files_to_member(id, member_id)
        .execute(&mut tx)
        .await?;
    query::attach_occupation(id, member_id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(Json(member_id))
}

#[get("/<id>/files")]
async fn files<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Vec<File>>> {
    keycloak.require_role(token, Role::ViewApplication)?;

    let files = get_application_files(id).fetch_all(db_pool.inner()).await?;
    Ok(Json(files))
}

pub fn routes() -> Vec<Route> {
    routes![
        list_unverified,
        list_processing,
        detail,
        files,
        reject,
        accept
    ]
}
