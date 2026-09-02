use common::db::Db;
use entities::{
    prelude::TicketsTicket,
    tickets_ticket::{Column, Entity, Relation, TicketId},
    user_relations_userrelation as user_relation,
    users_user::UserId,
};
use sea_orm::{
    ColumnTrait, Condition, DbConn, EntityTrait, JoinType::LeftJoin, QueryFilter, QuerySelect, RelationTrait,
    Select, TransactionError,
};
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
    #[error("{0}")]
    NotGivingTicket(String),
    #[error("You can only read a ticket you received.")]
    NotReceivingTicket(),
    #[error("You can only delete a ticket not used yet.")]
    NotUnusedTicket(),
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

fn get_query_tickets_with_access_to_user(user_id: UserId) -> Select<TicketsTicket> {
    Entity::find()
        .join(LeftJoin, Relation::UserRelationsUserrelation.def())
        .filter(
            Condition::any()
                .add(user_relation::Column::User1Id.eq(user_id))
                .add(user_relation::Column::User2Id.eq(user_id)),
        )
}
fn get_query_ticket_by_id(user_id: UserId, ticket_id: TicketId) -> Select<TicketsTicket> {
    get_query_tickets_with_access_to_user(user_id).filter(Column::Id.eq(ticket_id))
}
