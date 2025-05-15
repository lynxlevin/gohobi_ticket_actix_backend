use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    pub error: String,
}

pub fn response_400(message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
        error: message.to_string(),
    })
}

pub fn response_401() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse {
        error: "You are not authorized.".to_string(),
    })
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
