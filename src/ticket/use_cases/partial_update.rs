use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{types::UpdateTicketParams, TicketMutation, TicketQuery};
use entities::{custom_types::TicketStatus, users_user};

use crate::TicketVisible;

const PUBLISHED_STATUSES: [TicketStatus; 3] = [
    TicketStatus::Unread,
    TicketStatus::Read,
    TicketStatus::Edited,
];

pub async fn partial_update_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
    params: &mut UpdateTicketParams,
) -> Result<TicketVisible, UseCaseError> {
    let ticket = ticket_query
        .filter_which_user_has_access(user.id)
        .get_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id != user.id {
        return Err(UseCaseError::Forbidden);
    };

    if params
        .status
        .clone()
        .is_some_and(|status| status == TicketStatus::Draft)
        && PUBLISHED_STATUSES.contains(&(&ticket.status).into())
    {
        return Err(UseCaseError::Forbidden);
    };

    if params.description.is_some() && ticket.status == TicketStatus::Read.to_value() {
        params.status = Some(TicketStatus::Edited);
    }

    ticket_mutation
        .update(ticket, params.clone())
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|_| UseCaseError::InternalServerError)
}
