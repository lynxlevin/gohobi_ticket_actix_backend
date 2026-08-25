use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use common::{
    db::Db,
    errors::{
        error_responses::{response_401, response_404, response_500},
        use_case_errors::UseCaseError,
    },
    settings::types::Settings,
};
use db_adapters::{
    ticket::WishQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;

use crate::{types::SendWebPushRequest, use_cases::send_web_push::send_web_push_use_case};

#[post("/send/")]
async fn send_web_push_endpoint(
    db: Data<Db>,
    settings: Data<Settings>,
    user: Option<ReqData<users_user::Model>>,
    params: Json<SendWebPushRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match send_web_push_use_case(
                user.into_inner(),
                WishQuery::init_query(&db),
                WebPushSubscriptionQuery::init_query(&db),
                WebPushSubscriptionMutation { db: &db.db },
                &settings,
                params.into_inner(),
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("Web push subscription not found."),
                    _ => response_500(),
                },
            }
        }
        None => response_401(),
    }
}
