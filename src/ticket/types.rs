use chrono::NaiveDate;
use entities::tickets_ticket;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub enum TicketStatus {
    Unread,
    Read,
    Edited,
    Draft,
    Invalid,
}

impl TicketStatus {
    pub fn to_value(self) -> String {
        match self {
            TicketStatus::Unread => "unread".to_string(),
            TicketStatus::Read => "read".to_string(),
            TicketStatus::Edited => "edited".to_string(),
            TicketStatus::Draft => "draft".to_string(),
            TicketStatus::Invalid => "invalid".to_string(),
        }
    }
}

impl From<String> for TicketStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "unread" => TicketStatus::Unread,
            "read" => TicketStatus::Read,
            "edited" => TicketStatus::Edited,
            "draft" => TicketStatus::Draft,
            _ => TicketStatus::Invalid,
        }
    }
}

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
            status: value.status.into(),
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
            status: value.status.to_owned().into(),
            is_special: value.is_special,
        }
    }
}
