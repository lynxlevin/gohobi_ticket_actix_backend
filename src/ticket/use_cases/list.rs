use chrono::NaiveDate;
use db_adapters::ticket_service::{
    ListTicketsWithWishParams, TicketService, TicketServiceError, TicketServiceQuery,
};
use entities::{user_relations_userrelation::UserRelationId, users_user};
use serde::Deserialize;
use thiserror::Error;

use crate::{ListTicketResponse, TicketVisible};

#[derive(Deserialize, Debug, Default)]
pub struct ListTicketsParams {
    pub user_relation_id: UserRelationId,
    pub text_query: Option<Vec<String>>,
    pub gift_date_gte: Option<NaiveDate>,
    pub gift_date_lte: Option<NaiveDate>,
    pub is_giving: bool,
}

#[derive(Debug, Error)]
pub enum ListTicketsError {
    #[error("{0}")]
    InternalServerError(String),
}
impl From<TicketServiceError> for ListTicketsError {
    fn from(e: TicketServiceError) -> Self {
        ListTicketsError::InternalServerError(e.to_string())
    }
}

pub async fn list_tickets(
    user: users_user::Model,
    ticket_service: TicketService<'_>,
    params: ListTicketsParams,
) -> Result<ListTicketResponse, ListTicketsError> {
    let tickets_with_wish = ticket_service
        .list_tickets_with_wish(
            user.id,
            params.user_relation_id,
            ListTicketsWithWishParams {
                text_query: params.text_query,
                gift_date_gte: params.gift_date_gte,
                gift_date_lte: params.gift_date_lte,
                is_giving: params.is_giving,
            },
        )
        .await?;

    Ok(ListTicketResponse {
        tickets: tickets_with_wish
            .iter()
            .map(|(ticket, wish)| match wish {
                Some(wish) => TicketVisible::from(ticket).with_wish(wish),
                None => TicketVisible::from(ticket),
            })
            .collect(),
    })
}
