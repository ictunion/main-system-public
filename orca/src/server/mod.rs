use std::convert::Infallible;
use std::net::IpAddr;

cfg_if::cfg_if! {
    if #[cfg(feature="proxy-support")] {
        use std::str::FromStr;
    }
}

use rocket::request::{FromRequest, Outcome, Request};

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserAgent<'a>(Option<&'a str>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent<'r> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key = request.headers().get_one("user-agent");
        Outcome::Success(UserAgent(key))
    }
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct IpAddress(IpAddr);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for IpAddress {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        cfg_if::cfg_if! {
            // Trust x-real-ip header if present
            if #[cfg(feature="proxy-support")] {
                let header_val: Option<IpAddress> = request
                    .headers()
                    .get_one("x-real-ip")
                    .and_then(|val| IpAddr::from_str(val).ok())
                    .map(IpAddress);

                match header_val {
                    Some(val) => Outcome::Success(val),
                    None => {
                        let outcome = IpAddr::from_request(request).await;
                        outcome.map(IpAddress)
                    }
                }
            } else {
                let outcome = IpAddr::from_request(request).await;
                outcome.map(IpAddress)
            }
        }
    }
}
