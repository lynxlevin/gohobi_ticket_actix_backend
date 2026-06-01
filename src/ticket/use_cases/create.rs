use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    ticket::{types::CreateTicketParams, TicketMutation, TicketQuery},
    user_relation::{UserRelationMutation, UserRelationQuery},
};
use entities::users_user;

use crate::TicketVisible;

pub async fn create_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    user_relation_mutation: UserRelationMutation<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    params: &mut CreateTicketParams,
) -> Result<TicketVisible, UseCaseError> {
    let user_relation = user_relation_query
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

    let ticket = ticket_mutation
        .create(user.id, params.clone())
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    if ticket.giving_user_id == user_relation.user_1_id {
        if user_relation
            .first_user_1_giving_ticket_date
            .is_none_or(|date| date > ticket.gift_date)
        {
            user_relation_mutation
                .update_first_user_1_giving_ticket_date(user_relation, Some(ticket.gift_date))
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
        }
    } else {
        if user_relation
            .first_user_2_giving_ticket_date
            .is_none_or(|date| date > ticket.gift_date)
        {
            user_relation_mutation
                .update_first_user_2_giving_ticket_date(user_relation, Some(ticket.gift_date))
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
        }
    }

    Ok(TicketVisible::from(ticket))
}
