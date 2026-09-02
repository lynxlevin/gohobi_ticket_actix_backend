use domain_services::diary_tag::DiaryTagWithDiaryCount;
use entities::{
    diaries_diarytag::{self, Model},
    user_relations_userrelation::UserRelationId,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTagVisible {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
}
impl From<Model> for DiaryTagVisible {
    fn from(value: Model) -> Self {
        Self { id: value.id, text: value.text, sort_no: value.sort_no }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTagVisibleWithDiaryCount {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
    pub diary_count: i64,
}
impl From<DiaryTagWithDiaryCount> for DiaryTagVisibleWithDiaryCount {
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
    pub diary_tags: Vec<BulkUpdateDiaryTagInput>,
    pub user_relation_id: UserRelationId,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BulkUpdateDiaryTagInput {
    pub id: Option<Uuid>,
    pub text: String,
    pub sort_no: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkUpdateDiaryTagResponse {
    pub diary_tags: Vec<BulkUpdateDiaryTagItem>,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct BulkUpdateDiaryTagItem {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
}
impl From<diaries_diarytag::Model> for BulkUpdateDiaryTagItem {
    fn from(value: diaries_diarytag::Model) -> Self {
        Self { id: value.id, text: value.text, sort_no: value.sort_no }
    }
}
