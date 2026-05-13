use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{types::UpdateTicketParams, TicketMutation, TicketQuery};
use entities::{custom_types::TicketStatus, users_user};

use crate::TicketVisible;

pub async fn read_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
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

    ticket_mutation
        .update(
            ticket,
            UpdateTicketParams {
                status: Some(TicketStatus::Read),
                ..Default::default()
            },
        )
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|_| UseCaseError::InternalServerError)
}
