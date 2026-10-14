use axum::Json;
use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::extract::{FromRequest, FromRequestParts, Path, Request};
use axum::http::request::Parts;
use axum::response::Response;
use log::{error, warn};
use serde::de::DeserializeOwned;

use super::errors::{error_response, validation_error_response};

/// JSON request body. Rejections use our JSON error bodies:
/// malformed JSON → 400, wrong types or missing fields → 422,
/// missing `Content-Type` → 415, body over the limit → 413.
#[derive(Debug)]
pub(crate) struct ApiJson<T>(pub(crate) T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => {
                error!("Failed to parse request: {rejection}");
                Err(match rejection {
                    JsonRejection::JsonDataError(_) => validation_error_response(None),
                    other => error_response(other.status()),
                })
            }
        }
    }
}

/// Path parameters. A parameter that fails to parse (e.g. a bad UUID) is a 422
/// with the validation body, as it was under Rocket.
#[derive(Debug)]
pub(crate) struct ApiPath<T>(pub(crate) T);

impl<S, T> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(parts, state).await {
            Ok(Path(value)) => Ok(Self(value)),
            Err(rejection) => {
                warn!("Failed to parse path parameters: {rejection}");
                Err(match rejection {
                    PathRejection::FailedToDeserializePathParams(_) => {
                        validation_error_response(None)
                    }
                    other => error_response(other.status()),
                })
            }
        }
    }
}
