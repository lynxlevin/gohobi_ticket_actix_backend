use common::{
    errors::use_case_errors::UseCaseError,
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use db_adapters::web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery};
use domain_services::{
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
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    match params.r#type {
        MessageType::MakeWish => {
            handle_make_wish_case(
                user,
                wish_service,
                web_push_subscription_query,
                web_push_subscription_mutation,
                settings,
            )
            .await
        }
        MessageType::WishReply => {
            handle_wish_reply_case(
                user,
                wish_reply_service,
                web_push_subscription_query,
                web_push_subscription_mutation,
                settings,
            )
            .await
        }
    }
}

async fn handle_make_wish_case(
    user: users_user::Model,
    wish_service: WishService<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let (wish, ticket) = wish_service.get_newest(user.id).await.map_err(|e| match e {
        WishServiceError::WishNotFound() => UseCaseError::NotFound,
        _ => UseCaseError::InternalServerError,
    })?;

    let web_push_subscription = match web_push_subscription_query.get_by_user_id(user.id).await {
        Ok(sub) => sub,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };
    match web_push_subscription {
        Some(sub) => {
            match send_web_push(
                Message {
                    title: match ticket.is_special {
                        true => Some("⭐️(テスト送信)からの特別なおねがい⭐️".to_string()),
                        false => Some("(テスト送信)からのおねがい".to_string()),
                    },
                    body: wish.description,
                    message_type: MessageType::MakeWish,
                    user_relation_id: Some(wish.user_relation_id),
                    wish_id: Some(wish.id),
                    wish_reply_id: None,
                },
                &sub,
                settings,
            )
            .await
            {
                SendWebPushResult::Invalid => {
                    let _ = web_push_subscription_mutation.delete(sub).await;
                }
                _ => (),
            }
        }
        None => return Err(UseCaseError::NotFound),
    };

    Ok(())
}

async fn handle_wish_reply_case(
    user: users_user::Model,
    wish_reply_service: WishReplyService<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let (wish_reply, wish) = wish_reply_service.get_newest(user.id).await.map_err(|e| match e {
        WishReplyServiceError::WishReplyNotFound() => UseCaseError::NotFound,
        _ => UseCaseError::InternalServerError,
    })?;

    let web_push_subscription = match web_push_subscription_query.get_by_user_id(user.id).await {
        Ok(sub) => sub,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };
    match web_push_subscription {
        Some(sub) => {
            match send_web_push(
                Message {
                    title: Some("(テスト送信)からの返事".to_string()),
                    body: wish_reply.description,
                    message_type: MessageType::WishReply,
                    user_relation_id: Some(wish.user_relation_id),
                    wish_id: Some(wish.id),
                    wish_reply_id: Some(wish_reply.id),
                },
                &sub,
                settings,
            )
            .await
            {
                SendWebPushResult::Invalid => {
                    let _ = web_push_subscription_mutation.delete(sub).await;
                }
                _ => (),
            }
        }
        None => return Err(UseCaseError::NotFound),
    };

    Ok(())
}
