use common::errors::use_case_errors::UseCaseError;
use entities::users_user;

pub fn get_me(user: Option<users_user::Model>) -> Result<users_user::Model, UseCaseError> {
    match user {
        Some(user) => Ok(user),
        None => Err(UseCaseError::Unauthorized),
    }
}
