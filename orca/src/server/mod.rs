use std::convert::Infallible;

use rocket::request::{FromRequest, Outcome, Request};

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserAgent<'a>(Option<&'a str>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserAgent<'r> {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key = req.headers().get_one("user-agent");
        Outcome::Success(UserAgent(key))
    }
}
