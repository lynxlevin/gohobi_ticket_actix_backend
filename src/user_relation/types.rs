use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct ListUserRelationsResponse {
    pub user_relations: Vec<UserRelationVisible>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct UserRelationVisible {
    pub id: i64,
    pub related_user_name: String,
    pub giving_ticket_img: Option<String>,
    pub receiving_ticket_img: Option<String>,
    pub use_slack: bool,
}
