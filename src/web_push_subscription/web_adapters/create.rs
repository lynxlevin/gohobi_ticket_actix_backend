use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::{
    errors::error_responses::{response_401, response_500},
    settings::types::Settings,
};
use db_adapters::web_push_subscription::WebPushSubscriptionMutation;
use entities::users_user;
use sea_orm::DbConn;

use crate::{types::WebPushSubscriptionCreateRequest, use_cases::create::create_web_push_subscription};

#[post("")]
pub async fn create_web_push_subscription_endpoint(
    db: Data<DbConn>,
    settings: Data<Settings>,
    user: Option<ReqData<users_user::Model>>,
    req: Json<WebPushSubscriptionCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_web_push_subscription(
                user.into_inner(),
                &settings,
                req.into_inner(),
                WebPushSubscriptionMutation { db: &db },
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
