mod api;
pub mod config;
mod data;
mod db;
mod generate;
mod listmonk;
mod logging;
mod media;
mod processing;
mod server;
mod validation;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::ServiceExt;
use axum::extract::Request;
use config::Config;
use log::{error, info};
use server::oid::Provider;
use thiserror::Error;
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePath;

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("SQL error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Server error: {0}")]
    Server(#[from] std::io::Error),
    #[error("Logger error: {0}")]
    Logger(#[from] fern::InitError),
    #[error("Keycloak error: {0}")]
    Keycloak(#[from] server::oid::Error),
}

/// Start serving the Orca API
///
/// # Errors
///
/// When the initialization fails
pub async fn start(config: Config) -> Result<(), StartupError> {
    // Configure logger
    logging::setup_logger(config.log_level)?;

    let provider = Arc::new(Provider::init(&config).await?);

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

    let queue = processing::start(&config, processing_db_pool, &provider);

    // This will not work, until we figure out how to provide JWT Token for Keycloak calls
    processing::ensure_member_subs(&web_db_pool, &queue).await;

    let address = SocketAddr::new(config.address, config.port);

    let app = NormalizePath::trim_trailing_slash(api::router(api::AppState {
        db: web_db_pool,
        queue,
        config: Arc::new(config),
        provider,
    }));

    let listener = TcpListener::bind(address).await?;
    info!("Listening on http://{address}");

    axum::serve(
        listener,
        ServiceExt::<Request>::into_make_service_with_connect_info::<SocketAddr>(app),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Resolves on Ctrl+C or SIGTERM
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!("Failed to listen for Ctrl+C: {err}");
            std::future::pending::<()>().await;
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                error!("Failed to listen for SIGTERM: {err}");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutting down");
}
