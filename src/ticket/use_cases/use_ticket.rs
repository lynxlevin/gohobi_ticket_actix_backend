use crate::slack_adapter;
use chrono::Utc;
use common::{
    errors::use_case_errors::UseCaseError,
    settings::types::Settings,
    web_push::{Message, MessageType, WebPushMessenger, WebPushMessengerResult},
};
use db_adapters::{
    ticket::{types::UpdateTicketParams, TicketMutation, TicketQuery},
    user_relation::UserRelationQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;

use crate::{TicketVisible, UseTicketParams};

pub async fn use_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    ticket_id: i64,
    params: UseTicketParams,
    settings: &Settings,
) -> Result<TicketVisible, UseCaseError> {
    let ticket = ticket_query
        .filter_which_user_has_access(user.id)
        .exclude_draft_tickets()
        .get_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id == user.id {
        return Err(UseCaseError::Forbidden);
    };

    let user_relation = user_relation_query
        .find_by_id_with_user_name(ticket.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if user_relation.use_slack {
        let message = slack_adapter::get_message(&ticket, &user_relation, &params.use_description);
        slack_adapter::send_slack_message(&message, &settings).await?;
    }

    let related_user_id = match user.id == user_relation.user_1_id {
        true => user_relation.user_2_id,
        false => user_relation.user_1_id,
    };
    let title = match ticket.is_special {
        true => Some(format!("⭐️{}からの特別なおねがい⭐️", user.username)),
        false => Some(format!("{}からのおねがい", user.username)),
    };

    let web_push_result = send_web_push(
        Message {
            title,
            body: params.use_description.clone(),
            message_type: MessageType::UseTicket,
            user_relation_id: Some(user_relation.id),
            ticket_id: Some(ticket.id),
        },
        related_user_id,
        web_push_subscription_query,
        web_push_subscription_mutation,
        settings,
    )
    .await;

    // MYMEMO: It may be nice to let the user know if the other user has web_push on or not. Send info if related user does not have web_push_subscription

    ticket_mutation
        .update(
            ticket,
            UpdateTicketParams {
                use_description: Some(params.use_description),
                use_date: Some(Utc::now().date_naive()),
                ..Default::default()
            },
        )
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|_| UseCaseError::InternalServerError)
}

// MYMEMO: Think about moving these to common.
pub enum SendWebPushResult {
    Sent,
    NotSent,
}

async fn send_web_push(
    message: Message,
    user_id: i64,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    settings: &Settings,
) -> SendWebPushResult {
    let web_push_subscription = match web_push_subscription_query.get_by_user_id(user_id).await {
        Ok(sub) => match sub {
            Some(sub) => sub,
            None => return SendWebPushResult::NotSent,
        },
        Err(_) => return SendWebPushResult::NotSent,
    };
    let messenger = match WebPushMessenger::new(&web_push_subscription, settings) {
        Ok(messenger) => messenger,
        Err(_) => return SendWebPushResult::NotSent,
    };

    match messenger.send_message(message).await {
        Ok(result) => match result {
            WebPushMessengerResult::OK => SendWebPushResult::Sent,
            WebPushMessengerResult::InvalidSubscription => {
                // NOTE: iOS returns 201 even when it's unsubscribed.
                _ = web_push_subscription_mutation
                    .delete(web_push_subscription)
                    .await;
                return SendWebPushResult::NotSent;
            }
        },
        Err(_) => SendWebPushResult::NotSent,
    }
}
