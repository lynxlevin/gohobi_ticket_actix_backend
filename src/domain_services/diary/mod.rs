use chrono::NaiveDate;
use common::db::Db;
use entities::{diaries_diary::DiaryStatus, diaries_diarytag};
use sea_orm::{DbConn, TransactionError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod diary_mutation;
mod diary_query;

pub use diary_mutation::*;
pub use diary_query::*;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DiaryServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    UserRelationNotFound(),
    #[error("")]
    DiaryNotFound(),
    // #[error("Diary cannot be found.")]
    // DiaryNotFound(),
    // #[error("{0}")]
    // RelatedRecordNotFound(String),
}
impl From<TransactionError<DiaryServiceError>> for DiaryServiceError {
    fn from(value: TransactionError<DiaryServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

#[derive(Clone)]
pub struct DiaryService<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryTagInner {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
}
impl From<diaries_diarytag::Model> for DiaryTagInner {
    fn from(value: diaries_diarytag::Model) -> Self {
        Self { id: value.id, text: value.text, sort_no: value.sort_no }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq)]
pub struct DiaryWithTags {
    pub id: Uuid,
    pub entry: String,
    pub date: NaiveDate,
    pub tags: Vec<DiaryTagInner>,
    pub status: DiaryStatus,
}
