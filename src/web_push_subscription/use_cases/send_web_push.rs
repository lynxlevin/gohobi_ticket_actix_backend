use common::{errors::use_case_errors::UseCaseError, settings::types::Settings};
use domain_services::{
    web_push::{Message, MessageType, WebPushService, WebPushServiceError},
    wish::{WishService, WishServiceError, WishServiceQuery},
    wish_reply::{WishReplyService, WishReplyServiceError, WishReplyServiceQuery},
};
use entities::users_user;

use crate::types::WebPushSendRequest;

pub async fn send_web_push_use_case(
    user: users_user::Model,
    params: WebPushSendRequest,
    wish_service: WishService<'_>,
    wish_reply_service: WishReplyService<'_>,
    web_push_service: WebPushService<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    match params.r#type {
        MessageType::MakeWish => handle_make_wish_case(user, wish_service, web_push_service, settings).await,
        MessageType::WishReply => {
            handle_wish_reply_case(user, wish_reply_service, web_push_service, settings).await
        }
    }
}

async fn handle_make_wish_case(
    user: users_user::Model,
    wish_service: WishService<'_>,
    web_push_service: WebPushService<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let (wish, ticket) = wish_service.get_newest(user.id).await.map_err(|e| match e {
        WishServiceError::WishNotFound() => UseCaseError::NotFound,
        _ => UseCaseError::InternalServerError,
    })?;

    let message = Message {
        title: match ticket.is_special {
            true => Some("⭐️(テスト送信)からの特別なおねがい⭐️".to_string()),
            false => Some("(テスト送信)からのおねがい".to_string()),
        },
        body: wish.description,
        message_type: MessageType::MakeWish,
        user_relation_id: Some(wish.user_relation_id),
        wish_id: Some(wish.id),
        wish_reply_id: None,
    };
    web_push_service
        .send_message(message, user.id, settings)
        .await
        .map_err(|e| match e {
            WebPushServiceError::NotFound() => UseCaseError::NotFound,
            WebPushServiceError::DbErr(_) => UseCaseError::InternalServerError,
        })?;

    Ok(())
}

async fn handle_wish_reply_case(
    user: users_user::Model,
    wish_reply_service: WishReplyService<'_>,
    web_push_service: WebPushService<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let (wish_reply, wish) = wish_reply_service.get_newest(user.id).await.map_err(|e| match e {
        WishReplyServiceError::WishReplyNotFound() => UseCaseError::NotFound,
        _ => UseCaseError::InternalServerError,
    })?;

    let message = Message {
        title: Some("(テスト送信)からの返事".to_string()),
        body: wish_reply.description,
        message_type: MessageType::WishReply,
        user_relation_id: Some(wish.user_relation_id),
        wish_id: Some(wish.id),
        wish_reply_id: Some(wish_reply.id),
    };
    web_push_service
        .send_message(message, user.id, settings)
        .await
        .map_err(|e| match e {
            WebPushServiceError::NotFound() => UseCaseError::NotFound,
            WebPushServiceError::DbErr(_) => UseCaseError::InternalServerError,
        })?;

    Ok(())
}
