use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{
    types::{TicketStatus, UpdateTicketParams},
    TicketMutation, TicketQuery,
};
use entities::users_user;

use crate::TicketVisible;

pub async fn read_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
) -> Result<TicketVisible, UseCaseError> {
    let ticket = match ticket_query
        .filter_which_user_has_access(user.id)
        .get_by_id(ticket_id)
        .await
    {
        Ok(ticket) => match ticket {
            Some(ticket) => ticket,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    if ticket.giving_user_id == user.id {
        return Err(UseCaseError::Forbidden);
    };

    if ticket.status == TicketStatus::Draft.to_value() {
        return Err(UseCaseError::NotFound);
    };

    ticket_mutation
        .update(
            ticket,
            UpdateTicketParams {
                description: None,
                status: Some(TicketStatus::Read),
            },
        )
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })
}
