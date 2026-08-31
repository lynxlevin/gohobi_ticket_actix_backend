use std::future::Future;

use chrono::{Datelike, NaiveDate};
use entities::{
    prelude::TicketsTicket,
    tickets_ticket::{self, TicketId, TicketStatus},
    user_relations_userrelation::{self, UserRelationId},
    users_user::UserId,
    wish,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, Order, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait, Select,
};
use serde::{Deserialize, Serialize};

use crate::ticket::{TicketService, TicketServiceError};

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct ListTicketsWithWishParams {
    pub text_query: Option<Vec<String>>,
    pub gift_date_gte: Option<NaiveDate>,
    pub gift_date_lte: Option<NaiveDate>,
    pub is_giving: bool,
}

pub trait TicketServiceQuery {
    fn get_ticket_by_id(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
    ) -> impl Future<Output = Result<tickets_ticket::Model, TicketServiceError>>;
    fn get_ticket_with_wish_by_id(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
    ) -> impl Future<Output = Result<(tickets_ticket::Model, Option<wish::Model>), TicketServiceError>>;
    fn get_oldest_available_ticket(
        &self,
        receiving_user_id: UserId,
        user_relation_id: UserRelationId,
        is_special: bool,
    ) -> impl Future<Output = Result<Option<tickets_ticket::Model>, TicketServiceError>>;
    fn list_tickets_with_wish(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        params: ListTicketsWithWishParams,
    ) -> impl Future<Output = Result<Vec<(tickets_ticket::Model, Option<wish::Model>)>, TicketServiceError>>;
    fn check_special_ticket_existence(
        &self,
        giving_user_id: UserId,
        user_relation_id: UserRelationId,
        date: NaiveDate,
    ) -> impl Future<Output = Result<bool, TicketServiceError>>;
}

impl TicketServiceQuery for TicketService<'_> {
    async fn get_ticket_by_id(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
    ) -> Result<tickets_ticket::Model, TicketServiceError> {
        get_query_ticket_by_id(user_id, ticket_id)
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound())
    }
    async fn get_ticket_with_wish_by_id(
        &self,
        user_id: UserId,
        ticket_id: TicketId,
    ) -> Result<(tickets_ticket::Model, Option<wish::Model>), TicketServiceError> {
        get_query_ticket_by_id(user_id, ticket_id)
            .join(LeftJoin, tickets_ticket::Relation::Wish.def())
            .select_also(wish::Entity)
            .one(self.db)
            .await?
            .ok_or(TicketServiceError::TicketNotFound())
    }
    async fn get_oldest_available_ticket(
        &self,
        receiving_user_id: UserId,
        user_relation_id: UserRelationId,
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
        user_id: UserId,
        user_relation_id: UserRelationId,
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
                .filter(tickets_ticket::Column::Status.ne(TicketStatus::Draft));
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
        giving_user_id: UserId,
        user_relation_id: UserRelationId,
        date: NaiveDate,
    ) -> Result<bool, TicketServiceError> {
        let start_of_month = NaiveDate::from_ymd_opt(date.year(), date.month0() + 1, 1).ok_or(
            TicketServiceError::ValidationError("start_of_month calculation failed".to_string()),
        )?;
        let end_of_month = NaiveDate::from_ymd_opt(date.year(), date.month0() + 2, 1)
            .unwrap_or(NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap())
            .pred_opt()
            .ok_or(TicketServiceError::ValidationError(
                "end_of_month calculation failed".to_string(),
            ))?;
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

fn get_query_tickets_with_access_to_user(user_id: UserId) -> Select<TicketsTicket> {
    tickets_ticket::Entity::find()
        .join(LeftJoin, tickets_ticket::Relation::UserRelationsUserrelation.def())
        .filter(
            Condition::any()
                .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
        )
}
fn get_query_ticket_by_id(user_id: UserId, ticket_id: TicketId) -> Select<TicketsTicket> {
    get_query_tickets_with_access_to_user(user_id).filter(tickets_ticket::Column::Id.eq(ticket_id))
}
