use std::time::Instant;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use log::info;

use crate::config::LogLevel;

pub fn setup_logger(level: LogLevel) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ));
        })
        .level(level.into())
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

/// Logs method, path, status and duration of every request.
/// The query string is left out on purpose: `/files` carries the token there.
pub async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = Instant::now();

    let response = next.run(request).await;

    info!(
        "{method} {path} => {} ({} ms)",
        response.status().as_u16(),
        started.elapsed().as_millis()
    );
    response
}
