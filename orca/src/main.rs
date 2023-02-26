#[macro_use]
extern crate rocket;

mod api;
mod db;
mod server;
mod generate;

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
    let db_pool = db::connect(db::Config {
        connection_url: "postgres://orca@localhost/ictunion",
        max_connections: 5
    }).await?;

    let _rocket = api::build()
        .manage(db_pool)
        .launch()
        .await?;

    Ok(())
}
