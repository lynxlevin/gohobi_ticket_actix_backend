use crate::{
    constants::{NOT_FOUND_MESSAGE, USER_EMAIL_KEY, USER_ID_KEY},
    db_adapters::{UserMutation, UserQuery},
    types::{LoginRequest, UserVisible},
    use_cases::login::login_user,
};
use actix_session::SessionInsertError;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::Pool;
use general::errors::{
    error_responses::{response_404, response_500},
    use_case_errors::UseCaseError,
};
use sea_orm::DbConn;

#[post("/login")]
async fn login_user_endpoint(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req_user: Json<LoginRequest>,
    session: actix_session::Session,
) -> HttpResponse {
    let user_query = UserQuery { db: &db };
    let user_mutation = UserMutation { db: &db };
    match login_user(
        &redis_pool,
        req_user.into_inner(),
        user_query,
        user_mutation,
    )
    .await
    {
        Ok(user) => match renew_session(session, user.id, user.email.clone()) {
            Ok(_) => HttpResponse::Ok().json(UserVisible {
                id: user.id,
                email: user.email,
                username: user.username,
            }),
            Err(_) => response_500(),
        },
        Err(error) => match error {
            UseCaseError::NotFound => response_404(NOT_FOUND_MESSAGE),
            UseCaseError::InternalServerError => response_500(),
        },
    }
}

fn renew_session(
    session: actix_session::Session,
    id: i64,
    email: String,
) -> Result<(), SessionInsertError> {
    session.renew();
    session.insert(USER_ID_KEY, id)?;
    session.insert(USER_EMAIL_KEY, email)
}
