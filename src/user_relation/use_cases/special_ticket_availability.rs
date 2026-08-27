use crate::types::SpecialTicketAvailabilityQueryParam;
use chrono::NaiveDate;
use db_adapters::{
    ticket_service::{TicketService, TicketServiceError, TicketServiceQuery},
    user_relation::UserRelationQuery,
};
use entities::{user_relations_userrelation::UserRelationId, users_user::UserId};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckSpecialTicketAvailabilityError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    ValidationError(String),
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for CheckSpecialTicketAvailabilityError {
    fn from(e: TicketServiceError) -> Self {
        match e {
            _ => CheckSpecialTicketAvailabilityError::InternalServerError(e.to_string()),
        }
    }
}

pub async fn check_special_ticket_availability(
    user_id: UserId,
    user_relation_id: UserRelationId,
    user_relation_query: UserRelationQuery<'_>,
    ticket_service: TicketService<'_>,
    query: SpecialTicketAvailabilityQueryParam,
) -> Result<bool, CheckSpecialTicketAvailabilityError> {
    user_relation_query
        .find_by_id(user_relation_id, user_id)
        .await
        .map_err(|e| CheckSpecialTicketAvailabilityError::InternalServerError(e.to_string()))?
        .ok_or(CheckSpecialTicketAvailabilityError::NotFound(format!(
            "UserRelation not found for id: {}.",
            user_relation_id
        )))?;

    let date = NaiveDate::from_ymd_opt(query.year, query.month, 1).ok_or(
        CheckSpecialTicketAvailabilityError::ValidationError("Invalid year or month.".to_string()),
    )?;

    let exists = ticket_service
        .check_special_ticket_existence(user_id, user_relation_id, date)
        .await?;

    Ok(!exists)
}
