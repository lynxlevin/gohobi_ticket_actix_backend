use crate::{AvailableTicketsOldest, AvailableTicketsResponse};
use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    ticket_service::{TicketService, TicketServiceQuery},
    user_relation::UserRelationQuery,
};
use entities::users_user;
use ticket::TicketVisible;

pub async fn available_tickets(
    user: users_user::Model,
    user_relation_id: i64,
    user_relation_query: UserRelationQuery<'_>,
    ticket_service: TicketService<'_>,
) -> Result<AvailableTicketsResponse, UseCaseError> {
    user_relation_query
        .find_by_id(user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    let normal_ticket = ticket_service
        .get_oldest_available_ticket(user.id, user_relation_id, false)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;
    let special_ticket = ticket_service
        .get_oldest_available_ticket(user.id, user_relation_id, true)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    Ok(AvailableTicketsResponse {
        oldest: AvailableTicketsOldest {
            normal: normal_ticket.and_then(|ticket| Some(TicketVisible::from(ticket))),
            special: special_ticket.and_then(|ticket| Some(TicketVisible::from(ticket))),
        },
    })
}
