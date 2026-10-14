use std::sync::Arc;

use axum::extract::{DefaultBodyLimit, FromRef, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router, middleware};
use log::{error, warn};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use tokio::task::JoinError;
use validator::ValidationError;

pub(crate) mod applications;
pub(crate) mod errors;
pub(crate) mod extract;
mod files;
mod members;
mod oidc;
mod registration;
mod session;
mod stats;
mod sync;
mod workplaces;

use crate::api::errors::error_response;
use crate::config::Config;
use crate::db::{self, DbPool};
use crate::processing::{QueueSender, SenderError};
use crate::server::oid::{self, Provider};
use crate::{logging, server};

/// Shared state of all handlers; each field can be extracted on its own
/// with `State<T>`
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DbPool,
    pub(crate) queue: QueueSender,
    pub(crate) config: Arc<Config>,
    pub(crate) provider: Arc<Provider>,
}

impl FromRef<AppState> for DbPool {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for QueueSender {
    fn from_ref(input: &AppState) -> Self {
        input.queue.clone()
    }
}

impl FromRef<AppState> for Arc<Config> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.config)
    }
}

impl FromRef<AppState> for Arc<Provider> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.provider)
    }
}

#[derive(Debug)]
pub struct SqlError(sqlx::Error);

impl From<sqlx::Error> for SqlError {
    fn from(err: sqlx::Error) -> Self {
        Self(err)
    }
}

impl IntoResponse for SqlError {
    fn into_response(self) -> axum::response::Response {
        use sqlx::error::Error::{PoolTimedOut, RowNotFound};

        error!("SQL Error: {self:?}");

        if db::is_conflict(&self.0) {
            return error_response(StatusCode::CONFLICT);
        }

        match self.0 {
            RowNotFound => error_response(StatusCode::NOT_FOUND),
            PoolTimedOut => error_response(StatusCode::SERVICE_UNAVAILABLE),
            _ => error_response(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ThreadingError(#[from] JoinError);

impl IntoResponse for ThreadingError {
    fn into_response(self) -> axum::response::Response {
        error!("Threading Error: {self:?}");

        error_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for SenderError {
    fn into_response(self) -> axum::response::Response {
        error!("Sender Error: {self:?}");

        error_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for oid::Error {
    fn into_response(self) -> axum::response::Response {
        use oid::Error::{
            BadKey, BadToken, Disabled, Http, MissingOneOfRoles, MissingRealmRole, MissingRole,
            Parsing, Proxy,
        };

        warn!("JWT verification failed with {self:?}");

        let status = match self {
            Disabled => StatusCode::NOT_FOUND,
            BadKey(_) | Parsing(_) => StatusCode::INTERNAL_SERVER_ERROR,
            MissingRole(_) | MissingRealmRole(_) | MissingOneOfRoles(_) => StatusCode::FORBIDDEN,
            BadToken(_) => StatusCode::UNAUTHORIZED,
            Http(_) => StatusCode::BAD_GATEWAY,
            Proxy(status_code) => StatusCode::from_u16(status_code.as_u16())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        };

        error_response(status)
    }
}

#[derive(Debug)]
pub struct CustomError {
    status: StatusCode,
    json: serde_json::Value,
}

impl IntoResponse for CustomError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(self.json)).into_response()
    }
}

#[derive(Debug)]
pub enum ApiError {
    DbErr(SqlError),
    Status(StatusCode),
    ThreadFail(ThreadingError),
    QueueSender(Box<SenderError>),
    InvalidToken(Box<oid::Error>),
    Custom(CustomError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::DbErr(err) => err.into_response(),
            Self::Status(status) => error_response(status),
            Self::ThreadFail(err) => err.into_response(),
            Self::QueueSender(err) => err.into_response(),
            Self::InvalidToken(err) => err.into_response(),
            Self::Custom(err) => err.into_response(),
        }
    }
}

impl ApiError {
    pub fn data_conflict(description: &str) -> ApiError {
        let custom_error = CustomError {
            status: StatusCode::CONFLICT,
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

    pub fn invalid_group_id() -> ApiError {
        let custom_error = CustomError {
            status: StatusCode::BAD_REQUEST,
            json: json!(
            {
                "error": {
                    "code": 400,
                    "reason": "Bad Request",
                    "description": "Group ID needs to be valid UUID"
                }
            }
            ),
        };

        Self::Custom(custom_error)
    }

    pub fn oidc_group_not_found() -> ApiError {
        let custom_error = CustomError {
            status: StatusCode::BAD_REQUEST,
            json: json!(
            {
                "error": {
                    "code": 400,
                    "reason": "Bad Request",
                    "description": "OIDC group not found"
                }
            }
            ),
        };

        Self::Custom(custom_error)
    }

    pub fn keycloak_push(description: &str) -> ApiError {
        let custom_error = CustomError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            json: json!(
            {
                "error": {
                    "code": 500,
                    "reason": "Could not push user data to Keycloak.",
                    "description": description
                }
            }
            ),
        };

        Self::Custom(custom_error)
    }

    pub fn listmonk_error(description: &str) -> ApiError {
        let custom_error = CustomError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            json: json!(
            {
                "error": {
                    "code": 500,
                    "reason": "Listmonk error",
                    "description": description
                }
            }
            ),
        };

        Self::Custom(custom_error)
    }

    pub fn config_missing(item: &'static str) -> ApiError {
        let description = format!("Missing {item}");

        let custom_error = CustomError {
            status: StatusCode::CONFLICT,
            json: json!(
            {
                "error": {
                    "code": 500,
                    "reason": "Config Missing",
                    "description": description
                }
            }
            ),
        };
        Self::Custom(custom_error)
    }
    pub fn data_missing(item: &'static str) -> ApiError {
        let description = format!("Missing {item}");

        let custom_error = CustomError {
            status: StatusCode::CONFLICT,
            json: json!(
            {
                "error": {
                    "code": 500,
                    "reason": "Data Missing",
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

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        let description = format!("Got {err}");

        let custom_error = CustomError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            json: json!(
            {
                "error": {
                    "code": 503,
                    "reason": "Third party API call Failed",
                    "description": description
                }
            }
            ),
        };
        Self::Custom(custom_error)
    }
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
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
struct StatusResponse {
    http_status: u32,
    http_message: &'static str,
    authorization_connected: bool,
    database_connected: bool,
    proxy_support_enabled: bool,
}

async fn status_api(
    State(db_pool): State<DbPool>,
    State(oid_provider): State<Arc<Provider>>,
) -> Json<StatusResponse> {
    let authorization_connected = oid_provider.is_connected();

    let database_connected = sqlx::query_scalar!("SELECT TRUE")
        .fetch_one(&db_pool)
        .await
        .is_ok();

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

async fn not_found() -> axum::response::Response {
    error_response(StatusCode::NOT_FOUND)
}

async fn method_not_allowed() -> axum::response::Response {
    error_response(StatusCode::METHOD_NOT_ALLOWED)
}

pub(crate) fn router(state: AppState) -> Router {
    let json_limit = state.config.json_limit;

    Router::new()
        .route("/status", get(status_api))
        .nest("/registration", registration::router())
        .nest("/applications", applications::router())
        .nest("/session", session::router())
        .nest("/stats", stats::router())
        .nest("/members", members::router())
        .nest("/oidc", oidc::router())
        .nest("/workplaces", workplaces::router())
        .nest("/sync", sync::router())
        .nest("/files", files::router())
        .fallback(not_found)
        // Must come after all routes: it only applies to routes added before it
        .method_not_allowed_fallback(method_not_allowed)
        .layer(DefaultBodyLimit::max(json_limit))
        .layer(server::cors::layer())
        .layer(middleware::from_fn(logging::log_request))
        .with_state(state)
}

pub type Response<T> = Result<T, ApiError>;

/// This is pretty much a wrapper for `StatusCode`
/// which doesn't allow for response body content
/// since it standardizes it for common cases.
#[derive(Debug)]
pub enum SuccessResponse {
    Accepted,
    Created,
    NoContent,
}

#[derive(Debug, Serialize)]
pub struct OkResponse<'a> {
    status: u32,
    message: &'a str,
}

impl IntoResponse for SuccessResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Accepted => (
                StatusCode::ACCEPTED,
                Json(OkResponse {
                    status: 202,
                    message: "Accepted",
                }),
            )
                .into_response(),
            Self::Created => (
                StatusCode::CREATED,
                Json(OkResponse {
                    status: 201,
                    message: "Created",
                }),
            )
                .into_response(),
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

pub fn validate_non_empty(val: &str) -> Result<(), ValidationError> {
    if val.trim().is_empty() {
        return Err(ValidationError::new("empty"));
    }

    Ok(())
}
