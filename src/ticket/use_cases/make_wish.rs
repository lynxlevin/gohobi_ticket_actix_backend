use crate::types::{MakeWishResponse, WebPushResult};
use common::{
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use db_adapters::web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery};
use domain_services::wish::{CreateWishResponse, WishService, WishServiceError, WishServiceMutation};
use entities::{tickets_ticket::TicketId, users_user};
use thiserror::Error;

use crate::{MakeWishParams, TicketVisible};

#[derive(Debug, Error)]
pub enum MakeWishError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<WishServiceError> for MakeWishError {
    fn from(e: WishServiceError) -> Self {
        match e {
            WishServiceError::TicketNotFound() => MakeWishError::NotFound("Ticket not found.".to_string()),
            WishServiceError::GivingTicket() => {
                MakeWishError::Forbidden("You cannot make wish with a ticket you gave.".to_string())
            }
            _ => MakeWishError::InternalServerError(e.to_string()),
        }
    }
}

#[tracing::instrument(
    fields(
        user.id = user.id.to_string(),
        ticket_id = ticket_id.to_string(),
        params.use_description.len = params.use_description.len(),
    ),
    skip_all
)]
pub async fn make_wish(
    user: users_user::Model,
    wish_service: WishService<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    ticket_id: TicketId,
    params: MakeWishParams,
    settings: &Settings,
) -> Result<MakeWishResponse, MakeWishError> {
    let CreateWishResponse { wish, ticket, user_relation } =
        wish_service.create(user.id, ticket_id, params.use_description).await?;

    let related_user_id = match user.id == user_relation.user_1_id {
        true => user_relation.user_2_id,
        false => user_relation.user_1_id,
    };
    let web_push_subscription = match web_push_subscription_query.get_by_user_id(related_user_id).await {
        Ok(sub) => sub,
        Err(_) => None,
    };
    let web_push_result = match web_push_subscription {
        Some(sub) => {
            let result = send_web_push(
                Message {
                    title: match ticket.is_special {
                        true => Some(format!("⭐️{}からの特別なおねがい⭐️", user.username)),
                        false => Some(format!("{}からのおねがい", user.username)),
                    },
                    body: wish.description.clone(),
                    message_type: MessageType::MakeWish,
                    user_relation_id: Some(user_relation.id),
                    wish_id: Some(wish.id),
                    wish_reply_id: None,
                },
                &sub,
                settings,
            )
            .await;
            match result {
                SendWebPushResult::Sent => WebPushResult::Sent,
                SendWebPushResult::Invalid => {
                    let _ = web_push_subscription_mutation.delete(sub).await;
                    WebPushResult::NotSent
                }
                _ => WebPushResult::NotSent,
            }
        }
        None => WebPushResult::NotSent,
    };

    Ok(MakeWishResponse { ticket: TicketVisible::from(ticket).with_wish(&wish), web_push_result })
}
