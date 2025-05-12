use crate::{
    models::{UserMutation, UserQuery},
    password_util::{self, PasswordType},
    types::LoginRequest,
};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Connection, Pool,
};
use entities::users_user;
use general::errors::use_case_errors::UseCaseError;
use sea_orm::DbConn;

pub async fn login_user(
    db: &DbConn,
    redis_pool: &Pool,
    req: LoginRequest,
) -> Result<users_user::Model, UseCaseError> {
    // let conn = match redis_pool.get().await {
    //     Ok(ref mut conn) => conn,
    //     Err(_) => return Err(UseCaseError::InternalServerError),
    // };
    let user = match UserQuery::find_active_by_email(&db, req.email.clone()).await {
        Ok(user) => match user {
            Some(user) => user,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match password_util::verify(&req.password, &user.password) {
        Ok(password_type) => match password_type {
            PasswordType::Django => Ok(convert_to_argon2_password(db, user, &req.password).await),
            _ => Ok(user),
        },
        Err(_) => Err(UseCaseError::NotFound),
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
