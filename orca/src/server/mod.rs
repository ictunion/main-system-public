use core::fmt::Display;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

pub mod cors;
pub mod jwk;
pub mod oid;

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::StatusCode;
use axum::http::header::USER_AGENT;
use axum::http::request::Parts;
use log::error;
use serde::Serialize;

use crate::api::ApiError;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserAgent(Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = parts
            .headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        Ok(UserAgent(key))
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct IpAddress(IpAddr);

impl Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.0.fmt(f)
    }
}

impl Serialize for IpAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for IpAddress {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Trust x-real-ip header if present, regardless of the `proxy-support`
        // feature. This is what Rocket did (its `ip_header` defaults to X-Real-IP).
        let header_val: Option<IpAddress> = parts
            .headers
            .get("x-real-ip")
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<IpAddr>().ok())
            .map(IpAddress);

        if let Some(val) = header_val {
            return Ok(val);
        }

        // Present when served with `into_make_service_with_connect_info`
        let Some(ConnectInfo(addr)) = parts.extensions.get::<ConnectInfo<SocketAddr>>() else {
            error!("Client address is missing, is the server started with connect info?");
            return Err(ApiError::Status(StatusCode::INTERNAL_SERVER_ERROR));
        };

        Ok(IpAddress(addr.ip()))
    }
}
