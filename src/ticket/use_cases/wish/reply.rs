use crate::types::WishReplyResponse;
use common::{
    settings::types::Settings,
    web_push::{Message, MessageType},
};
use domain_services::{
    web_push::{WebPushResult, WebPushService},
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

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        wish_id = wish_id.to_string(),
        description.len = description.len(),
    ),
    skip_all
)]
pub async fn reply(
    user: users_user::Model,
    wish_id: Uuid,
    description: String,
    wish_reply_service: WishReplyService<'_>,
    web_push_service: WebPushService<'_>,
    settings: &Settings,
) -> Result<WishReplyResponse, WishReplyError> {
    let (wish_reply, wish, user_relation) = wish_reply_service
        .create_wish_reply(user.id, CreateWishReplyParams { wish_id, description })
        .await?;

    let related_user_id = match user.id == user_relation.user_1_id {
        true => user_relation.user_2_id,
        false => user_relation.user_1_id,
    };
    let message = Message {
        title: Some(format!("{}からの返事", user.username)),
        body: wish_reply.description,
        message_type: MessageType::WishReply,
        user_relation_id: Some(user_relation.id),
        wish_id: Some(wish.id),
        wish_reply_id: Some(wish_reply.id),
    };

    let web_push_result = web_push_service
        .send_message(message, related_user_id, settings)
        .await
        .unwrap_or(WebPushResult::NotSent);

    Ok(WishReplyResponse { web_push_result: web_push_result.into() })
}
