use rocket::response::{self, Responder};
use rocket::{Build, Request, Rocket};
use tokio::task::JoinError;
use validator::ValidationError;

mod applications;
mod registration;
mod stats;

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

#[derive(Debug)]
pub struct JwtError(jsonwebtoken::errors::Error);

impl<'r> Responder<'r, 'static> for JwtError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use jsonwebtoken::errors::ErrorKind;
        use rocket::http::Status;

        let kind = self.0.kind();

        info!("JWT verification failed with {:?}", kind);
        match kind {
            ErrorKind::ExpiredSignature => Err(Status::Forbidden),
            _ => Err(Status::Unauthorized),
        }
    }
}

#[derive(Debug, Responder)]
pub enum ApiError {
    #[response(status = 500)]
    DbErr(SqlError),
    Status(rocket::http::Status),
    #[response(status = 500)]
    ThreadFail(ThreadingError),
    #[response(status = 500)]
    QueueSender(SenderError),
    #[response(status = 401)]
    InvalidToken(JwtError),
    #[response(status = 403)]
    MissingRole(String),
    #[response(status = 401)]
    AuthorizationDisabled(()),
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

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::InvalidToken(JwtError(err))
    }
}

#[get("/status")]
fn status_api() -> SuccessResponse {
    SuccessResponse::Ok
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .mount("/applications", applications::routes())
        .mount("/stats", stats::routes())
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
    Ok,
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
            Self::Ok => Json(" { 'status: 200, 'message': 'OK' }").respond_to(request),
        }
    }
}

pub fn validate_non_empty(val: &str) -> Result<(), ValidationError> {
    if val.trim().is_empty() {
        return Err(ValidationError::new("empty"));
    }

    Ok(())
}
