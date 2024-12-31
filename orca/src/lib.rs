mod api;
pub mod config;
mod data;
mod db;
mod generate;
mod logging;
mod media;
mod processing;
mod server;
mod validation;

use config::Config;
use server::oid::Provider;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("SQL error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Server error: {0}")]
    Server(#[from] rocket::Error),
    #[error("Logger error: {0}")]
    Logger(#[from] fern::InitError),
    #[error("Keycloak error: {0}")]
    Keycloak(#[from] server::oid::Error),
}

pub async fn start(config: Config) -> Result<(), StartupError> {
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

    api::build()
        .attach(server::cors::Cors)
        .manage(web_db_pool)
        .manage(queue)
        .manage(config)
        .manage(provider)
        .launch()
        .await?;
    Ok(())
}
