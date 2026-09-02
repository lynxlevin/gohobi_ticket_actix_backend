use std::future::Future;

use chrono::{Datelike, NaiveDate, Utc};
use entities::{
    tickets_ticket::{ActiveModel, Column, Entity, Model, Relation, TicketId, TicketStatus},
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, DbErr, EntityTrait,
    IntoActiveModel, JoinType::LeftJoin, ModelTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::ticket::{TicketService, TicketServiceError};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateTicketParams {
    pub gift_date: NaiveDate,
    pub description: String,
    pub user_relation_id: UserRelationId,
    pub is_special: bool,
    pub is_draft: bool,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct UpdateTicketParams {
    pub description: String,
    pub status: TicketStatus,
    pub is_special: bool,
}

pub trait TicketServiceMutation {
    fn create_ticket(
        &self,
        user_id: UserId,
        params: CreateTicketParams,
    ) -> impl Future<Output = Result<Model, TicketServiceError>>;
    fn update_ticket(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
        params: UpdateTicketParams,
    ) -> impl Future<Output = Result<Model, TicketServiceError>>;
    fn mark_ticket_read(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
    ) -> impl Future<Output = Result<Model, TicketServiceError>>;
    fn delete_ticket(
        &self,
        user_id: UserId,
        ticket: Model,
    ) -> impl Future<Output = Result<(), TicketServiceError>>;
}

impl TicketServiceMutation for TicketService<'_> {
    async fn create_ticket(
        &self,
        creator_id: UserId,
        params: CreateTicketParams,
    ) -> Result<Model, TicketServiceError> {
        if params.is_special {
            // MYMEMO: code is same as check_special_ticket_existence.
            let start_of_month =
                NaiveDate::from_ymd_opt(params.gift_date.year(), params.gift_date.month0() + 1, 1).ok_or(
                    TicketServiceError::ValidationError("start_of_month calculation failed".to_string()),
                )?;
            let end_of_month = NaiveDate::from_ymd_opt(params.gift_date.year(), params.gift_date.month0() + 2, 1)
                .unwrap_or(NaiveDate::from_ymd_opt(params.gift_date.year() + 1, 1, 1).unwrap())
                .pred_opt()
                .ok_or(TicketServiceError::ValidationError(
                    "end_of_month calculation failed".to_string(),
                ))?;
            let existing_special_ticket_count = Entity::find()
                .filter(Column::GivingUserId.eq(creator_id))
                .filter(Column::UserRelationId.eq(params.user_relation_id))
                .filter(Column::IsSpecial.eq(true))
                .filter(Column::GiftDate.between(start_of_month, end_of_month))
                .count(self.db)
                .await?;
            if existing_special_ticket_count > 0 {
                return Err(TicketServiceError::ValidationError(
                    "A SpecialTicket already exists for the month.".to_string(),
                ));
            }
        }

        let user_relation = user_relation::Entity::find_by_id(params.user_relation_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(creator_id))
                    .add(user_relation::Column::User2Id.eq(creator_id)),
            )
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::UserRelationNotFound())?;
        let creator_is_user_1 = user_relation.user_1_id == creator_id;
        let now = Utc::now();

        let ticket = self
            .db
            .transaction(|txn| {
                Box::pin(async move {
                    let ticket = ActiveModel {
                        giving_user_id: Set(creator_id),
                        description: Set(params.description),
                        user_relation_id: Set(params.user_relation_id),
                        gift_date: Set(params.gift_date),
                        status: Set(match params.is_draft {
                            true => TicketStatus::Draft,
                            false => TicketStatus::Unread,
                        }),
                        is_special: Set(params.is_special),
                        created_at: Set(now.into()),
                        updated_at: Set(now.into()),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    if creator_is_user_1 {
                        if user_relation
                            .first_user_1_giving_ticket_date
                            .is_none_or(|date| date > ticket.gift_date)
                        {
                            update_first_user_1_giving_ticket_date(txn, user_relation, ticket.gift_date).await?;
                        }
                    } else {
                        if user_relation
                            .first_user_2_giving_ticket_date
                            .is_none_or(|date| date > ticket.gift_date)
                        {
                            update_first_user_2_giving_ticket_date(txn, user_relation, ticket.gift_date).await?;
                        }
                    }

                    Ok(ticket)
                })
            })
            .await?;
        Ok(ticket)
    }

    async fn update_ticket(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
        params: UpdateTicketParams,
    ) -> Result<Model, TicketServiceError> {
        let ticket = Entity::find_by_id(ticket_id)
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound())?;

        if ticket.giving_user_id != user_id {
            return Err(TicketServiceError::NotGivingTicket());
        }
        if params.status == TicketStatus::Draft && ticket.status.is_published() {
            return Err(TicketServiceError::ValidationError(
                "This ticket cannot be turned back to draft state.".to_string(),
            ));
        };
        let status = if ticket.status == TicketStatus::Read && params.description != ticket.description {
            TicketStatus::Edited
        } else {
            params.status
        };

        let mut ticket = ticket.into_active_model();
        ticket.description = Set(params.description);
        ticket.status = Set(status);
        ticket.is_special = Set(params.is_special);
        ticket.updated_at = Set(Utc::now().into());
        let ticket = ticket.update(self.db).await?;

        Ok(ticket)
    }

    async fn mark_ticket_read(&self, user_id: UserId, ticket_id: TicketId) -> Result<Model, TicketServiceError> {
        let ticket = Entity::find_by_id(ticket_id)
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .filter(Column::Status.ne(TicketStatus::Draft))
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound())?;

        if ticket.giving_user_id == user_id {
            return Err(TicketServiceError::NotReceivingTicket());
        }

        let mut ticket = ticket.into_active_model();
        ticket.status = Set(TicketStatus::Read);
        ticket.updated_at = Set(Utc::now().into());
        let ticket = ticket.update(self.db).await?;

        Ok(ticket)
    }

    async fn delete_ticket(&self, user_id: UserId, ticket: Model) -> Result<(), TicketServiceError> {
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
                            let oldest_ticket = Entity::find()
                                .filter(Column::UserRelationId.eq(user_relation.id))
                                .filter(Column::GivingUserId.eq(user_id))
                                .order_by(Column::GiftDate, Order::Asc)
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
                            let oldest_ticket = Entity::find()
                                .filter(Column::UserRelationId.eq(user_relation.id))
                                .filter(Column::GivingUserId.eq(user_id))
                                .order_by(Column::GiftDate, Order::Asc)
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
            .map_err(|e| e.into())
    }
}

async fn get_user_relation(
    txn: &DatabaseTransaction,
    user_id: UserId,
    user_relation_id: UserRelationId,
) -> Result<user_relation::Model, TicketServiceError> {
    user_relation::Entity::find_by_id(user_relation_id)
        .filter(
            Condition::any()
                .add(user_relation::Column::User1Id.eq(user_id))
                .add(user_relation::Column::User2Id.eq(user_id)),
        )
        .one(txn)
        .await?
        .ok_or(TicketServiceError::UserRelationNotFound())
}

async fn update_first_user_1_giving_ticket_date<T: ConnectionTrait>(
    db: &T,
    user_relation: user_relation::Model,
    date: NaiveDate,
) -> Result<user_relation::Model, DbErr> {
    let mut user_relation = user_relation.into_active_model();
    user_relation.first_user_1_giving_ticket_date = Set(Some(date));
    user_relation.updated_at = Set(Utc::now().into());
    user_relation.update(db).await
}
async fn update_first_user_2_giving_ticket_date<T: ConnectionTrait>(
    db: &T,
    user_relation: user_relation::Model,
    date: NaiveDate,
) -> Result<user_relation::Model, DbErr> {
    let mut user_relation = user_relation.into_active_model();
    user_relation.first_user_2_giving_ticket_date = Set(Some(date));
    user_relation.updated_at = Set(Utc::now().into());
    user_relation.update(db).await
}
