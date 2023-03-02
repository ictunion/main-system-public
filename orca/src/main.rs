#[macro_use]
extern crate rocket;

use std::fs;

mod api;
mod data;
mod db;
mod generate;
mod media;
mod processing;
mod server;

#[derive(Debug)]
enum StartupError {
    DatabaseError(db::SqlError),
    ServerError(rocket::Error),
}

impl From<db::SqlError> for StartupError {
    fn from(err: db::SqlError) -> Self {
        Self::DatabaseError(err)
    }
}

impl From<rocket::Error> for StartupError {
    fn from(err: rocket::Error) -> Self {
        Self::ServerError(err)
    }
}

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    // create temp directory for orca
    let _dir = fs::create_dir_all("/tmp/orca");

    let db_pool = db::connect(db::Config {
        connection_url: "postgres://orca@localhost/ictunion",
        max_connections: 5,
    })
    .await?;

    let queue_db_pool = db::connect(db::Config {
        connection_url: "postgres://orca@localhost/ictunion",
        max_connections: 2,
    })
    .await?;

    let queue = processing::start(64, queue_db_pool);

    let _rocket = api::build().manage(db_pool).manage(queue).launch().await?;

    Ok(())
}
