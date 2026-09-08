use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod web_push_subscription_mutation;
mod web_push_subscription_query;

pub use web_push_subscription_mutation::*;
pub use web_push_subscription_query::*;

#[derive(Debug, Error)]
pub enum WebPushSubscriptionServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    NotFound(),
}
impl From<TransactionError<WebPushSubscriptionServiceError>> for WebPushSubscriptionServiceError {
    fn from(value: TransactionError<WebPushSubscriptionServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct WebPushSubscriptionService<'a> {
    pub db: &'a DbConn,
}

impl<'a> WebPushSubscriptionService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
