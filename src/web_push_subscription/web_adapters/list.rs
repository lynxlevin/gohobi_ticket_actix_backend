use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use db_adapters::web_push_subscription::WebPushSubscriptionQuery;
use entities::users_user;
use sea_orm::DbConn;

use crate::use_cases::list::list_web_push_subscription;

#[get("")]
pub async fn list_web_push_subscription_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_web_push_subscription(user.into_inner(), WebPushSubscriptionQuery::init_query(&db)).await {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
