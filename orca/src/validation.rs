use log::error;
use rocket::data::FromData;
use rocket::data::Outcome;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{Data, Request};
use std::fmt::Debug;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

pub(crate) type ValidationErrorsCache = Option<ValidationErrors>;
pub(crate) struct Validated<T> {
    inner: T,
}

impl<T> Validated<T> {
    fn new(value: T) -> Self {
        Self { inner: value }
    }

    pub(crate) fn into_inner(self) -> T {
        self.inner
    }
}

#[derive(Debug, Error)]
pub(crate) enum ValidatorError {
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    #[error("Validation error: {0}")]
    ValidationErrors(#[from] ValidationErrors),
}

#[rocket::async_trait]
impl<'r, T: rocket::serde::Deserialize<'r> + Validate> FromData<'r> for Validated<Json<T>> {
    type Error = ValidatorError;

    async fn from_data(request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self, Self::Error> {
        <Json<T> as FromData<'r>>::from_data(request, data)
            .await
            .map_error(|(s, e)| {
                error!("Failed to parse request: {e}");
                (s, Self::Error::Deserialization(e.to_string()))
            })
            .and_then(|value| {
                if let Err(e) = value.validate() {
                    request.local_cache(|| Some(e.to_owned()));
                    return Outcome::Error((
                        Status::UnprocessableEntity,
                        Self::Error::ValidationErrors(e),
                    ));
                };
                Outcome::Success(Self::new(value))
            })
    }
}
