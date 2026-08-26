use actix_web::{post, HttpResponse};

#[tracing::instrument(skip(session))]
#[post("/logout")]
pub async fn logout_endpoint(session: actix_session::Session) -> HttpResponse {
    session.purge();
    HttpResponse::Ok().finish()
}
