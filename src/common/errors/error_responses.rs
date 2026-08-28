use std::fmt::Debug;

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    pub error: String,
}

/// Bad Request
pub fn response_400(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse { error: message.to_string() })
}

/// Unauthorized
pub fn response_401() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse { error: "You are not authorized.".to_string() })
}

/// Forbidden
pub fn response_403(message: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(ErrorResponse { error: message.to_string() })
}

/// Not Found
pub fn response_404(message: impl ToString) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResponse { error: message.to_string() })
}

/// Internal Server Error
pub fn response_500<T: Debug>(e: T) -> HttpResponse {
    event!(target: "backend", Level::ERROR, "{:?}", e);
    HttpResponse::InternalServerError()
        .json(ErrorResponse { error: "Some unexpected error happened. Please try again later.".to_string() })
}
