use rocket::serde::json::Value;
use rocket::Catcher;

#[derive(Debug)]
enum ErrorType {
    InternalError,
    NotFound,
    Unauthorized,
    Forbidden,
}

impl ErrorType {
    fn to_status_code(&self) -> u16 {
        match self {
            Self::InternalError => 500,
            Self::NotFound => 404,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
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

pub fn catchers() -> Vec<Catcher> {
    catchers![
        internal_error,
        not_found_error,
        unauthorized_error,
        forbidden_error
    ]
}
