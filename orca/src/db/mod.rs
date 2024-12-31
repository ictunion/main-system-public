use sqlx::{
    postgres::{PgArguments, PgPoolOptions},
    PgPool, Postgres,
};
use std::ops::Deref;

pub struct Config<'a> {
    pub connection_url: &'a str,
    pub max_connections: u32,
}

pub async fn connect(config: Config<'_>) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(config.connection_url)
        .await
}

/// Reference
/// https://www.postgresql.org/docs/current/errcodes-appendix.html
const UNIQUE_VIOLATION: &str = "23505";

pub fn is_err_code(err: &sqlx::Error, code: &str) -> bool {
    match err {
        sqlx::Error::Database(db_error) => match db_error.code() {
            Some(cow) => cow.deref() == code,
            None => false,
        },
        _ => false,
    }
}

pub fn is_conflict(err: &sqlx::Error) -> bool {
    is_err_code(err, UNIQUE_VIOLATION)
}

pub fn fail_duplicated<T>(res: &Result<T, sqlx::Error>) -> bool {
    match res {
        Ok(_) => false,
        Err(err) => is_conflict(err),
    }
}

pub type DbPool = PgPool;
pub type Query<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;
pub type QueryAs<'q, T> = sqlx::query::QueryAs<'q, Postgres, T, PgArguments>;
