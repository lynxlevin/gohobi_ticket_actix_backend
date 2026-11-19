use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod web_push_mutation;
mod web_push_query;

pub use web_push_mutation::*;
pub use web_push_query::*;

#[derive(Debug, Error)]
pub enum WebPushServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    NotFound(),
}
impl From<TransactionError<WebPushServiceError>> for WebPushServiceError {
    fn from(value: TransactionError<WebPushServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct WebPushService<'a> {
    pub db: &'a DbConn,
}

impl<'a> WebPushService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
