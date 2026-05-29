use crate::{
    slack_adapter,
    types::{MakeWishResponse, WebPushResult},
};
use common::{
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use db_adapters::{
    ticket::{types::CreateWishParams, WishMutation},
    ticket_service::{TicketService, TicketServiceError, TicketServiceQuery},
    user_relation::UserRelationQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::{custom_types::TicketStatus, users_user};
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
impl From<TicketServiceError> for MakeWishError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::TicketNotFound(_) => MakeWishError::NotFound(e.to_string()),
            _ => MakeWishError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn make_wish(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_service: TicketService<'_>,
    wish_mutation: WishMutation<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    ticket_id: i64,
    params: MakeWishParams,
    settings: &Settings,
) -> Result<MakeWishResponse, MakeWishError> {
    let ticket = ticket_service.get_ticket_by_id(user.id, ticket_id).await?;

    if ticket.status == TicketStatus::Draft.to_value() {
        return Err(MakeWishError::NotFound(format!(
            "Ticket not found for id: {}",
            ticket_id
        )));
    }
    if ticket.giving_user_id == user.id {
        return Err(MakeWishError::Forbidden(
            "You cannot delete a ticket you gave.".to_string(),
        ));
    };

    let user_relation = user_relation_query
        .find_by_id_with_user_name(ticket.user_relation_id, user.id)
        .await
        .map_err(|e| MakeWishError::InternalServerError(e.to_string()))?
        .ok_or(MakeWishError::InternalServerError(format!(
            "UserRelation for ticket_id: {} not found. This should not happen.",
            ticket_id
        )))?;

    if user_relation.use_slack {
        // TODO: When dropping Slack feature, user_relation can be retrieved alongside with ticket to reduce query.
        let message = slack_adapter::get_message(&ticket, &user_relation, &params.use_description);
        slack_adapter::send_slack_message(&message, &settings)
            .await
            .map_err(|e| MakeWishError::InternalServerError(e))?;
    }

    let wish = match wish_mutation
        .create(CreateWishParams {
            use_description: params.use_description.clone(),
            ticket_id: ticket.id,
            user_relation_id: user_relation.id,
        })
        .await
    {
        Ok(wish) => wish,
        Err(e) => return Err(MakeWishError::InternalServerError(e.to_string())),
    };

    let related_user_id = match user.id == user_relation.user_1_id {
        true => user_relation.user_2_id,
        false => user_relation.user_1_id,
    };
    let web_push_subscription = match web_push_subscription_query
        .get_by_user_id(related_user_id)
        .await
    {
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
                    body: params.use_description,
                    message_type: MessageType::MakeWish,
                    user_relation_id: Some(user_relation.id),
                    ticket_id: None,
                    wish_id: Some(wish.id),
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

    Ok(MakeWishResponse {
        ticket: TicketVisible::from(ticket).with_wish(&wish),
        web_push_result,
    })
}
