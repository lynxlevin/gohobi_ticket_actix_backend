use sea_orm::FromQueryResult;
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

#[derive(FromQueryResult)]
pub struct UserRelationWithName {
    pub id: i64,
    pub user_1_giving_ticket_img: Option<String>,
    pub user_2_giving_ticket_img: Option<String>,
    pub user_1_id: i64,
    pub user_1_name: String,
    pub user_2_name: String,
    pub use_slack: bool,
}
