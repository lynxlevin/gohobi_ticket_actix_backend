use db_adapters::ticket_service::{
    TicketService, TicketServiceError, TicketServiceMutation, TicketServiceQuery, UpdateTicketParams,
};
use entities::{
    tickets_ticket::{TicketId, TicketStatus},
    users_user,
};
use thiserror::Error;

use crate::TicketVisible;

#[derive(Debug, Error)]
pub enum PartialUpdateTicketError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for PartialUpdateTicketError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::TicketNotFound(_) => PartialUpdateTicketError::NotFound(e.to_string()),
            _ => PartialUpdateTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn partial_update_ticket(
    user: users_user::Model,
    ticket_service: TicketService<'_>,
    ticket_id: TicketId,
    params: &mut UpdateTicketParams,
) -> Result<TicketVisible, PartialUpdateTicketError> {
    let ticket = ticket_service.get_ticket_by_id(user.id, ticket_id).await?;

    if ticket.giving_user_id != user.id {
        return Err(PartialUpdateTicketError::Forbidden(
            "You cannot update a ticket you received.".to_string(),
        ));
    }
    if params
        .status
        .clone()
        .is_some_and(|status| status == TicketStatus::Draft)
        && ticket.status.is_published()
    {
        return Err(PartialUpdateTicketError::Forbidden(
            "This ticket cannot be turned back to draft state.".to_string(),
        ));
    };
    if params.description.is_some() && ticket.status == TicketStatus::Read {
        params.status = Some(TicketStatus::Edited);
    }

    let ticket = ticket_service.update_ticket(ticket, params.clone()).await?;

    Ok(TicketVisible::from(ticket))
}
