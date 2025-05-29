use chrono::NaiveDate;
use db_adapters::ticket::types::{CreateTicketParams, TicketStatus, UpdateTicketParams};
use entities::tickets_ticket;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct ListTicketResponse {
    pub tickets: Vec<TicketVisible>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct TicketVisible {
    pub id: i64,
    pub user_relation_id: i64,
    pub giving_user_id: i64,
    pub description: String,
    pub gift_date: NaiveDate,
    pub use_description: String,
    pub use_date: Option<NaiveDate>,
    pub status: TicketStatus,
    pub is_special: bool,
}

impl From<tickets_ticket::Model> for TicketVisible {
    fn from(value: tickets_ticket::Model) -> Self {
        Self {
            id: value.id,
            user_relation_id: value.user_relation_id,
            giving_user_id: value.giving_user_id,
            description: value.description,
            gift_date: value.gift_date,
            use_description: value.use_description,
            use_date: value.use_date,
            status: (&value.status).into(),
            is_special: value.is_special,
        }
    }
}

impl From<&tickets_ticket::Model> for TicketVisible {
    fn from(value: &tickets_ticket::Model) -> Self {
        Self {
            id: value.id,
            user_relation_id: value.user_relation_id,
            giving_user_id: value.giving_user_id,
            description: value.description.to_owned(),
            gift_date: value.gift_date,
            use_description: value.use_description.to_owned(),
            use_date: value.use_date,
            status: (&value.status).into(),
            is_special: value.is_special,
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CreateTicketRequest {
    pub ticket: CreateTicketParams,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpsertTicketResponse {
    pub ticket: TicketVisible,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UpdateTicketRequest {
    pub ticket: UpdateTicketParams,
}
