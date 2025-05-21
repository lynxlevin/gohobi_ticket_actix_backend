use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, PartialEq, FromQueryResult)]
pub struct DiaryTagVisible {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
    pub diary_count: i64,
}
