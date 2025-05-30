use crate::{
    db_adapters::{UserMutation, UserQuery},
    password_util::{self, PasswordType},
    redis_adapter::UserRedis,
    types::LoginRequest,
};
use common::errors::use_case_errors::UseCaseError;
use entities::users_user;

pub async fn login_user(
    req: LoginRequest,
    user_query: UserQuery<'_>,
    user_mutation: UserMutation<'_>,
    user_redis: UserRedis<'_>,
) -> Result<users_user::Model, UseCaseError> {
    let login_attempts_count_key = format!("gt_login_count_{}", &req.email);
    let login_attempts_count = user_redis
        .clone()
        .validate_request_count(&login_attempts_count_key)
        .await?;

    let user = user_query
        .find_active_by_email(req.email.clone())
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    match password_util::verify(&req.password, &user.password) {
        Ok(password_type) => match password_type {
            PasswordType::Django => {
                let _ = user_mutation
                    .convert_to_argon2_password(user.clone(), &req.password)
                    .await;
                Ok(user)
            }
            _ => Ok(user),
        },
        Err(_) => {
            user_redis
                .increment_login_attempts_count(&login_attempts_count_key, login_attempts_count)
                .await;
            return Err(UseCaseError::NotFound);
        }
    }
}
