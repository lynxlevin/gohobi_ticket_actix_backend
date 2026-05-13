use db_adapters::ticket_service::{CreateTicketParams, TicketService, TicketServiceError};
use entities::users_user;
use thiserror::Error;

use crate::TicketVisible;

#[derive(Debug, Error)]
pub enum CreateTicketError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InternalServerError(String),
}

pub async fn create_ticket(
    user: users_user::Model,
    params: &mut CreateTicketParams,
    ticket_repository: TicketService<'_>,
) -> Result<TicketVisible, CreateTicketError> {
    if params.is_special {
        let special_ticket_exists = ticket_repository
            .check_special_ticket_existence(user.id, params.user_relation_id, params.gift_date)
            .await
            .map_err(|e| CreateTicketError::InternalServerError(e.to_string()))?;
        params.is_special = !special_ticket_exists;
    }

    let ticket = ticket_repository
        .create_ticket(user.id, params.clone())
        .await
        .map_err(|e| match e {
            TicketServiceError::UserRelationNotFound(_) => {
                CreateTicketError::NotFound(e.to_string())
            }
            _ => CreateTicketError::InternalServerError(e.to_string()),
        })?;

    Ok(TicketVisible::from(ticket))
}
