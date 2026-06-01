use db_adapters::ticket_service::{TicketService, TicketServiceError, TicketServiceMutation, TicketServiceQuery};
use entities::users_user;
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
            TicketServiceError::TicketNotFound(_) => DeleteTicketError::NotFound(e.to_string()),
            _ => DeleteTicketError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn delete_ticket(
    user: users_user::Model,
    ticket_id: i64,
    ticket_service: TicketService<'_>,
) -> Result<(), DeleteTicketError> {
    let (ticket, wish) = ticket_service.get_ticket_with_wish_by_id(user.id, ticket_id).await?;

    if ticket.giving_user_id != user.id {
        return Err(DeleteTicketError::Forbidden(
            "You cannot delete a ticket you received.".to_string(),
        ));
    };
    if wish.is_some() {
        return Err(DeleteTicketError::Forbidden(
            "You cannot delete a used ticket.".to_string(),
        ));
    };

    ticket_service.delete_ticket(user.id, ticket).await?;

    Ok(())
}
