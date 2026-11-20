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
use domain_services::{web_push::WebPushService, wish::WishService, wish_reply::WishReplyService};
use entities::users_user;

use crate::{types::WebPushSendRequest, use_cases::send_web_push::send_web_push_use_case};

#[tracing::instrument(skip(
    db, settings, user
))]
#[post("/send/")]
async fn send_web_push_endpoint(
    db: Data<Db>,
    settings: Data<Settings>,
    req: Json<WebPushSendRequest>,
    user: Option<ReqData<users_user::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match send_web_push_use_case(
                user.into_inner(),
                req.into_inner(),
                WishService::init(&db),
                WishReplyService::init(&db),
                WebPushService::init(&db),
                &settings,
            )
            .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => match e {
                    UseCaseError::NotFound => response_404("Web push subscription not found."),
                    _ => response_500(e),
                },
            }
        }
        None => response_401(),
    }
}
