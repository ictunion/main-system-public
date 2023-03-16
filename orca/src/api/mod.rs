use rocket::response::{self, Responder};
use rocket::{Build, Request, Rocket};
use tokio::task::JoinError;
use validator::ValidationError;

mod registration;
use crate::processing::SenderError;

#[derive(Debug)]
pub struct SqlError(sqlx::Error);

impl From<sqlx::Error> for SqlError {
    fn from(err: sqlx::Error) -> Self {
        Self(err)
    }
}

impl<'r> Responder<'r, 'static> for SqlError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use rocket::http::Status;
        use sqlx::error::Error::*;

        match self.0 {
            RowNotFound => Err(Status::NotFound),
            PoolTimedOut => Err(Status::ServiceUnavailable),
            _ => Err(Status::InternalServerError),
        }
    }
}

#[derive(Debug)]
pub struct ThreadingError(JoinError);

impl From<JoinError> for ThreadingError {
    fn from(value: JoinError) -> Self {
        ThreadingError(value)
    }
}

impl<'r> response::Responder<'r, 'static> for ThreadingError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        Err(rocket::http::Status::InternalServerError)
    }
}

impl<'r> response::Responder<'r, 'static> for SenderError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        Err(rocket::http::Status::InternalServerError)
    }
}

#[derive(Debug, Responder)]
pub enum ApiError {
    DbErr(SqlError),
    Status(rocket::http::Status),
    ThreadFail(ThreadingError),
    QueueSender(SenderError),
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::DbErr(err.into())
    }
}

impl From<rocket::http::Status> for ApiError {
    fn from(status: rocket::http::Status) -> Self {
        Self::Status(status)
    }
}

impl From<JoinError> for ApiError {
    fn from(err: JoinError) -> Self {
        Self::ThreadFail(err.into())
    }
}

impl From<SenderError> for ApiError {
    fn from(err: SenderError) -> Self {
        Self::QueueSender(err)
    }
}

#[get("/status")]
fn status_api() -> &'static str {
    "OK"
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .register(
            "/registration",
            catchers![rocket_validation::validation_catcher],
        )
}

pub type Response<T> = Result<T, ApiError>;

/// This is pretty much a wrapper for `Status`
/// which doesn't allow for response body content
/// since it standardizes it for common cases.
#[derive(Debug)]
pub enum SuccessResponse {
    Accepted,
}

impl<'r> Responder<'r, 'static> for SuccessResponse {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        use rocket::response::status;
        use rocket::serde::json::Json;

        match &self {
            Self::Accepted => {
                status::Accepted(Some(Json("{ 'status': 202, 'message': 'Accepted' }")))
                    .respond_to(request)
            }
        }
    }
}

pub fn validate_non_empty(val: &str) -> Result<(), ValidationError> {
    if val.trim().is_empty() {
        return Err(ValidationError::new("empty"));
    }

    Ok(())
}
