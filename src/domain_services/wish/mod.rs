use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod wish_query;

pub use wish_query::*;

#[derive(Debug, Error)]
pub enum WishServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    WishNotFound(),
    #[error("")]
    TicketNotFound(),
}
impl From<TransactionError<WishServiceError>> for WishServiceError {
    fn from(value: TransactionError<WishServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct WishService<'a> {
    pub db: &'a DbConn,
}

impl<'a> WishService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
