use common::db::Db;
use domain_services::ticket::{CreateTicketParams, TicketService, TicketServiceError, TicketServiceMutation};
use entities::users_user;
use thiserror::Error;

use crate::TicketVisible;

#[derive(Debug, Error)]
pub enum CreateTicketError {
    #[error("UserRelation not found")]
    UserRelationNotFound(),
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for CreateTicketError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            TicketServiceError::ValidationError(_) => Self::ValidationError(e.to_string()),
            TicketServiceError::UserRelationNotFound() => Self::UserRelationNotFound(),
            _ => Self::InternalServerError(e.to_string()),
        }
    }
}

pub async fn create_ticket(
    user: users_user::Model,
    params: CreateTicketParams,
    db: &Db,
) -> Result<TicketVisible, CreateTicketError> {
    let ticket_service = TicketService::init(db);

    let ticket = ticket_service.create_ticket(user.id, params).await?;

    Ok(TicketVisible::from(ticket))
}
