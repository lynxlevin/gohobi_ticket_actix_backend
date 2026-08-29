use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod diary_mutation;
mod diary_query;

pub use diary_mutation::*;
pub use diary_query::*;

#[derive(Debug, Error)]
pub enum DiaryServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error(transparent)]
    DbTransactionErr(#[from] TransactionError<sea_orm::DbErr>),
    // #[error("Diary cannot be found.")]
    // DiaryNotFound(),
    // #[error("{0}")]
    // RelatedRecordNotFound(String),
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
