use db_adapters::ticket_service::{TicketService, TicketServiceError, TicketServiceMutation, TicketServiceQuery};
use entities::{custom_types::TicketStatus, users_user};
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
            TicketServiceError::TicketNotFound(_) => ReadTicketError::NotFound(e.to_string()),
            _ => ReadTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn read_ticket(
    user: users_user::Model,
    ticket_id: i64,
    ticket_service: TicketService<'_>,
) -> Result<TicketVisible, ReadTicketError> {
    let ticket = ticket_service.get_ticket_by_id(user.id, ticket_id).await?;

    if ticket.giving_user_id == user.id {
        return Err(ReadTicketError::Forbidden(
            "You cannot read your own giving ticket.".to_string(),
        ));
    };
    if ticket.status == TicketStatus::Draft.to_value() {
        return Err(ReadTicketError::NotFound(format!(
            "Ticket not found for id: {ticket_id}"
        )));
    }

    let ticket = ticket_service.mark_ticket_read(ticket).await?;

    Ok(TicketVisible::from(ticket))
}
