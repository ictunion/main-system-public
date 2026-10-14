use axum::extract::{FromRequest, Request};
use axum::response::Response;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::api::errors::validation_error_response;
use crate::api::extract::ApiJson;

/// JSON request body that passed `validator` checks.
/// Failed validation is a 422 listing the errors per field.
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

impl<S, T> FromRequest<S> for Validated<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let ApiJson(value) = ApiJson::<T>::from_request(req, state).await?;

        if let Err(errors) = value.validate() {
            return Err(validation_error_response(Some(&errors)));
        }

        Ok(Self::new(value))
    }
}
