use common::{
    errors::use_case_errors::UseCaseError,
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use db_adapters::{
    ticket::WishQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;

use crate::types::SendWebPushRequest;

pub async fn send_web_push_use_case(
    user: users_user::Model,
    wish_query: WishQuery<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
    params: SendWebPushRequest,
) -> Result<(), UseCaseError> {
    match params.r#type {
        MessageType::UseTicket => {
            handle_use_ticket_case(
                user,
                wish_query,
                web_push_subscription_query,
                web_push_subscription_mutation,
                settings,
            )
            .await
        }
    }
}

async fn handle_use_ticket_case(
    user: users_user::Model,
    wish_query: WishQuery<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
) -> Result<(), UseCaseError> {
    let (wish, ticket) = match wish_query
        .join_user_relation()
        .join_ticket()
        .filter_which_user_has_access(user.id)
        .get_random_with_ticket()
        .await
    {
        Ok(wish) => match wish {
            Some((wish, ticket)) => (wish, ticket),
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    let ticket = match ticket {
        Some(ticket) => ticket,
        None => unreachable!("Wish.ticket_id is required."),
    };

    let web_push_subscription = match web_push_subscription_query.get_by_user_id(user.id).await {
        Ok(sub) => sub,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };
    match web_push_subscription {
        Some(sub) => {
            match send_web_push(
                Message {
                    title: match ticket.is_special {
                        true => Some(format!("⭐️{}からの特別なおねがい⭐️", user.username)),
                        false => Some(format!("{}からのおねがい", user.username)),
                    },
                    body: wish.description,
                    message_type: MessageType::UseTicket,
                    user_relation_id: Some(wish.user_relation_id),
                    ticket_id: None,
                    wish_id: Some(wish.id),
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
