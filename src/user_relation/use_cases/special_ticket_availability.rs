use crate::types::SpecialTicketAvailabilityQueryParam;
use chrono::NaiveDate;
use common::errors::use_case_errors::UseCaseError;
use db_adapters::{ticket::TicketQuery, user_relation::UserRelationQuery};

pub async fn check_special_ticket_availability(
    user_id: i64,
    user_relation_id: i64,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    query: SpecialTicketAvailabilityQueryParam,
) -> Result<bool, UseCaseError> {
    match user_relation_query
        .find_by_id(user_relation_id, user_id)
        .await
    {
        Ok(user_relation) => match user_relation {
            Some(_) => {}
            None => return Err(UseCaseError::NotFound),
        },
        Err(_) => return Err(UseCaseError::InternalServerError),
    }
    match ticket_query
        .filter_by_user(user_id)
        .exists_other_special_ticket(
            user_id,
            user_relation_id,
            match NaiveDate::from_ymd_opt(query.year, query.month, 1) {
                Some(date) => date,
                None => return Err(UseCaseError::InternalServerError),
            },
        )
        .await
    {
        Ok(exists) => Ok(!exists),
        Err(_) => Err(UseCaseError::InternalServerError),
    }
}
