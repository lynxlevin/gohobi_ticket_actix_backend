use chrono::{DateTime, FixedOffset, NaiveDate};
use db_adapters::ticket_service::{CreateTicketParams, UpdateTicketParams};
use entities::{
    custom_types::{TicketStatus, WishStatus},
    tickets_ticket,
    user_relations_userrelation::UserRelationId,
    users_user::UserId,
    wish,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct ListTicketResponse {
    pub tickets: Vec<TicketVisible>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct TicketVisible {
    pub id: i64,
    pub user_relation_id: UserRelationId,
    pub giving_user_id: UserId,
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

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct WishVisible {
    pub id: Uuid,
    pub description: String,
    pub status: WishStatus,
    pub created_at: DateTime<FixedOffset>,
    pub ticket: TicketInner,
}
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct TicketInner {
    pub id: i64,
    pub giving_user_id: UserId,
    pub description: String,
    pub gift_date: NaiveDate,
    pub is_special: bool,
}
impl From<(&wish::Model, &tickets_ticket::Model)> for WishVisible {
    fn from((wish, ticket): (&wish::Model, &tickets_ticket::Model)) -> Self {
        Self {
            id: wish.id,
            description: wish.description.to_owned(),
            status: (&wish.status).into(),
            created_at: wish.created_at,
            ticket: TicketInner {
                id: ticket.id,
                giving_user_id: ticket.giving_user_id,
                description: ticket.description.to_owned(),
                gift_date: ticket.gift_date,
                is_special: ticket.is_special,
            },
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

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub enum WebPushResult {
    Sent,
    NotSent,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct MakeWishParams {
    pub use_description: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MakeWishRequest {
    pub ticket: MakeWishParams,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MakeWishResponse {
    pub ticket: TicketVisible,
    pub web_push_result: WebPushResult,
}
