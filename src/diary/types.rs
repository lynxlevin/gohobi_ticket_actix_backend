use chrono::NaiveDate;
use db_adapters::diary::types::DiaryStatus;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTag {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryVisible {
    pub id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub tags: Vec<DiaryTag>,
    pub status: DiaryStatus,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateDiaryRequest {
    pub user_relation_id: i64,
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Vec<Uuid>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct UpdateDiaryRequest {
    pub entry: String,
    pub date: NaiveDate,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct UpsertDiaryResponse {
    pub id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub status: DiaryStatus,
    pub tag_ids: Option<Vec<Uuid>>,
}
