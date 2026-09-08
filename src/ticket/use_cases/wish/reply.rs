use crate::types::{WebPushResult, WishReplyResponse};
use common::{
    db::Db,
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use domain_services::{
    web_push_subscription::{
        WebPushSubscriptionService, WebPushSubscriptionServiceMutation, WebPushSubscriptionServiceQuery,
    },
    wish_reply::{CreateWishReplyParams, WishReplyService, WishReplyServiceError, WishReplyServiceMutation},
};
use entities::users_user;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WishReplyError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishReplyServiceError> for WishReplyError {
    fn from(e: WishReplyServiceError) -> Self {
        match e {
            WishReplyServiceError::WishNotFound() | WishReplyServiceError::UserRelationNotFound() => {
                WishReplyError::NotFound(e.to_string())
            }
            _ => WishReplyError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn reply(
    user: users_user::Model,
    wish_id: Uuid,
    description: String,
    db: &Db,
    settings: &Settings,
) -> Result<WishReplyResponse, WishReplyError> {
    let wish_reply_service = WishReplyService::init(db);
    let web_push_subscription_service = WebPushSubscriptionService::init(db);

    let (wish_reply, wish, user_relation) = wish_reply_service
        .create_wish_reply(user.id, CreateWishReplyParams { wish_id, description })
        .await?;

    let related_user_id = match user.id == user_relation.user_1_id {
        true => user_relation.user_2_id,
        false => user_relation.user_1_id,
    };
    let web_push_subscription = match web_push_subscription_service.get_opt_by_user_id(related_user_id).await {
        Ok(sub) => sub,
        Err(_) => None,
    };
    let web_push_result = match web_push_subscription {
        Some(sub) => {
            let result = send_web_push(
                Message {
                    title: Some(format!("{}からの返事", user.username)),
                    body: wish_reply.description,
                    message_type: MessageType::WishReply,
                    user_relation_id: Some(user_relation.id),
                    ticket_id: Some(wish.ticket_id),
                    wish_id: Some(wish.id),
                },
                &sub,
                settings,
            )
            .await;
            match result {
                SendWebPushResult::Sent => WebPushResult::Sent,
                SendWebPushResult::Invalid => {
                    let _ = web_push_subscription_service.delete(sub).await;
                    WebPushResult::NotSent
                }
                _ => WebPushResult::NotSent,
            }
        }
        None => WebPushResult::NotSent,
    };

    Ok(WishReplyResponse { web_push_result })
}
