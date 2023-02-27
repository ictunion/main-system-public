use rocket::futures::TryFutureExt;
use sqlx::{
    postgres::{PgArguments, PgPoolOptions},
    PgPool, Postgres,
};
use std::ops::Deref;

#[derive(Debug)]
pub struct SqlError(sqlx::Error);

pub struct Config {
    pub connection_url: &'static str,
    pub max_connections: u32,
}

pub async fn connect(config: Config) -> Result<PgPool, SqlError> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(config.connection_url)
        .map_err(SqlError)
        .await
}

const DUPLICATE_KEY: &str = "32505";

pub fn is_err_code(err: &sqlx::Error, code: &str) -> bool {
    match err {
        sqlx::Error::Database(db_error) => match db_error.code() {
            Some(cow) => cow.deref() == code,
            None => false,
        },
        _ => false,
    }
}

pub fn fail_duplicated<T>(res: &Result<T, sqlx::Error>) -> bool {
    match res {
        Ok(_) => false,
        Err(err) => is_err_code(err, DUPLICATE_KEY),
    }
}

pub type DbPool = PgPool;
pub type Query<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;
pub type QueryAs<'q, T> = sqlx::query::QueryAs<'q, Postgres, T, PgArguments>;
