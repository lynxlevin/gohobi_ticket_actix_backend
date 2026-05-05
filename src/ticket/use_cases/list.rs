use chrono::NaiveDate;
use common::errors::use_case_errors::UseCaseError;
use db_adapters::ticket::{Order, TicketQuery};
use entities::users_user;
use serde::Deserialize;

use crate::{ListTicketResponse, TicketVisible};

#[derive(Debug, Deserialize, Default)]
pub struct ListTicketsQueryParam {
    pub user_relation_id: i64,
    pub is_giving: Option<String>,
    pub gift_date_gte: Option<NaiveDate>,
    pub gift_date_lte: Option<NaiveDate>,
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
        .filter_by_relation(query_param.user_relation_id)
        .join_wish();
    if let Some(text_query) = text_query {
        ticket_query = ticket_query.filter_contains_texts(text_query);
    }
    if let Some(gift_date_gte) = query_param.gift_date_gte {
        ticket_query = ticket_query.filter_gift_date_gte(gift_date_gte);
    }
    if let Some(gift_date_lte) = query_param.gift_date_lte {
        ticket_query = ticket_query.filter_gift_date_lte(gift_date_lte);
    }
    ticket_query
        .order_by_gift_date(Order::Desc)
        .order_by_created_at(Order::Desc)
        .get_tickets_with_wish(user.id, is_giving)
        .await
        .map(|tickets| ListTicketResponse {
            tickets: tickets
                .iter()
                .map(|(ticket, wish)| match wish {
                    Some(wish) => TicketVisible::from(ticket).with_wish(wish),
                    None => TicketVisible::from(ticket),
                })
                .collect(),
        })
        .map_err(|e| {
            dbg!(e);
            UseCaseError::InternalServerError
        })
}
