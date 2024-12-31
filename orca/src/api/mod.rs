use log::{error, warn};
use rocket::response::{self, Responder};
use rocket::serde::{json::Json, Serialize};
use rocket::{catchers, get, routes, Build, Request, Rocket, State};
use thiserror::Error;
use tokio::task::JoinError;
use validator::ValidationError;

mod applications;
mod errors;
mod files;
mod members;
mod registration;
mod session;
mod stats;
mod workplaces;

use crate::api::errors::validation_error;
use crate::db::{self, DbPool};
use crate::processing::SenderError;
use crate::server::oid::{self, Provider};

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

        if db::is_conflict(&self.0) {
            return Err(Status::Conflict);
        }

        match self.0 {
            RowNotFound => Err(Status::NotFound),
            PoolTimedOut => Err(Status::ServiceUnavailable),
            _ => Err(Status::InternalServerError),
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ThreadingError(#[from] JoinError);

impl<'r> Responder<'r, 'static> for ThreadingError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        error!("Threading Error: {:?}", self);

        Err(rocket::http::Status::InternalServerError)
    }
}

impl<'r> Responder<'r, 'static> for SenderError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        error!("Sender Error: {:?}", self);

        Err(rocket::http::Status::InternalServerError)
    }
}

impl<'r> Responder<'r, 'static> for oid::Error {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use oid::Error::*;
        use rocket::http::Status;

        warn!("JWT verification failed with {:?}", self);

        match self {
            Disabled => Err(Status::NotFound),
            BadKey(_) => Err(Status::InternalServerError),
            MissingRole(_) => Err(Status::Forbidden),
            MissingRealmRole(_) => Err(Status::Forbidden),
            MissingOneOfRoles(_) => Err(Status::Forbidden),
            BadToken(_) => Err(Status::Unauthorized),
            Http(_) => Err(Status::BadGateway),
            Parsing(_) => Err(Status::InternalServerError),
            Proxy(status_code) => Err(Status {
                code: status_code.as_u16(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct CustomError {
    status: rocket::http::Status,
    json: rocket::serde::json::Value,
}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use rocket::http::ContentType;
        use rocket::response::Response;
        use std::io::Cursor;

        let body = self.json.to_string();

        Response::build()
            .header(ContentType::JSON)
            .status(self.status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

#[derive(Debug, Responder)]
pub enum ApiError {
    DbErr(SqlError),
    Status(rocket::http::Status),
    #[response(status = 500)]
    ThreadFail(ThreadingError),
    #[response(status = 500)]
    QueueSender(Box<SenderError>),
    #[response(status = 401)]
    InvalidToken(Box<oid::Error>),
    Custom(CustomError),
}

impl ApiError {
    pub fn data_conflict(description: String) -> ApiError {
        use rocket::http::Status;
        use rocket::serde::json::json;

        let custom_error = CustomError {
            status: Status::Conflict,
            json: json!(
            {
                "error": {
                    "code": 409,
                    "reason": "Conflict",
                    "description": description
                }
            }
            ),
        };

        Self::Custom(custom_error)
    }
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
        Self::QueueSender(Box::new(err))
    }
}

impl From<oid::Error> for ApiError {
    fn from(err: oid::Error) -> Self {
        Self::InvalidToken(Box::new(err))
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse<'a> {
    http_status: u32,
    http_message: &'a str,
    authorization_connected: bool,
    database_connected: bool,
    proxy_support_enabled: bool,
}

#[get("/status")]
async fn status_api<'a>(
    db_pool: &State<DbPool>,
    oid_provider: &State<Provider>,
) -> Json<StatusResponse<'a>> {
    let authorization_connected = oid_provider.is_connected();

    let res: Result<(bool,), sqlx::Error> = sqlx::query_as("SELECT true")
        .fetch_one(db_pool.inner())
        .await;
    let database_connected = res.is_ok();

    // inline bool for proxy support
    cfg_if::cfg_if! {
        if #[cfg(feature="proxy-support")] {
            let proxy_support_enabled = true;
        } else {
            let proxy_support_enabled = false;
        }
    }

    Json(StatusResponse {
        http_status: 200,
        http_message: "ok",
        authorization_connected,
        database_connected,
        proxy_support_enabled,
    })
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .register("/registration", catchers![validation_error])
        .mount("/applications", applications::routes())
        .register("/applications", errors::catchers())
        .mount("/session", session::routes())
        .register("/session", errors::catchers())
        .mount("/stats", stats::routes())
        .register("/stats", errors::catchers())
        .mount("/members", members::routes())
        .register("/members", errors::catchers())
        .mount("/workplaces", workplaces::routes())
        .register("/workplaces", errors::catchers())
        // Files use default catchers
        .mount("/files", files::routes())
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
