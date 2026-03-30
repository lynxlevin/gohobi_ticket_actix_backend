use actix_web::{
    delete,
    web::{Data, ReqData},
    HttpResponse,
};
use common::errors::error_responses::{response_401, response_500};
use db_adapters::web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery};
use entities::users_user;
use sea_orm::DbConn;

use crate::use_cases::delete::delete_web_push_subscription;

#[delete("")]
pub async fn delete_web_push_subscription_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<users_user::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_web_push_subscription(
                user.into_inner(),
                WebPushSubscriptionQuery::init_query(&db),
                WebPushSubscriptionMutation { db: &db },
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(_) => response_500(),
            }
        }
        None => response_401(),
    }
}
