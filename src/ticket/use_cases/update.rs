use domain_services::ticket::{TicketService, TicketServiceError, TicketServiceMutation, UpdateTicketParams};
use entities::{tickets_ticket::TicketId, users_user};
use thiserror::Error;

use crate::TicketVisible;

#[derive(Debug, Error)]
pub enum UpdateTicketError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for UpdateTicketError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::ValidationError(_) => UpdateTicketError::ValidationError(e.to_string()),
            TicketServiceError::NotGivingTicket() => UpdateTicketError::Forbidden(e.to_string()),
            TicketServiceError::TicketNotFound() => UpdateTicketError::NotFound(e.to_string()),
            _ => UpdateTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn update_ticket(
    user: users_user::Model,
    ticket_service: TicketService<'_>,
    ticket_id: TicketId,
    params: UpdateTicketParams,
) -> Result<TicketVisible, UpdateTicketError> {
    let ticket = ticket_service.update_ticket(user.id, ticket_id, params).await?;

    Ok(TicketVisible::from(ticket))
}
