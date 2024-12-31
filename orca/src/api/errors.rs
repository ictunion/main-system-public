use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use rocket::{catch, catchers, Catcher, Request};

use crate::validation::ValidationErrorsCache;

#[derive(Debug)]
enum ErrorType {
    InternalError,
    NotFound,
    Unauthorized,
    Forbidden,
    ValidationFailed,
}

impl ErrorType {
    fn to_status_code(&self) -> u16 {
        match self {
            Self::InternalError => 500,
            Self::NotFound => 404,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::ValidationFailed => 422,
        }
    }
}

fn error_json(error_type: ErrorType, message: &'static str) -> Value {
    use rocket::serde::json::json;

    json!({
        "error": {
            "code": error_type.to_status_code(),
            "reason": format!("{:?}", error_type),
            "description": message,
        }
    })
}

#[catch(500)]
fn internal_error() -> Value {
    let error_type = ErrorType::InternalError;
    let message = "Something went wrong.";
    error_json(error_type, message)
}

#[catch(404)]
fn not_found_error() -> Value {
    let error_type = ErrorType::NotFound;
    let message = "The requested resource could not be found.";
    error_json(error_type, message)
}

#[catch(401)]
fn unauthorized_error() -> Value {
    let error_type = ErrorType::Unauthorized;
    let message = "The request requires user authentication.";
    error_json(error_type, message)
}

#[catch(403)]
fn forbidden_error() -> Value {
    let error_type = ErrorType::Forbidden;
    let message = "This request requires permissions not granted to you.";
    error_json(error_type, message)
}

#[catch(422)]
pub(crate) fn validation_error(request: &Request) -> Value {
    let error_type = ErrorType::ValidationFailed;
    let cached_errors: &ValidationErrorsCache = request.local_cache(|| None);
    json! ({
        "code": error_type.to_status_code(),
        "message": "Unprocessable Entity. The request was well-formed but was unable to be followed \
                  due to semantic errors.",
        "errors": cached_errors
    })
}

pub fn catchers() -> Vec<Catcher> {
    catchers![
        validation_error,
        internal_error,
        not_found_error,
        unauthorized_error,
        forbidden_error
    ]
}
