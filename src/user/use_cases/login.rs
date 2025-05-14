use crate::{
    db_adapters::{UserMutation, UserQuery},
    password_util::{self, PasswordType},
    types::LoginRequest,
};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Connection, Pool,
};
use entities::users_user;
use general::{errors::use_case_errors::UseCaseError, settings::types::Settings};

pub async fn login_user(
    redis_pool: &Pool,
    req: LoginRequest,
    user_query: UserQuery<'_>,
    user_mutation: UserMutation<'_>,
    settings: &Settings,
) -> Result<users_user::Model, UseCaseError> {
    let redis_conn = redis_pool.get().await;
    let ref mut redis_conn = match redis_conn {
        Ok(conn) => conn,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    let login_attempts_count_key = format!("gt_login_count_{}", &req.email);
    let login_attempts_count =
        match validate_request_count(redis_conn, &login_attempts_count_key, settings).await {
            Ok(count) => count,
            Err(e) => return Err(e),
        };

    let user = match user_query.find_active_by_email(req.email.clone()).await {
        Ok(user) => match user {
            Some(user) => user,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    match password_util::verify(&req.password, &user.password) {
        Ok(password_type) => match password_type {
            PasswordType::Django => match user_mutation
                .convert_to_argon2_password(user.clone(), &req.password)
                .await
            {
                Ok(user) => Ok(user),
                Err(_) => Ok(user),
            },
            _ => Ok(user),
        },
        Err(_) => {
            increment_login_attempts_count(
                redis_conn,
                login_attempts_count_key,
                login_attempts_count,
            )
            .await;
            return Err(UseCaseError::NotFound);
        }
    }
}

async fn validate_request_count(
    redis_conn: &mut Connection,
    login_attempts_count_key: &str,
    settings: &Settings,
) -> Result<u32, UseCaseError> {
    let login_attempts_count = redis_conn
        .get(login_attempts_count_key.clone())
        .await
        .unwrap_or(0);
    match login_attempts_count >= settings.application.max_login_attempts {
        true => Err(UseCaseError::Unauthorized),
        false => Ok(login_attempts_count),
    }
}

async fn increment_login_attempts_count(
    redis_conn: &mut Connection,
    login_attempts_count_key: String,
    login_attempts_count: u32,
) -> () {
    redis_conn
        .set_options::<String, u32, String>(
            login_attempts_count_key,
            login_attempts_count + 1,
            SetOptions::default().with_expiration(SetExpiry::EX(3600)),
        )
        .await;
}
