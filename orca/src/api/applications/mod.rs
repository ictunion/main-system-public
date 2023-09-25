use chrono::{DateTime, Utc};
use rocket::serde::json::Json;
use rocket::{Route, State};
use serde::Serialize;
use time::Date;

use crate::api::Response;
use crate::data::{Id, RegistrationRequest};
use crate::db::DbPool;
use crate::server::keycloak::{JwtToken, Keycloak, Role};
use crate::server::IpAddress;

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

#[get("/<id>")]
async fn detail<'r>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
    token: JwtToken<'r>,
    id: Id<RegistrationRequest>,
) -> Response<Json<Detail>> {
    keycloak.require_any_role(token, &[Role::ViewApplication, Role::ListApplications])?;

    let detail = query::get_application(id)
        .fetch_one(db_pool.inner())
        .await?;

    Ok(Json(detail))
}

pub fn routes() -> Vec<Route> {
    routes![list_unverified, list_processing, detail]
}
