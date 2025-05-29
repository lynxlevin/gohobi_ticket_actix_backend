use std::str::FromStr;

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

impl From<&String> for TicketStatus {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}

impl FromStr for TicketStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unread" => Ok(TicketStatus::Unread),
            "read" => Ok(TicketStatus::Read),
            "edited" => Ok(TicketStatus::Edited),
            "draft" => Ok(TicketStatus::Draft),
            // NOTE: Invalid status should fall back to unread so that after reading, it will safely be turned to read.
            _ => Ok(TicketStatus::Unread),
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
