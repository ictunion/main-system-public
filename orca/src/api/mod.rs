use rocket::response::{self, Responder};
use rocket::serde::{json::Json, Serialize};
use rocket::{Build, Request, Rocket, State};
use tokio::task::JoinError;
use validator::ValidationError;

mod applications;
mod files;
mod registration;
mod session;
mod stats;

use crate::db::DbPool;
use crate::processing::SenderError;
use crate::server::keycloak::{self, Keycloak};

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

        error!("SQL Error: {:?}", self);

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
        error!("Threading Error: {:?}", self);

        Err(rocket::http::Status::InternalServerError)
    }
}

impl<'r> response::Responder<'r, 'static> for SenderError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        error!("Sender Error: {:?}", self);

        Err(rocket::http::Status::InternalServerError)
    }
}

impl<'r> Responder<'r, 'static> for keycloak::Error {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use keycloak::Error::*;
        use rocket::http::Status;

        warn!("JWT verification failed with {:?}", self);

        match self {
            Disabled => Err(Status::NotAcceptable),
            BadKey(_) => Err(Status::InternalServerError),
            MissingRole(_) => Err(Status::Forbidden),
            MissingOneOfRoles(_) => Err(Status::Forbidden),
            BadToken(jwt_err) => {
                use jsonwebtoken::errors::ErrorKind;

                match jwt_err.kind() {
                    ErrorKind::ExpiredSignature => Err(Status::Unauthorized),
                    _ => Err(Status::Forbidden),
                }
            }
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
    InvalidToken(keycloak::Error),
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

impl From<keycloak::Error> for ApiError {
    fn from(err: keycloak::Error) -> Self {
        Self::InvalidToken(err)
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse<'a> {
    http_status: u32,
    http_message: &'a str,
    authorization_connected: bool,
    database_connected: bool,
}

#[get("/status")]
async fn status_api<'a>(
    db_pool: &State<DbPool>,
    keycloak: &State<Keycloak>,
) -> Json<StatusResponse<'a>> {
    let authorization_connected = keycloak.is_connected();

    let res: Result<(bool,), sqlx::Error> = sqlx::query_as("SELECT true")
        .fetch_one(db_pool.inner())
        .await;
    let database_connected = res.is_ok();

    Json(StatusResponse {
        http_status: 200,
        http_message: "ok",
        authorization_connected,
        database_connected,
    })
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .mount("/applications", applications::routes())
        .mount("/session", session::routes())
        .mount("/stats", stats::routes())
        .mount("/files", files::routes())
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

#[derive(Debug, Serialize)]
pub struct OkResponse<'a> {
    status: u32,
    message: &'a str,
}

impl<'r> Responder<'r, 'static> for SuccessResponse {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        use rocket::response::status;

        match &self {
            Self::Accepted => status::Accepted(Some(Json(OkResponse {
                status: 202,
                message: "Accepted",
            })))
            .respond_to(request),
        }
    }
}

pub fn validate_non_empty(val: &str) -> Result<(), ValidationError> {
    if val.trim().is_empty() {
        return Err(ValidationError::new("empty"));
    }

    Ok(())
}
