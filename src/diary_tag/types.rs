use db_adapters::diary_tag::types::{BulkUpdateDiaryTagItem, DiaryTagVisible};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListDiaryTagsQuery {
    pub user_relation_id: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ListDiaryTagsResponse {
    pub diary_tags: Vec<DiaryTagVisible>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BulkUpdateDiaryTagRequest {
    pub diary_tags: Vec<BulkUpdateDiaryTagItem>,
    pub user_relation_id: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkUpdateDiaryTagResponse {
    pub diary_tags: Vec<BulkUpdateDiaryTagItem>,
}
