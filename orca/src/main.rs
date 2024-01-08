//! We use rocket and sqlx so please refer to documentation of these 2 creates
//!
//! ## Features
//! some configuration is done by compile time configuration
//!
//! ### Reverse Proxy Support
//! If your server runs behind reverse proxy you trust
//! You should enable `proxy-support` feature.
//! This means value from `X-Real-IP` header will be trusted.
//! ```
//! cargo build --features proxy-support
//! ```

extern crate rand;
#[macro_use]
extern crate rocket;
extern crate cfg_if;
extern crate image;
extern crate rocket_validation;
extern crate rustc_serialize;
extern crate sqlx;
extern crate time;
extern crate tokio;
extern crate validator;
#[macro_use]
extern crate log;
extern crate chrono;
extern crate fern;
extern crate handlebars;
extern crate jsonwebtoken;
extern crate phf;
extern crate reqwest;

mod api;
mod config;
mod data;
mod db;
mod generate;
mod logging;
mod media;
mod processing;
mod server;

use server::oid::Provider;

#[derive(Debug)]
enum StartupError {
    Database(db::SqlError),
    Server(rocket::Error),
    Logger(fern::InitError),
    Keyclaok(server::oid::Error),
}

impl From<db::SqlError> for StartupError {
    fn from(err: db::SqlError) -> Self {
        Self::Database(err)
    }
}

impl From<rocket::Error> for StartupError {
    fn from(err: rocket::Error) -> Self {
        Self::Server(err)
    }
}

impl From<fern::InitError> for StartupError {
    fn from(err: fern::InitError) -> Self {
        Self::Logger(err)
    }
}

impl From<server::oid::Error> for StartupError {
    fn from(value: server::oid::Error) -> Self {
        Self::Keyclaok(value)
    }
}

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    // Read cofiguration
    let config = config::Config::get();

    // Configure logger
    logging::setup_logger(config.log_level)?;

    let provider = Provider::init(&config).await?;

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
        .attach(server::cors::Cors)
        .manage(web_db_pool)
        .manage(queue)
        .manage(config)
        .manage(provider)
        .launch()
        .await?;

    Ok(())
}
