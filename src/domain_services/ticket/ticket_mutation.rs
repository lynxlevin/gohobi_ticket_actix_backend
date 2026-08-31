use std::future::Future;

use chrono::{NaiveDate, Utc};
use entities::{
    tickets_ticket::{self, TicketStatus},
    user_relations_userrelation::{self, UserRelationId},
    users_user::UserId,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseTransaction, EntityTrait, IntoActiveModel, ModelTrait,
    Order, QueryFilter, QueryOrder, Set, TransactionTrait,
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
    pub description: Option<String>,
    pub status: Option<TicketStatus>,
    pub is_special: Option<bool>,
}

pub trait TicketServiceMutation {
    fn create_ticket(
        &self,
        user_id: UserId,
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
        user_id: UserId,
        ticket: tickets_ticket::Model,
    ) -> impl Future<Output = Result<(), TicketServiceError>>;
}

impl TicketServiceMutation for TicketService<'_> {
    async fn create_ticket(
        &self,
        user_id: UserId,
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
                        status: Set(status),
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
            .map_err(|e| e.into())
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
            ticket.status = Set(status);
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
        ticket.status = Set(TicketStatus::Read);
        ticket.updated_at = Set(Utc::now().into());
        let ticket = ticket.update(self.db).await?;

        Ok(ticket)
    }

    async fn delete_ticket(
        &self,
        user_id: UserId,
        ticket: tickets_ticket::Model,
    ) -> Result<(), TicketServiceError> {
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
            .map_err(|e| e.into())
    }
}

async fn get_user_relation(
    txn: &DatabaseTransaction,
    user_id: UserId,
    user_relation_id: UserRelationId,
) -> Result<user_relations_userrelation::Model, TicketServiceError> {
    user_relations_userrelation::Entity::find_by_id(user_relation_id)
        .filter(
            Condition::any()
                .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
        )
        .one(txn)
        .await?
        .ok_or(TicketServiceError::UserRelationNotFound())
}
