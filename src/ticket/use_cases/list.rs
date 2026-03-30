use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{Order, TicketQuery};
use entities::users_user;
use serde::Deserialize;

use crate::{ListTicketResponse, TicketVisible};

#[derive(Debug, Deserialize)]
pub struct ListTicketsQueryParam {
    pub user_relation_id: i64,
    pub is_giving: Option<String>,
}

pub async fn list_tickets(
    user: users_user::Model,
    ticket_query: TicketQuery<'_>,
    query_param: ListTicketsQueryParam,
    text_query: Option<Vec<&str>>,
) -> Result<ListTicketResponse, UseCaseError> {
    let is_giving = query_param
        .is_giving
        .is_some_and(|x| x != "false".to_string());

    let mut ticket_query = ticket_query
        .filter_which_user_has_access(user.id)
        .filter_by_relation(query_param.user_relation_id);
    if let Some(text_query) = text_query {
        ticket_query = ticket_query.join_wish().filter_contains_texts(text_query);
    }
    ticket_query
        .order_by_gift_date(Order::Desc)
        .order_by_created_at(Order::Desc)
        .get_tickets(user.id, is_giving)
        .await
        .map(|tickets| ListTicketResponse {
            tickets: tickets
                .iter()
                .map(|ticket| TicketVisible::from(ticket))
                .collect(),
        })
        .map_err(|_| UseCaseError::InternalServerError)
}
