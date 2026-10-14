use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header::LOCATION;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use chrono::NaiveDate;
use log::{info, warn};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

mod query;
use super::{AppState, Response, SuccessResponse, validate_non_empty};
use crate::api::extract::ApiPath;
use crate::config::Config;
use crate::data::{self, Id};
use crate::db::{self, DbPool};
use crate::generate;
use crate::media::RawBase64;
use crate::processing::{Command, QueueSender};
use crate::server::{IpAddress, UserAgent};
use crate::validation::Validated;
#[derive(Debug, Clone, Deserialize, Validate)]
/// Input for join api (website form)
pub struct RegistrationRequest {
    #[validate(required)]
    #[validate(email)]
    email: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    first_name: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    last_name: Option<String>,
    /// We use option here so that `null` value goes
    /// through the parser into validator
    #[validate(required)]
    date_of_birth: Option<NaiveDate>,
    address: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    city: Option<String>,
    postal_code: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    phone_number: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    company_name: Option<String>,
    #[validate(required)]
    #[validate(custom(function = "validate_non_empty"))]
    occupation: Option<String>,
    #[validate(required)]
    signature: Option<RawBase64>,
    #[validate(length(min = 2, max = 2))]
    local: String,
}

/// Registration form request
async fn api_join(
    ip_addr: IpAddress,
    user_agent: UserAgent,
    State(db_pool): State<DbPool>,
    State(queue): State<QueueSender>,
    validated_user: Validated<RegistrationRequest>,
) -> Response<SuccessResponse> {
    let user = validated_user.into_inner();

    // First lets create a request record so we don't loose
    // any people even if rest of the stuff goes wrong for one reason or another
    let reg_id: Id<data::RegistrationRequest>;
    let mut confirmation_token;
    loop {
        confirmation_token = generate::string(64);
        let res = query::create_join_request(
            &db_pool,
            ip_addr,
            &user_agent,
            confirmation_token.clone(),
            &user,
        )
        .await;

        if db::fail_duplicated(&res) {
            // We won the loterry and generated confirmation token which already exists...
            // lets just try a new one
            continue;
        }
        match res {
            Err(err) => {
                // Oh shoot some issue with DB query...
                return Err(err.into());
            }
            Ok(id) => {
                // All went well...
                // break the loop so we can respond with Ok
                reg_id = id;
                break;
            }
        }
    }

    // Decode signature image data
    let mut signature_data = user
        .signature
        .as_ref()
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY) // Can't happen due to validator
        .and_then(|data| {
            data.to_image_data().map_err(|e| {
                warn!("Bad input image data: {e:?}"); // FIXME: Rethink the error output to expose this to API consumer
                StatusCode::UNPROCESSABLE_ENTITY
            })
        })?;

    // This is done in response thread because we want to be sure
    // it succeeds before we return OK status
    signature_data
        .resize(492, 192)
        .map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;

    query::create_signature_file(&db_pool, reg_id, &signature_data).await?;

    // Rest of the processing then happens on async thread outside of web worker
    queue
        .send(Command::NewRegistrationRequest(reg_id, signature_data))
        .await?;

    info!("IP: {ip_addr} New registration request added to processing");
    Ok(SuccessResponse::Accepted)
}

/// `302 Found` redirect; axum's `Redirect` offers only 303, 307 and 308
#[derive(Debug)]
struct Found(String);

impl IntoResponse for Found {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::FOUND, [(LOCATION, self.0)]).into_response()
    }
}

/// Confirma registration request using email
async fn api_confirm(
    State(db_pool): State<DbPool>,
    State(config): State<Arc<Config>>,
    ApiPath(code): ApiPath<String>,
) -> Response<Found> {
    use sqlx::error::Error::RowNotFound;

    match query::confirm_email(&db_pool, &code).await {
        Ok(local) => {
            info!("Registration request verified.");
            // redirect user to the right place
            Ok(Found(config.verify_redirect_for_local(&local)))
        }
        // Even if not found we want to still redirect!
        // This is especially in cases user goes back to confirmation
        // email and clicks the link again
        Err(RowNotFound) => {
            info!("Verification token not found, redirecting still...");
            Ok(Found(config.verify_redirect_for_local("default")))
        }
        Err(err) => Err(err.into()),
    }
}

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/join", post(api_join))
        .route("/{code}/confirm", get(api_confirm))
}
