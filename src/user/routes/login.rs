use crate::{
    constants::{NOT_FOUND_MESSAGE, USER_EMAIL_KEY, USER_ID_KEY},
    models::{UserMutation, UserQuery},
    password_util::{self, PasswordType},
    types::{LoginRequest, UserVisible},
};
use actix_session::SessionInsertError;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Connection, Pool,
};
use entities::users_user;
use general::error_responses::{response_404, response_500};
use sea_orm::DbConn;

#[post("/login")]
async fn login_user(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req_user: Json<LoginRequest>,
    session: actix_session::Session,
) -> HttpResponse {
    match UserQuery::find_active_by_email(&db, req_user.email.clone()).await {
        Ok(user) => match user {
            Some(user) => match password_util::verify(&req_user.password, &user.password) {
                Ok(password_type) => match password_type {
                    PasswordType::Django => {
                        let user = convert_to_argon2_password(&db, user, &req_user.password).await;
                        match renew_session(session, user.id, user.email.clone()) {
                            Ok(_) => HttpResponse::Ok().json(UserVisible {
                                id: user.id,
                                email: user.email,
                                username: user.username,
                            }),
                            Err(_) => response_500(),
                        }
                    }
                    _ => match renew_session(session, user.id, user.email.clone()) {
                        Ok(_) => HttpResponse::Ok().json(UserVisible {
                            id: user.id,
                            email: user.email,
                            username: user.username,
                        }),
                        Err(_) => response_500(),
                    },
                },
                Err(_) => response_404(NOT_FOUND_MESSAGE),
            },
            None => response_404(NOT_FOUND_MESSAGE),
        },
        Err(_) => response_500(),
    }
}

async fn convert_to_argon2_password(
    db: &DbConn,
    user: users_user::Model,
    password: &str,
) -> users_user::Model {
    match UserMutation::convert_to_argon2_password(db, user.clone(), password).await {
        Ok(user) => user,
        Err(_) => user,
    }
}

// async fn validate_request_count(
//     redis_con: &mut Connection,
//     email: &str,
// ) -> Result<(String, i32), String> {
//     let max_login_request_count = 5;
//     let login_request_count_key = format!("login_count_{}", email);
//     let login_request_count = redis_con.get(login_request_count_key.clone()).await.map_err(|e| {
//         tracing::event!(target: "backend", tracing::Level::WARN, "Error getting login_request_count, defaults to 0: {}", e);
//     }).unwrap_or(0);
//     if login_request_count >= max_login_request_count {
//         Err("Too many login requests".to_string())
//     } else {
//         Ok((login_request_count_key, login_request_count))
//     }
// }

fn renew_session(
    session: actix_session::Session,
    id: i64,
    email: String,
) -> Result<(), SessionInsertError> {
    session.renew();
    session.insert(USER_ID_KEY, id)?;
    session.insert(USER_EMAIL_KEY, email)
}

// async fn increment_login_request_count(
//     redis_con: &mut Connection,
//     login_request_count_key: String,
//     login_request_count: i32,
// ) -> () {
//     if let Err(e) = redis_con
//         .set_options::<String, i32, String>(
//             login_request_count_key,
//             login_request_count + 1,
//             SetOptions::default().with_expiration(SetExpiry::EX(3600)),
//         )
//         .await
//     {
//         tracing::event!(target: "redis", tracing::Level::WARN, "Error adding login_request_count_key to Redis: {:#?}", e)
//     };
// }
