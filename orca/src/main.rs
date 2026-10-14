//! We use axum and sqlx so please refer to documentation of these 2 crates
//!
//! ## Client IP
//! The client IP recorded for registrations is taken from the `X-Real-IP`
//! header when present, otherwise from the connection. The header is always
//! trusted, so Orca must only be reachable through a reverse proxy that sets it.
//!
//! ## Features
//! ### Reverse Proxy Support
//! The `proxy-support` feature only reports `proxy_support_enabled` in `/status`;
//! it does not change how the client IP is read.
//! ```
//! cargo build --features proxy-support
//! ```

use orca::config::Config;
use orca::{StartupError, start};

#[tokio::main]
async fn main() -> Result<(), Box<StartupError>> {
    // Read cofiguration
    let config = Config::get();
    start(config).await?;
    Ok(())
}
