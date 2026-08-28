use crate::{
    constants::{NOT_FOUND_MESSAGE, TOO_MANY_LOGIN_ATTEMPTS_MESSAGE, USER_EMAIL_KEY, USER_ID_KEY},
    db_adapters::{UserMutation, UserQuery},
    redis_adapter::UserRedis,
    types::{LoginRequest, UserVisible},
    use_cases::login::login_user,
};
use actix_session::SessionInsertError;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use common::db::Db;
use common::{
    errors::{
        error_responses::{response_400, response_404, response_500},
        use_case_errors::UseCaseError,
    },
    settings::types::Settings,
};
use deadpool_redis::Pool;
use entities::users_user::UserId;

#[tracing::instrument(skip(db, redis_pool, settings, req_user, session))]
#[post("/login")]
async fn login_user_endpoint(
    db: Data<Db>,
    redis_pool: Data<Pool>,
    settings: Data<Settings>,
    req_user: Json<LoginRequest>,
    session: actix_session::Session,
) -> HttpResponse {
    let user_query = UserQuery { db: &db.db };
    let user_mutation = UserMutation { db: &db.db };
    let user_redis = UserRedis { pool: &redis_pool, settings: &settings };

    match login_user(req_user.into_inner(), user_query, user_mutation, user_redis).await {
        Ok(user) => match renew_session(session, user.id, user.email.clone()) {
            Ok(_) => HttpResponse::Ok().json(UserVisible::from(user)),
            Err(_) => response_500(),
        },
        Err(error) => match error {
            UseCaseError::BadRequest => response_400(TOO_MANY_LOGIN_ATTEMPTS_MESSAGE),
            UseCaseError::NotFound => response_404(NOT_FOUND_MESSAGE),
            _ => response_500(),
        },
    }
}

fn renew_session(session: actix_session::Session, id: UserId, email: String) -> Result<(), SessionInsertError> {
    session.renew();
    session.insert(USER_ID_KEY, id)?;
    session.insert(USER_EMAIL_KEY, email)
}
