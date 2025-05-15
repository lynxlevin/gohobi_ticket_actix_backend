use actix_web::{post, HttpResponse};

#[post("/logout")]
pub async fn logout_endpoint(session: actix_session::Session) -> HttpResponse {
    session.purge();
    HttpResponse::Ok().finish()
}
