use chrono::NaiveDate;
use entities::{diaries_diary::DiaryStatus, user_relations_userrelation::UserRelationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateDiaryParams {
    pub entry: String,
    pub date: NaiveDate,
    pub user_relation_id: UserRelationId,
    pub user_1_status: DiaryStatus,
    pub user_2_status: DiaryStatus,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateDiaryParams {
    pub entry: Option<String>,
    pub date: Option<NaiveDate>,
    pub user_1_status: Option<DiaryStatus>,
    pub user_2_status: Option<DiaryStatus>,
}
