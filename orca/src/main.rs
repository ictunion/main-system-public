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

use orca::config::Config;
use orca::{start, StartupError};

#[rocket::main]
async fn main() -> Result<(), StartupError> {
    // Read cofiguration
    let config = Config::get();
    start(config).await?;
    Ok(())
}
