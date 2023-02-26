use std::net::SocketAddr;
use time::Date;

use rocket::response::{status, Redirect};
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use rocket_validation::{Validate, Validated};

use super::Response;
use crate::db::{self, DbPool};
use crate::generate;
use crate::server::UserAgent;

mod query;

#[derive(Debug, Deserialize, Validate)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationRequest<'r> {
    #[validate(email)]
    email: &'r str,
    #[validate(length(min = 1))]
    first_name: &'r str,
    #[validate(length(min = 1))]
    last_name: &'r str,
    // We use option here so that `null` value goes
    // through the parser into validator
    #[validate(required)]
    date_of_birth: Option<Date>,
    address: Option<&'r str>,
    city: Option<&'r str>,
    postal_code: Option<&'r str>,
    phone_number: Option<&'r str>,
    company_name: Option<&'r str>,
    occupation: Option<&'r str>,
    // TODO: add image base 64
    #[validate(length(min = 2, max = 2))]
    local: &'r str,
}

#[post("/join", format = "json", data = "<validated_user>")]
async fn api_join(
    remote_addr: SocketAddr,
    user_agent: UserAgent<'_>,
    db_pool: &State<DbPool>,
    validated_user: Validated<Json<RegistrationRequest<'_>>>,
) -> Response<status::Accepted<&'static str>> {
    let user = validated_user.0;
    loop {
        let confirmation_token = generate::string(64);
        let res = query::create_join_request(remote_addr, user_agent, confirmation_token, &user)
            .execute(db_pool.inner())
            .await;

        if db::fail_duplicated(&res) {
            // We won't the loterry and generated confirmation token
            // which already exists...
            // lets just try a new one
            continue;
        } else {
            match res {
                Err(err) => {
                    // Shoot some issue with DB query...
                    return Err(err.into());
                }
                Ok(_) => {
                    // All went well...
                    // break the loop so we can respond with Ok
                    break;
                }
            }
        }
    }
    Ok(status::Accepted(Some("ok")))
}

#[get("/<code>/confirm")]
async fn api_confirm(db_pool: &State<DbPool>, code: &'_ str) -> Response<Redirect> {
    // let local = ret.0.unwrap_or("en".to_string());

    // Redirect::temporary(format!(
    //     "{}/{}/{}",
    //     "https://ictunion.cz", local, "registered"
    // ))
    let redirect =  Ok(Redirect::found("https://union.planning-game.com"));

    use sqlx::error::Error::*;
    match query::confirm_email(code).fetch_one(db_pool.inner()).await {
        Ok(_) => redirect,
        // Even if not found we want to still redirect!
        // This is especially in cases user goes back to confirmation
        // email and clicks the link again
        Err(RowNotFound) => redirect,
        Err(err) => Err(err.into()),
    }
}

pub fn routes() -> Vec<Route> {
    routes![api_join, api_confirm]
}
