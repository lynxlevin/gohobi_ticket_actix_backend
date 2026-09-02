use chrono::NaiveDate;
use domain_services::diary::{DiaryTagInner, DiaryWithTags};
use entities::{diaries_diary::DiaryStatus, diaries_diarytag, user_relations_userrelation::UserRelationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTag {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
}
impl From<&diaries_diarytag::Model> for DiaryTag {
    fn from(value: &diaries_diarytag::Model) -> Self {
        Self { id: value.id, text: value.text.clone(), sort_no: value.sort_no }
    }
}
impl From<&DiaryTagInner> for DiaryTag {
    fn from(value: &DiaryTagInner) -> Self {
        Self { id: value.id, text: value.text.clone(), sort_no: value.sort_no }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryVisible {
    pub id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub status: DiaryStatus,
    pub tags: Vec<DiaryTag>,
}
impl From<DiaryWithTags> for DiaryVisible {
    fn from(value: DiaryWithTags) -> Self {
        Self {
            id: value.id,
            entry: value.entry.clone(),
            date: value.date,
            status: value.status,
            tags: value.tags.iter().map(|tag| tag.into()).collect(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateDiaryRequest {
    pub user_relation_id: UserRelationId,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateDiaryRequest {
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}
