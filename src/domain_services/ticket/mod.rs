use common::db::Db;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod ticket_mutation;
mod ticket_query;

pub use ticket_mutation::*;
pub use ticket_query::*;

#[derive(Debug, Error)]
pub enum TicketServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    UserRelationNotFound(),
    #[error("")]
    TicketNotFound(),
    #[error("You can only update a ticket you gave.")]
    NotGivingTicket(),
    #[error("You can only read a ticket you received.")]
    NotReceivingTicket(),
    #[error("{0}")]
    ValidationError(String),
}
impl From<TransactionError<TicketServiceError>> for TicketServiceError {
    fn from(value: TransactionError<TicketServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

// MYMEMO: refactor TicketService and ticket use_cases like diary
// MYMEMO: remove this later
#[derive(Clone)]
pub struct TicketService<'a> {
    pub db: &'a DbConn,
}

impl<'a> TicketService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }
}
