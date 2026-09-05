use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod wish_reply_mutation;

pub use wish_reply_mutation::*;

#[derive(Debug, Error)]
pub enum WishReplyServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    WishNotFound(),
    #[error("")]
    UserRelationNotFound(),
}
impl From<TransactionError<WishReplyServiceError>> for WishReplyServiceError {
    fn from(value: TransactionError<WishReplyServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct WishReplyService<'a> {
    pub db: &'a DbConn,
}

impl<'a> WishReplyService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
