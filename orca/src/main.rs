#[macro_use]
extern crate rocket;

use std::convert::Infallible;
use std::net::SocketAddr;
use std::ops::Deref;

use rand::{distributions::Alphanumeric, Rng};
use rocket::response::status;
use rocket::response::Redirect;
use time::Date;

use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::{json::Json, Deserialize};
use rocket::State;

use sqlx::{postgres::PgPoolOptions, Error, PgPool};

const DUPLICATE_KEY: &str = "32505";

struct UserAgent<'r>(Option<&'r str>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent<'r> {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key = req.headers().get_one("user-agent");
        Outcome::Success(UserAgent(key))
    }
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct RegistrationRequest<'r> {
    email: &'r str,
    first_name: &'r str,
    last_name: &'r str,
    date_of_birth: Date,
    address: &'r str,
    city: &'r str,
    postal_code: &'r str,
    phone_number: &'r str,
    company_name: &'r str,
    occupation: &'r str,
    // TODO: add image base 64
    local: &'r str,
}

fn get_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

#[get("/status")]
fn status_api() -> &'static str {
    "OK"
}

#[get("/confirm/<code>")]
async fn confirm(db_pool: &State<PgPool>, code: &'_ str) -> Redirect {
    let _ret: (Option<String>,) = sqlx::query_as(
        "
update members as m
set confirmed_at = now()
  , confirmation_token = NULL
where
  confirmation_token = $1
returning
  m.registration_local
",
    )
    .bind(code)
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

#[post("/member/join", format = "json", data = "<user>")]
async fn member_join(
    remote_addr: SocketAddr,
    user_agent: UserAgent<'_>,
    db_pool: &State<PgPool>,
    user: Json<RegistrationRequest<'_>>,
) -> status::Accepted<&'static str> {
    loop {
        let confirmation_token = get_random_string(64);
        let res = sqlx::query(
            "
insert into members
( email
, first_name
, last_name
, date_of_birth
, address
, city
, postal_code
, phone_number
, company_name
, occupation
, registration_local
, registration_ip
, registration_user_agent
, registration_source
, confirmation_token
)
values
( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15 )
",
        )
        .bind(user.email)
        .bind(user.first_name)
        .bind(user.last_name)
        .bind(user.date_of_birth)
        .bind(user.address)
        .bind(user.city)
        .bind(user.postal_code)
        .bind(user.phone_number)
        .bind(user.company_name)
        .bind(user.occupation)
        .bind(user.local)
        .bind(remote_addr.ip())
        .bind(user_agent.0)
        .bind("website_join_form")
        .bind(confirmation_token)
        .execute(db_pool.inner())
        .await;

        match res {
            Err(err) => {
                if matches_code(&err, DUPLICATE_KEY) {
                    continue;
                } else {
                    break;
                }
            }
            Ok(_) => break,
        }
    }
    status::Accepted(Some("ok"))
}

fn matches_code(err: &sqlx::Error, code: &str) -> bool {
    match err {
        Error::Database(db_error) => match db_error.code() {
            Some(cow) => cow.deref() == code,
            None => false,
        },
        _ => false,
    }
}

#[rocket::main]
async fn main() {
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://orca@localhost/ictunion")
        .await
        .unwrap();

    let _ = rocket::build()
        .mount("/", routes![status_api, member_join, confirm,])
        .manage(db_pool)
        .launch()
        .await
        .unwrap();
}
