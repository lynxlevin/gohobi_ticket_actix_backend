use std::future::Future;

use chrono::{Datelike, NaiveDate, Utc};
use entities::{
    custom_types::TicketStatus, prelude::TicketsTicket, tickets_ticket, user_relations_userrelation, wish,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseTransaction, DbConn, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin, ModelTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    Select, Set, TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateTicketParams {
    pub gift_date: NaiveDate,
    pub description: String,
    pub user_relation_id: i64,
    pub is_special: bool,
    pub is_draft: bool,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct UpdateTicketParams {
    pub description: Option<String>,
    pub status: Option<TicketStatus>,
    pub is_special: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct ListTicketsWithWishParams {
    pub text_query: Option<Vec<String>>,
    pub gift_date_gte: Option<NaiveDate>,
    pub gift_date_lte: Option<NaiveDate>,
    pub is_giving: bool,
}

#[derive(Debug, Error)]
pub enum TicketServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("UserRelation not found for id: {0}")]
    UserRelationNotFound(i64),
    #[error("Ticket not found for id: {0}")]
    TicketNotFound(i64),
    #[error("{0}")]
    ValidationError(String),
}

#[derive(Clone)]
pub struct TicketService<'a> {
    pub db: &'a DbConn,
}

impl<'a> TicketService<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }
}

pub trait TicketServiceQuery {
    fn get_ticket_by_id(
        &self,
        user_id: i64,
        ticket_id: i64,
    ) -> impl Future<Output = Result<tickets_ticket::Model, TicketServiceError>>;
    fn get_ticket_with_wish_by_id(
        &self,
        user_id: i64,
        ticket_id: i64,
    ) -> impl Future<Output = Result<(tickets_ticket::Model, Option<wish::Model>), TicketServiceError>>;
    fn get_oldest_available_ticket(
        &self,
        receiving_user_id: i64,
        user_relation_id: i64,
        is_special: bool,
    ) -> impl Future<Output = Result<Option<tickets_ticket::Model>, TicketServiceError>>;
    fn list_tickets_with_wish(
        &self,
        user_id: i64,
        user_relation_id: i64,
        params: ListTicketsWithWishParams,
    ) -> impl Future<Output = Result<Vec<(tickets_ticket::Model, Option<wish::Model>)>, TicketServiceError>>;
    fn check_special_ticket_existence(
        &self,
        giving_user_id: i64,
        user_relation_id: i64,
        date: NaiveDate,
    ) -> impl Future<Output = Result<bool, TicketServiceError>>;
}

impl TicketServiceQuery for TicketService<'_> {
    async fn get_ticket_by_id(
        &self,
        user_id: i64,
        ticket_id: i64,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        get_query_ticket_by_id(user_id, ticket_id)
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound(ticket_id))
    }
    async fn get_ticket_with_wish_by_id(
        &self,
        user_id: i64,
        ticket_id: i64,
    ) -> Result<(tickets_ticket::Model, Option<wish::Model>), TicketServiceError> {
        get_query_ticket_by_id(user_id, ticket_id)
            .join(LeftJoin, tickets_ticket::Relation::Wish.def())
            .select_also(wish::Entity)
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound(ticket_id))
    }
    async fn get_oldest_available_ticket(
        &self,
        receiving_user_id: i64,
        user_relation_id: i64,
        is_special: bool,
    ) -> Result<Option<tickets_ticket::Model>, TicketServiceError> {
        get_query_tickets_with_access_to_user(receiving_user_id)
            .join(LeftJoin, tickets_ticket::Relation::Wish.def())
            .filter(tickets_ticket::Column::GivingUserId.ne(receiving_user_id))
            .filter(tickets_ticket::Column::UserRelationId.eq(user_relation_id))
            .filter(tickets_ticket::Column::IsSpecial.eq(is_special))
            .filter(wish::Column::Id.is_null())
            .order_by_asc(tickets_ticket::Column::GiftDate)
            .one(self.db)
            .await
            .map_err(|e| e.into())
    }

    async fn list_tickets_with_wish(
        &self,
        user_id: i64,
        user_relation_id: i64,
        params: ListTicketsWithWishParams,
    ) -> Result<Vec<(tickets_ticket::Model, Option<wish::Model>)>, TicketServiceError> {
        let mut query = get_query_tickets_with_access_to_user(user_id)
            .filter(tickets_ticket::Column::UserRelationId.eq(user_relation_id))
            .join(LeftJoin, tickets_ticket::Relation::Wish.def());
        if let Some(text_query) = params.text_query {
            let mut cond = Condition::all();
            for text in text_query {
                cond = cond.add(
                    Condition::any()
                        .add(tickets_ticket::Column::Description.contains(&text))
                        .add(wish::Column::Description.contains(&text)),
                )
            }
            query = query.filter(cond);
        }
        if let Some(gift_date_gte) = params.gift_date_gte {
            query = query.filter(tickets_ticket::Column::GiftDate.gte(gift_date_gte));
        }
        if let Some(gift_date_lte) = params.gift_date_lte {
            query = query.filter(tickets_ticket::Column::GiftDate.lte(gift_date_lte));
        }
        if params.is_giving {
            query = query.filter(tickets_ticket::Column::GivingUserId.eq(user_id));
        } else {
            query = query
                .filter(tickets_ticket::Column::GivingUserId.ne(user_id))
                .filter(tickets_ticket::Column::Status.ne(TicketStatus::Draft.to_value()));
        }

        let tickets = query
            .order_by(tickets_ticket::Column::GiftDate, Order::Desc)
            .order_by(tickets_ticket::Column::CreatedAt, Order::Desc)
            .select_also(wish::Entity)
            .all(self.db)
            .await?;

        Ok(tickets)
    }

    async fn check_special_ticket_existence(
        &self,
        giving_user_id: i64,
        user_relation_id: i64,
        date: NaiveDate,
    ) -> Result<bool, TicketServiceError> {
        let start_of_month = match NaiveDate::from_ymd_opt(date.year(), date.month0() + 1, 1) {
            Some(date) => date,
            None => {
                return Err(TicketServiceError::ValidationError(
                    "start_of_month calculation failed".to_string(),
                ))
            }
        };
        let end_of_month = match NaiveDate::from_ymd_opt(date.year(), date.month0() + 2, 1)
            .unwrap_or(NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap())
            .pred_opt()
        {
            Some(date) => date,
            None => {
                return Err(TicketServiceError::ValidationError(
                    "end_of_month calculation failed".to_string(),
                ))
            }
        };
        let count = tickets_ticket::Entity::find()
            .filter(tickets_ticket::Column::GivingUserId.eq(giving_user_id))
            .filter(tickets_ticket::Column::UserRelationId.eq(user_relation_id))
            .filter(tickets_ticket::Column::IsSpecial.eq(true))
            .filter(tickets_ticket::Column::GiftDate.between(start_of_month, end_of_month))
            .count(self.db)
            .await?;
        Ok(count > 0)
    }
}

pub trait TicketServiceMutation {
    fn create_ticket(
        &self,
        user_id: i64,
        params: CreateTicketParams,
    ) -> impl Future<Output = Result<tickets_ticket::Model, TicketServiceError>>;
    fn update_ticket(
        &self,
        ticket: tickets_ticket::Model,
        params: UpdateTicketParams,
    ) -> impl Future<Output = Result<tickets_ticket::Model, TicketServiceError>>;
    fn mark_ticket_read(
        &self,
        ticket: tickets_ticket::Model,
    ) -> impl Future<Output = Result<tickets_ticket::Model, TicketServiceError>>;
    fn delete_ticket(
        &self,
        user_id: i64,
        ticket: tickets_ticket::Model,
    ) -> impl Future<Output = Result<(), TicketServiceError>>;
}

impl TicketServiceMutation for TicketService<'_> {
    async fn create_ticket(
        &self,
        user_id: i64,
        params: CreateTicketParams,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    let user_relation = get_user_relation(txn, user_id, params.user_relation_id).await?;
                    let now = Utc::now();
                    let status = if params.is_draft {
                        TicketStatus::Draft
                    } else {
                        TicketStatus::Unread
                    };
                    let ticket = tickets_ticket::ActiveModel {
                        giving_user_id: Set(user_id),
                        description: Set(params.description),
                        user_relation_id: Set(params.user_relation_id),
                        gift_date: Set(params.gift_date),
                        status: Set(status.to_value()),
                        is_special: Set(params.is_special),
                        created_at: Set(now.into()),
                        updated_at: Set(now.into()),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    if ticket.giving_user_id == user_relation.user_1_id {
                        if user_relation
                            .first_user_1_giving_ticket_date
                            .is_none_or(|date| date > ticket.gift_date)
                        {
                            let mut user_relation = user_relation.into_active_model();
                            user_relation.first_user_1_giving_ticket_date = Set(Some(ticket.gift_date));
                            user_relation.updated_at = Set(now.into());
                            user_relation.update(txn).await?;
                        }
                    } else {
                        if user_relation
                            .first_user_2_giving_ticket_date
                            .is_none_or(|date| date > ticket.gift_date)
                        {
                            let mut user_relation = user_relation.into_active_model();
                            user_relation.first_user_2_giving_ticket_date = Set(Some(ticket.gift_date));
                            user_relation.updated_at = Set(now.into());
                            user_relation.update(txn).await?;
                        }
                    }

                    Ok(ticket)
                })
            })
            .await
            .map_err(parse_transaction_error)
    }

    async fn update_ticket(
        &self,
        ticket: tickets_ticket::Model,
        params: UpdateTicketParams,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        let mut ticket = ticket.into_active_model();
        if let Some(description) = params.description {
            ticket.description = Set(description);
        };
        if let Some(status) = params.status {
            ticket.status = Set(status.to_value());
        };
        if let Some(is_special) = params.is_special {
            ticket.is_special = Set(is_special);
        }
        ticket.updated_at = Set(Utc::now().into());
        let ticket = ticket.update(self.db).await?;

        Ok(ticket)
    }

    async fn mark_ticket_read(
        &self,
        ticket: tickets_ticket::Model,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        let mut ticket = ticket.into_active_model();
        ticket.status = Set(TicketStatus::Read.to_value());
        ticket.updated_at = Set(Utc::now().into());
        let ticket = ticket.update(self.db).await?;

        Ok(ticket)
    }

    async fn delete_ticket(&self, user_id: i64, ticket: tickets_ticket::Model) -> Result<(), TicketServiceError> {
        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    let user_relation = get_user_relation(txn, user_id, ticket.user_relation_id).await?;
                    let gift_date = ticket.gift_date;
                    let giving_user_id = ticket.giving_user_id;
                    ticket.delete(txn).await?;

                    if giving_user_id == user_relation.user_1_id {
                        if user_relation
                            .first_user_1_giving_ticket_date
                            .is_some_and(|date| date == gift_date)
                        {
                            let oldest_ticket = tickets_ticket::Entity::find()
                                .filter(tickets_ticket::Column::UserRelationId.eq(user_relation.id))
                                .filter(tickets_ticket::Column::GivingUserId.eq(user_id))
                                .order_by(tickets_ticket::Column::GiftDate, Order::Asc)
                                .one(txn)
                                .await?;
                            let mut user_relation = user_relation.into_active_model();
                            user_relation.first_user_1_giving_ticket_date =
                                Set(oldest_ticket.and_then(|t| Some(t.gift_date)));
                            user_relation.updated_at = Set(Utc::now().into());
                            user_relation.update(txn).await?;
                        }
                    } else {
                        if user_relation
                            .first_user_2_giving_ticket_date
                            .is_some_and(|date| date == gift_date)
                        {
                            let oldest_ticket = tickets_ticket::Entity::find()
                                .filter(tickets_ticket::Column::UserRelationId.eq(user_relation.id))
                                .filter(tickets_ticket::Column::GivingUserId.eq(user_id))
                                .order_by(tickets_ticket::Column::GiftDate, Order::Asc)
                                .one(txn)
                                .await?;
                            let mut user_relation = user_relation.into_active_model();
                            user_relation.first_user_2_giving_ticket_date =
                                Set(oldest_ticket.and_then(|t| Some(t.gift_date)));
                            user_relation.updated_at = Set(Utc::now().into());
                            user_relation.update(txn).await?;
                        }
                    }

                    Ok(())
                })
            })
            .await
            .map_err(parse_transaction_error)
    }
}

fn parse_transaction_error(e: TransactionError<TicketServiceError>) -> TicketServiceError {
    match e {
        TransactionError::Connection(e) => e.into(),
        TransactionError::Transaction(e) => e,
    }
}

async fn get_user_relation(
    txn: &DatabaseTransaction,
    user_id: i64,
    user_relation_id: i64,
) -> Result<user_relations_userrelation::Model, TicketServiceError> {
    user_relations_userrelation::Entity::find_by_id(user_relation_id)
        .filter(
            Condition::any()
                .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
        )
        .one(txn)
        .await?
        .ok_or(TicketServiceError::UserRelationNotFound(user_relation_id))
}

fn get_query_tickets_with_access_to_user(user_id: i64) -> Select<TicketsTicket> {
    tickets_ticket::Entity::find()
        .join(LeftJoin, tickets_ticket::Relation::UserRelationsUserrelation.def())
        .filter(
            Condition::any()
                .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
        )
}
fn get_query_ticket_by_id(user_id: i64, ticket_id: i64) -> Select<TicketsTicket> {
    get_query_tickets_with_access_to_user(user_id).filter(tickets_ticket::Column::Id.eq(ticket_id))
}
