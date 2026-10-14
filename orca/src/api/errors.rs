use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use validator::ValidationErrors;

/// Error response `{"error": {"code", "reason", "description"}}` for a status
pub(crate) fn error_response(status: StatusCode) -> Response {
    let (reason, description) = own_text(status).unwrap_or_else(|| orca_status_text(status));

    let body = json!({
        "error": {
            "code": status.as_u16(),
            "reason": reason,
            "description": description,
        }
    });

    (status, Json(body)).into_response()
}

/// Validation failure response; `errors` is `null` when the body or path
/// could not be deserialized at all
pub(crate) fn validation_error_response(errors: Option<&ValidationErrors>) -> Response {
    let body = json!({
        "code": StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
        "message": "Unprocessable Entity. The request was well-formed but was unable to be followed \
                  due to semantic errors.",
        "errors": errors,
    });

    (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
}

/// Wording of the error catchers Orca registered under Rocket
fn own_text(status: StatusCode) -> Option<(&'static str, &'static str)> {
    match status.as_u16() {
        401 => Some(("Unauthorized", "The request requires user authentication.")),
        403 => Some((
            "Forbidden",
            "This request requires permissions not granted to you.",
        )),
        404 => Some(("NotFound", "The requested resource could not be found.")),
        500 => Some(("InternalError", "Something went wrong.")),
        _ => None,
    }
}

fn orca_status_text(status: StatusCode) -> (&'static str, &'static str) {
    ORCA_ERROR_TEXTS
        .iter()
        .find(|(code, _, _)| *code == status.as_u16())
        .map_or(
            ("Unknown Error", "An unknown error has occurred."),
            |(_, reason, description)| (*reason, *description),
        )
}

#[rustfmt::skip]
const ORCA_ERROR_TEXTS: &[(u16, &str, &str)] = &[
    (400, "Bad Request", "The request could not be understood by the server due to malformed syntax."),
    (402, "Payment Required", "The request could not be processed due to lack of payment."),
    (405, "Method Not Allowed", "The request method is not supported for the requested resource."),
    (406, "Not Acceptable", "The requested resource is capable of generating only content not acceptable according to the Accept headers sent in the request."),
    (407, "Proxy Authentication Required", "Authentication with the proxy is required."),
    (408, "Request Timeout", "The server timed out waiting for the request."),
    (409, "Conflict", "The request could not be processed because of a conflict in the request."),
    (410, "Gone", "The resource requested is no longer available and will not be available again."),
    (411, "Length Required", "The request did not specify the length of its content, which is required by the requested resource."),
    (412, "Precondition Failed", "The server does not meet one of the preconditions specified in the request."),
    (413, "Payload Too Large", "The request is larger than the server is willing or able to process."),
    (414, "URI Too Long", "The URI provided was too long for the server to process."),
    (415, "Unsupported Media Type", "The request entity has a media type which the server or resource does not support."),
    (416, "Range Not Satisfiable", "The portion of the requested file cannot be supplied by the server."),
    (417, "Expectation Failed", "The server cannot meet the requirements of the Expect request-header field."),
    (418, "I'm a teapot", "I was requested to brew coffee, and I am a teapot."),
    (421, "Misdirected Request", "The server cannot produce a response for this request."),
    (422, "Unprocessable Entity", "The request was well-formed but was unable to be followed due to semantic errors."),
    (426, "Upgrade Required", "Switching to the protocol in the Upgrade header field is required."),
    (428, "Precondition Required", "The server requires the request to be conditional."),
    (429, "Too Many Requests", "Too many requests have been received recently."),
    (431, "Request Header Fields Too Large", "The server is unwilling to process the request because either an individual header field, or all the header fields collectively, are too large."),
    (451, "Unavailable For Legal Reasons", "The requested resource is unavailable due to a legal demand to deny access to this resource."),
    (501, "Not Implemented", "The server either does not recognize the request method, or it lacks the ability to fulfill the request."),
    (503, "Service Unavailable", "The server is currently unavailable."),
    (504, "Gateway Timeout", "The server did not receive a timely response from an upstream server."),
    (510, "Not Extended", "Further extensions to the request are required for the server to fulfill it."),
];
