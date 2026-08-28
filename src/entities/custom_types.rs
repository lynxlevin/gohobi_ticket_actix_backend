use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum WishStatus {
    Unread,
    Read,
    Invalid,
}

impl WishStatus {
    pub fn to_value(self) -> String {
        match self {
            WishStatus::Unread => "unread".to_string(),
            WishStatus::Read => "read".to_string(),
            WishStatus::Invalid => "invalid".to_string(),
        }
    }
}

impl From<&String> for WishStatus {
    fn from(value: &String) -> Self {
        value.parse().unwrap()
    }
}

impl FromStr for WishStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unread" => Ok(WishStatus::Unread),
            "read" => Ok(WishStatus::Read),
            // NOTE: Invalid status should fall back to unread so that after reading, it will safely be turned to read.
            _ => Ok(WishStatus::Unread),
        }
    }
}
