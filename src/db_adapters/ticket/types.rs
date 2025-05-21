use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
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

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateTicketParams {
    pub gift_date: NaiveDate,
    pub description: String,
    pub user_relation_id: i64,
    pub is_special: Option<bool>,
    pub status: Option<TicketStatus>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateTicketParams {
    pub description: Option<String>,
    pub status: Option<TicketStatus>,
}
