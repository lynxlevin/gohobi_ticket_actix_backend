use domain_services::ticket::{TicketService, TicketServiceError, TicketServiceMutation};
use entities::{tickets_ticket::TicketId, users_user};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeleteTicketError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for DeleteTicketError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::NotGivingTicket(_) | TicketServiceError::NotUnusedTicket() => {
                DeleteTicketError::Forbidden(e.to_string())
            }
            TicketServiceError::TicketNotFound() | TicketServiceError::UserRelationNotFound() => {
                DeleteTicketError::NotFound(e.to_string())
            }
            _ => DeleteTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn delete_ticket(
    user: users_user::Model,
    ticket_id: TicketId,
    ticket_service: TicketService<'_>,
) -> Result<(), DeleteTicketError> {
    ticket_service.delete_ticket(user.id, ticket_id).await?;

    Ok(())
}
