use entities::diaries_diarytag;
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct BulkUpdateDiaryTagItem {
    pub id: Option<Uuid>,
    pub text: String,
    pub sort_no: i32,
}

impl From<&diaries_diarytag::Model> for BulkUpdateDiaryTagItem {
    fn from(value: &diaries_diarytag::Model) -> Self {
        Self {
            id: Some(value.id),
            text: value.text.clone(),
            sort_no: value.sort_no,
        }
    }
}
