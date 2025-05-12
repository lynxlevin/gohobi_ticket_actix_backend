use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    pub error: String,
}

pub fn response_404(message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(ErrorResponse {
        error: message.to_string(),
    })
}

pub fn response_500() -> HttpResponse {
    HttpResponse::InternalServerError().json(ErrorResponse {
        error: "Some unexpected error happened. Please try again later.".to_string(),
    })
}
