use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    ticket::{types::CreateTicketParams, TicketMutation, TicketQuery},
    user_relation::UserRelationQuery,
};
use entities::users_user;

use crate::TicketVisible;

pub async fn create_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    params: &mut CreateTicketParams,
) -> Result<TicketVisible, UseCaseError> {
    user_relation_query
        .find_by_id(params.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if params.is_special.is_some_and(|b| b) {
        let exists = ticket_query
            .exists_other_special_ticket(user.id, params.user_relation_id, params.gift_date)
            .await
            .map_err(|_| UseCaseError::InternalServerError)?;
        params.is_special = Some(!exists);
    };

    ticket_mutation
        .create(user.id, params.clone())
        .await
        .map(|ticket| TicketVisible::from(ticket))
        .map_err(|_| UseCaseError::InternalServerError)
}
