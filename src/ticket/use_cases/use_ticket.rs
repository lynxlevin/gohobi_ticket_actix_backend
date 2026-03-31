use crate::{
    slack_adapter,
    types::{UseTicketResponse, WebPushResult},
};
use chrono::Utc;
use common::{
    errors::use_case_errors::UseCaseError,
    settings::types::Settings,
    web_push::{send_web_push, Message, MessageType, SendWebPushResult},
};
use db_adapters::{
    ticket::{types::CreateWishParams, TicketQuery, WishMutation},
    user_relation::UserRelationQuery,
    web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery},
};
use entities::users_user;

use crate::{TicketVisible, UseTicketParams};

pub async fn use_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    wish_mutation: WishMutation<'_>,
    web_push_subscription_query: WebPushSubscriptionQuery<'_>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'_>,
    ticket_id: i64,
    params: UseTicketParams,
    settings: &Settings,
) -> Result<UseTicketResponse, UseCaseError> {
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
                    body: params.use_description.clone(),
                    message_type: MessageType::UseTicket,
                    user_relation_id: Some(user_relation.id),
                    ticket_id: Some(ticket.id),
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

    let wish = match wish_mutation
        .create(CreateWishParams {
            use_description: params.use_description,
            use_date: Utc::now().date_naive(),
            ticket_id: ticket.id,
            user_relation_id: user_relation.id,
        })
        .await
    {
        Ok(wish) => wish,
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    Ok(UseTicketResponse {
        ticket: TicketVisible::from(ticket).with_wish(&wish),
        web_push_result,
    })
}
