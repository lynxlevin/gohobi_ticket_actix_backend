use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{TicketMutation, TicketQuery};
use entities::users_user;

pub async fn delete_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
) -> Result<(), UseCaseError> {
    let (ticket, wish) = ticket_query
        .filter_which_user_has_access(user.id)
        .join_wish()
        .get_with_wish_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id != user.id || wish.is_some() {
        return Err(UseCaseError::Forbidden);
    };

    ticket_mutation
        .delete(ticket)
        .await
        .map_err(|_| UseCaseError::InternalServerError)
}
