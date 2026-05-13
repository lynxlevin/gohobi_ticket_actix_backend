use chrono::{Datelike, NaiveDate, Utc};
use entities::{custom_types::TicketStatus, tickets_ticket, user_relations_userrelation};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DbConn, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, Set, TransactionError, TransactionTrait,
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

#[derive(Debug, Error)]
pub enum TicketServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("UserRelation not found for id: {0}")]
    UserRelationNotFound(i64),
    #[error("{0}")]
    ValidationError(String),
}

pub struct TicketService<'a> {
    pub db: &'a DbConn,
}

fn parse_transaction_error(e: TransactionError<TicketServiceError>) -> TicketServiceError {
    match e {
        TransactionError::Connection(e) => e.into(),
        TransactionError::Transaction(e) => e,
    }
}

impl<'a> TicketService<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self { db }
    }

    pub async fn check_special_ticket_existence(
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

    pub async fn create_ticket(
        self,
        user_id: i64,
        params: CreateTicketParams,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        self.db
            .transaction(|txn| {
                Box::pin(async move {
                    // MYMEMO: execute inside transaction
                    let user_relation =
                        user_relations_userrelation::Entity::find_by_id(params.user_relation_id)
                            .filter(
                                Condition::any()
                                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
                            )
                            .one(txn)
                            .await?
                            .ok_or(TicketServiceError::UserRelationNotFound(
                                params.user_relation_id,
                            ))?;
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
                            user_relation.first_user_1_giving_ticket_date =
                                Set(Some(ticket.gift_date));
                            user_relation.updated_at = Set(now.into());
                            user_relation.update(txn).await?;
                        }
                    } else {
                        if user_relation
                            .first_user_2_giving_ticket_date
                            .is_none_or(|date| date > ticket.gift_date)
                        {
                            let mut user_relation = user_relation.into_active_model();
                            user_relation.first_user_2_giving_ticket_date =
                                Set(Some(ticket.gift_date));
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

    // pub async fn update(
    //     self,
    //     ticket: tickets_ticket::Model,
    //     params: UpdateTicketParams,
    // ) -> Result<tickets_ticket::Model, DbErr> {
    //     let mut ticket = ticket.into_active_model();
    //     if let Some(description) = params.description {
    //         ticket.description = Set(description);
    //     };
    //     if let Some(status) = params.status {
    //         ticket.status = Set(status.to_value());
    //     };
    //     if let Some(is_special) = params.is_special {
    //         ticket.is_special = Set(is_special);
    //     };
    //     ticket.updated_at = Set(Utc::now().into());
    //     ticket.update(self.db).await
    // }

    // pub async fn delete(self, ticket: tickets_ticket::Model) -> Result<(), DbErr> {
    //     ticket.delete(self.db).await.map(|_| ())
    // }
}
