#[macro_use]
extern crate rocket;

use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use sqlx::{postgres::PgPoolOptions, PgPool};
use time::Date;

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

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/member/join", format = "json", data = "<user>")]
async fn member_join(db_pool: &State<PgPool>, user: Json<RegistrationRequest<'_>>) -> String {
    sqlx::query(
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
)
values
( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11 )
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
    .execute(db_pool.inner())
    .await
    .unwrap();

    "ok".to_string()
}

#[rocket::main]
async fn main() {
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://orca@localhost/ictunion")
        .await
        .unwrap();

    let _ = rocket::build()
        .mount("/", routes![index, member_join])
        .manage(db_pool)
        .launch()
        .await
        .unwrap();
}
