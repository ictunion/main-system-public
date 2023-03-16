use time::Date;

use rocket::http::Status;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use rocket_validation::{Validate, Validated};

use super::{validate_non_empty, Response, SuccessResponse};
use crate::config::Config;
use crate::data::{self, Id};
use crate::db::{self, DbPool};
use crate::generate;
use crate::media::RawBase64;
use crate::processing::Command;
use crate::processing::QueueSender;
use crate::server::{IpAddress, UserAgent};

mod query;

#[derive(Debug, Clone, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
/// Input for join api (website form)
pub struct RegistrationRequest<'r> {
    #[validate(required)]
    #[validate(email)]
    email: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    first_name: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    last_name: Option<&'r str>,
    /// We use option here so that `null` value goes
    /// through the parser into validator
    #[validate(required)]
    date_of_birth: Option<Date>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    address: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    city: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    postal_code: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    phone_number: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    company_name: Option<&'r str>,
    #[validate(required)]
    #[validate(custom = "validate_non_empty")]
    occupation: Option<&'r str>,
    #[validate(required)]
    signature: Option<RawBase64<'r>>,
    #[validate(length(min = 2, max = 2))]
    local: &'r str,
}

#[post("/join", format = "json", data = "<validated_user>")]
/// Registration form request
async fn api_join<'r>(
    ip_addr: IpAddress,
    user_agent: UserAgent<'_>,
    db_pool: &State<DbPool>,
    queue: &State<QueueSender>,
    validated_user: Validated<Json<RegistrationRequest<'r>>>,
) -> Response<SuccessResponse> {
    let user = validated_user.0;

    // First lets create a request record so we don't loose
    // any people even if rest of the stuff goes wrong for one reason or another
    let reg_id: Id<data::RegistrationRequest>;
    let mut confirmation_token;
    loop {
        confirmation_token = generate::string(64);
        let res =
            query::create_join_request(ip_addr, user_agent, confirmation_token.clone(), &user)
                .fetch_one(db_pool.inner())
                .await;

        if db::fail_duplicated(&res) {
            // We won the loterry and generated confirmation token which already exists...
            // lets just try a new one
            continue;
        } else {
            match res {
                Err(err) => {
                    // Oh shoot some issue with DB query...
                    return Err(err.into());
                }
                Ok((id,)) => {
                    // All went well...
                    // break the loop so we can respond with Ok
                    reg_id = id;
                    break;
                }
            }
        }
    }

    // Decode signature image data
    let mut signature_data = user
        .signature
        .as_ref()
        .ok_or(Status::UnprocessableEntity)
        .and_then(|data| {
            data.to_image_data()
                .map_err(|_| Status::UnprocessableEntity)
        })?;

    // This is done in response thread because we want to be sure
    // it succeeds before we return OK status
    signature_data
        .resize(492, 192)
        .map_err(|_| Status::UnprocessableEntity)?;

    query::create_singature_file(reg_id, &signature_data)
        .execute(db_pool.inner())
        .await?;

    // Rest of the processing then happens on async thread outside of web worker
    queue
        .inner()
        .send(Command::NewRegistrationRequest(
            reg_id,
            signature_data,
            confirmation_token,
        ))
        .await?;

    info!(
        "IP: {} New registration request added to processing",
        ip_addr
    );
    Ok(SuccessResponse::Accepted)
}

#[get("/<code>/confirm")]
/// Confirma registration request using email
async fn api_confirm(
    db_pool: &State<DbPool>,
    config: &State<Config>,
    queue: &State<QueueSender>,
    code: &'_ str,
) -> Response<Redirect> {
    use sqlx::error::Error::*;
    match query::confirm_email(code).fetch_one(db_pool.inner()).await {
        Ok((reg_id, local)) => {
            // notify about new registration
            queue
                .inner()
                .send(Command::RegistrationRequestVerified(reg_id))
                .await?;

            info!("Registration request verified.");
            // redirect user to the right place
            Ok(Redirect::found(
                config.inner().verify_redirect_for_local(&local),
            ))
        }
        // Even if not found we want to still redirect!
        // This is especially in cases user goes back to confirmation
        // email and clicks the link again
        Err(RowNotFound) => {
            info!("Verification token not found, redirecting still...");
            Ok(Redirect::found(
                config.inner().verify_redirect_for_local("default"),
            ))
        }
        Err(err) => Err(err.into()),
    }
}

pub fn routes() -> Vec<Route> {
    routes![api_join, api_confirm]
}
