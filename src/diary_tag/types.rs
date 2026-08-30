use db_adapters::diary_tag::types::BulkUpdateDiaryTagItem;
use domain_services::diary_tag::DiaryTagWithDiaryCount;
use entities::user_relations_userrelation::UserRelationId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTagVisible {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
    pub diary_count: i64,
}
impl From<DiaryTagWithDiaryCount> for DiaryTagVisible {
    fn from(value: DiaryTagWithDiaryCount) -> Self {
        Self { id: value.id, text: value.text, sort_no: value.sort_no, diary_count: value.diary_count }
    }
}

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
