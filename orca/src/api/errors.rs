use rocket::serde::{json::Json, Serialize};
use rocket::Catcher;

#[derive(Serialize)]
enum ErrorType {
    InternalError,
    NotFound,
    Unauthorized,
    Forbidden,
}

#[derive(Serialize)]
struct ErrorJson<'a> {
    error_type: ErrorType,
    message: &'a str,
}

impl<'a> ErrorJson<'a> {
    pub fn json(error_type: ErrorType, message: &'a str) -> Json<Self> {
        Json(ErrorJson {
            error_type,
            message,
        })
    }
}

#[catch(500)]
fn internal_error() -> Json<ErrorJson<'static>> {
    let error_type = ErrorType::InternalError;
    let message = "Something went wrong.";
    ErrorJson::json(error_type, message)
}

#[catch(404)]
fn not_found_error() -> Json<ErrorJson<'static>> {
    let error_type = ErrorType::NotFound;
    let message = "The requested resource could not be found.";
    ErrorJson::json(error_type, message)
}

#[catch(401)]
fn unauthorized_error() -> Json<ErrorJson<'static>> {
    let error_type = ErrorType::Unauthorized;
    let message = "The request requires user authentication.";
    ErrorJson::json(error_type, message)
}

#[catch(403)]
fn forbidden_error() -> Json<ErrorJson<'static>> {
    let error_type = ErrorType::Forbidden;
    let message = "This request requires permissions not granted to you.";
    ErrorJson::json(error_type, message)
}

pub fn catchers() -> Vec<Catcher> {
    catchers![
        internal_error,
        not_found_error,
        unauthorized_error,
        forbidden_error
    ]
}
