use common::errors::use_case_errors::UseCaseError;
use db_adapters::{
    ticket::{Order, TicketMutation, TicketQuery},
    user_relation::{UserRelationMutation, UserRelationQuery},
};
use entities::users_user;

pub async fn delete_ticket(
    user: users_user::Model,
    user_relation_query: UserRelationQuery<'_>,
    user_relation_mutation: UserRelationMutation<'_>,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
) -> Result<(), UseCaseError> {
    let (ticket, wish) = ticket_query
        .clone()
        .filter_which_user_has_access(user.id)
        .join_wish()
        .get_with_wish_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id != user.id || wish.is_some() {
        return Err(UseCaseError::Forbidden);
    };

    let user_relation = user_relation_query
        .find_by_id(ticket.user_relation_id, user.id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;
    if user.id == user_relation.user_1_id {
        if user_relation
            .first_user_1_giving_ticket_date
            .is_some_and(|date| date == ticket.gift_date)
        {
            let next_oldest_ticket = ticket_query
                .filter_by_relation(user_relation.id)
                .filter_by_giving_user(user.id)
                .exclude_id(ticket.id)
                .order_by_gift_date(Order::Asc)
                .get_one()
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
            user_relation_mutation
                .update_first_user_1_giving_ticket_date(
                    user_relation,
                    next_oldest_ticket.and_then(|t| Some(t.gift_date)),
                )
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
        }
    } else {
        if user_relation
            .first_user_2_giving_ticket_date
            .is_some_and(|date| date == ticket.gift_date)
        {
            let next_oldest_ticket = ticket_query
                .filter_by_relation(user_relation.id)
                .filter_by_giving_user(user.id)
                .exclude_id(ticket.id)
                .order_by_gift_date(Order::Asc)
                .get_one()
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
            user_relation_mutation
                .update_first_user_2_giving_ticket_date(
                    user_relation,
                    next_oldest_ticket.and_then(|t| Some(t.gift_date)),
                )
                .await
                .map_err(|_| UseCaseError::InternalServerError)?;
        }
    }

    ticket_mutation
        .delete(ticket)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    Ok(())
}
