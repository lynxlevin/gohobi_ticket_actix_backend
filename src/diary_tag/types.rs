use db_adapters::diary_tag::types::DiaryTagVisible;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListDiaryTagsQuery {
    pub user_relation_id: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ListDiaryTagsResponse {
    pub diary_tags: Vec<DiaryTagVisible>,
}
