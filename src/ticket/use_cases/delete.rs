use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{TicketMutation, TicketQuery};
use entities::users_user;

pub async fn delete_ticket(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    ticket_mutation: TicketMutation<'_>,
    ticket_id: i64,
) -> Result<(), UseCaseError> {
    let ticket = ticket_query
        .filter_which_user_has_access(user.id)
        .get_by_id(ticket_id)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?
        .ok_or(UseCaseError::NotFound)?;

    if ticket.giving_user_id != user.id || ticket.use_date.is_some() {
        return Err(UseCaseError::Forbidden);
    };

    ticket_mutation
        .delete(ticket)
        .await
        .map_err(|_| UseCaseError::InternalServerError)
}
