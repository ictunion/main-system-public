#[macro_use]
extern crate rocket;

mod api;
mod config;
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
    // Read cofiguration
    let config = config::Config::get();

    let web_db_pool = db::connect(db::Config {
        connection_url: &config.postgres,
        max_connections: config.web_db_pool,
    })
    .await?;

    let processing_db_pool = db::connect(db::Config {
        connection_url: &config.postgres,
        max_connections: config.processing_db_pool,
    })
    .await?;

    let queue = processing::start(&config, processing_db_pool);

    let _rocket = api::build()
        .manage(web_db_pool)
        .manage(queue)
        .manage(config)
        .launch()
        .await?;

    Ok(())
}
