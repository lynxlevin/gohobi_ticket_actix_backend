use common::db::Db;
use entities::{
    diaries_diarytag::{Column, Entity},
    prelude::DiariesDiarytag,
    user_relations_userrelation::UserRelationId,
};
use sea_orm::{
    ColumnTrait, DbConn, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Select,
    TransactionError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod diary_tag_mutation;
mod diary_tag_query;

pub use diary_tag_mutation::*;
pub use diary_tag_query::*;

#[derive(Debug, Error)]
pub enum DiaryTagServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    UserRelationNotFound(),
    #[error("")]
    DiaryTagNotFound(),
}
impl From<TransactionError<DiaryTagServiceError>> for DiaryTagServiceError {
    fn from(value: TransactionError<DiaryTagServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct DiaryTagService<'a> {
    pub db: &'a DbConn,
}

impl<'a> DiaryTagService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq, FromQueryResult)]
pub struct DiaryTagWithDiaryCount {
    pub id: Uuid,
    pub text: String,
    pub sort_no: i32,
    pub diary_count: i64,
}

fn list_tags_query(user_relation_id: UserRelationId) -> Select<DiariesDiarytag> {
    Entity::find()
        .filter(Column::UserRelationId.eq(user_relation_id))
        .group_by(Column::Id)
        .order_by_asc(Column::SortNo)
}
