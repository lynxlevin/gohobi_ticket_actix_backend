use db_adapters::diary_tag::types::{BulkUpdateDiaryTagItem, DiaryTagVisible};
use entities::user_relations_userrelation::UserRelationId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ListDiaryTagsQuery {
    pub user_relation_id: UserRelationId,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ListDiaryTagsResponse {
    pub diary_tags: Vec<DiaryTagVisible>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BulkUpdateDiaryTagRequest {
    pub diary_tags: Vec<BulkUpdateDiaryTagItem>,
    pub user_relation_id: UserRelationId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkUpdateDiaryTagResponse {
    pub diary_tags: Vec<BulkUpdateDiaryTagItem>,
}
