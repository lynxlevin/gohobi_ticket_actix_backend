use crate::{constants::NOT_AUTHORIZED_MESSAGE, types::UserVisible, use_cases::get_me::get_me};
use actix_web::{get, web::ReqData, HttpResponse};
use common::errors::{
    error_responses::{response_401, response_500},
    use_case_errors::UseCaseError,
};
use entities::users_user;

#[get("/me")]
async fn get_me_endpoint(user: Option<ReqData<users_user::Model>>) -> HttpResponse {
    match get_me(match user {
        Some(user) => Some(user.into_inner()),
        None => None,
    }) {
        Ok(user) => HttpResponse::Ok().json(UserVisible::from(user)),
        Err(e) => match e {
            UseCaseError::Unauthorized => response_401(NOT_AUTHORIZED_MESSAGE),
            _ => response_500(),
        },
    }
}
