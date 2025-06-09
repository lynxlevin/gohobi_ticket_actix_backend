use crate::slack_adapter;
use chrono::Utc;
use common::{errors::use_case_errors::UseCaseError, settings::types::Settings};
use db_adapters::{
    ticket::{types::UpdateTicketParams, TicketMutation, TicketQuery},
    user_relation::UserRelationQuery,
};
use entities::users_user;

use crate::{TicketVisible, UseTicketParams};

pub async fn use_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
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
