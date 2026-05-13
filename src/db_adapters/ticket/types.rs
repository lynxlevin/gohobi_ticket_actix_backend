use chrono::NaiveDate;
use entities::custom_types::TicketStatus;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct UpdateTicketParams {
    pub description: Option<String>,
    pub is_special: Option<bool>,
    pub status: Option<TicketStatus>,
    pub use_description: Option<String>,
    pub use_date: Option<NaiveDate>,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct CreateWishParams {
    pub use_description: String,
    pub ticket_id: i64,
    pub user_relation_id: i64,
}
