use domain_services::ticket::{TicketService, TicketServiceError, TicketServiceMutation};
use entities::{tickets_ticket::TicketId, users_user};
use thiserror::Error;

use crate::TicketVisible;

#[derive(Debug, Error)]
pub enum ReadTicketError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for ReadTicketError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::NotReceivingTicket() => ReadTicketError::Forbidden(e.to_string()),
            TicketServiceError::TicketNotFound() => ReadTicketError::NotFound(e.to_string()),
            _ => ReadTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn read_ticket(
    user: users_user::Model,
    ticket_id: TicketId,
    ticket_service: TicketService<'_>,
) -> Result<TicketVisible, ReadTicketError> {
    let ticket = ticket_service.mark_ticket_read(user.id, ticket_id).await?;

    Ok(TicketVisible::from(ticket))
}
