use chrono::{DateTime, FixedOffset, NaiveDate};
use db_adapters::ticket::types::{
    CreateTicketParams, TicketStatus, UpdateTicketParams, WishStatus,
};
use entities::{tickets_ticket, wish};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct ListTicketResponse {
    pub tickets: Vec<TicketVisible>,
}

// MYMEMO: Add wish, maybe
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct TicketVisible {
    pub id: i64,
    pub user_relation_id: i64,
    pub giving_user_id: i64,
    pub description: String,
    pub gift_date: NaiveDate,
    pub status: TicketStatus,
    pub is_special: bool,
    pub wish: Option<WishInner>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct WishInner {
    pub id: Uuid,
    pub description: String,
    pub status: WishStatus,
    pub created_at: DateTime<FixedOffset>,
}

impl From<tickets_ticket::Model> for TicketVisible {
    fn from(value: tickets_ticket::Model) -> Self {
        Self {
            id: value.id,
            user_relation_id: value.user_relation_id,
            giving_user_id: value.giving_user_id,
            description: value.description,
            gift_date: value.gift_date,
            status: (&value.status).into(),
            is_special: value.is_special,
            wish: None,
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
            status: (&value.status).into(),
            is_special: value.is_special,
            wish: None,
        }
    }
}
impl TicketVisible {
    pub fn with_wish(mut self, wish: &wish::Model) -> Self {
        self.wish = Some(WishInner {
            id: wish.id,
            description: wish.description.clone(),
            status: (&wish.status).into(),
            created_at: wish.created_at,
        });
        self
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

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub enum WebPushResult {
    Sent,
    NotSent,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UseTicketParams {
    pub use_description: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UseTicketRequest {
    pub ticket: UseTicketParams,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UseTicketResponse {
    pub ticket: TicketVisible,
    pub web_push_result: WebPushResult,
}
