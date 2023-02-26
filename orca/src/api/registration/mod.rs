use std::net::SocketAddr;
use time::Date;

use rocket::response::{status, Redirect};
use rocket::serde::{json::Json, Deserialize};
use rocket::{Route, State};
use rocket_validation::{Validate, Validated};

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
) -> status::Accepted<&'static str> {
    let user = validated_user.0;
    loop {
        let confirmation_token = generate::string(64);
        let res = query::create_join_request(remote_addr, user_agent, confirmation_token, &user)
            .execute(db_pool.inner())
            .await;

        if db::fail_duplicated(&res) {
            continue;
        } else {
            break;
        }
    }
    status::Accepted(Some("ok"))
}

#[get("/<code>/confirm")]
async fn api_confirm(db_pool: &State<DbPool>, code: &'_ str) -> Redirect {
    query::confirm_email(code)
        .fetch_one(db_pool.inner())
        .await
        .unwrap();

    // let local = ret.0.unwrap_or("en".to_string());

    // Redirect::temporary(format!(
    //     "{}/{}/{}",
    //     "https://ictunion.cz", local, "registered"
    // ))

    Redirect::found("https://union.planning-game.com")
}

pub fn routes() -> Vec<Route> {
    routes![api_join, api_confirm]
}
