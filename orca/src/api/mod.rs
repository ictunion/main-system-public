use rocket::{Build, Rocket, Request};
use rocket::response;

mod registration;

#[derive(Debug)]
pub struct SqlError(sqlx::Error);

impl From<sqlx::Error> for SqlError {
    fn from(err: sqlx::Error) -> Self {
        Self(err)
    }
}

impl<'r> response::Responder<'r, 'static> for SqlError {
    fn respond_to(self, _request: &'r Request<'_>) -> response::Result<'static> {
        use sqlx::error::Error::*;
        use rocket::http::Status;

        match self.0 {
            RowNotFound => Err(Status::NotFound),
            PoolTimedOut => Err(Status::ServiceUnavailable),
            _ => Err(Status::InternalServerError),
        }
    }
}

#[derive(Debug, Responder)]
pub enum ApiError {
    DbErr(SqlError),
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self::DbErr(err.into())
    }
}

#[get("/status")]
fn status_api() -> &'static str {
    "OK"
}

pub fn build() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status_api])
        .mount("/registration", registration::routes())
        .register(
            "/registration",
            catchers![rocket_validation::validation_catcher],
        )
}

pub type Response<T> = Result<T, ApiError>;
