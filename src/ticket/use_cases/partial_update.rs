use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{
    types::{TicketStatus, UpdateTicketParams},
    TicketMutation, TicketQuery,
};
use entities::users_user;

use crate::TicketVisible;

pub async fn partial_update_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
    params: &mut UpdateTicketParams,
) -> Result<TicketVisible, UseCaseError> {
    let ticket = match ticket_query
        .filter_by_user(user.id)
        .get_by_id(ticket_id)
        .await
    {
        Ok(ticket) => match ticket {
            Some(ticket) => ticket,
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    };

    if ticket.giving_user_id != user.id {
        return Err(UseCaseError::Forbidden);
    };

    let published_statuses = vec![
        TicketStatus::Unread,
        TicketStatus::Read,
        TicketStatus::Edited,
    ];
    if params
        .status
        .clone()
        .is_some_and(|status| status == TicketStatus::Draft)
        && published_statuses.contains(&TicketStatus::from(ticket.status.to_owned()))
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
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })
}
