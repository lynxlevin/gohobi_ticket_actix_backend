use std::future::Future;

use chrono::{DateTime, FixedOffset};
use entities::{
    tickets_ticket as ticket,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
    wish::{Column, Entity, Model, Relation},
    wish_reply,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::wish::{WishService, WishServiceError};

#[derive(Deserialize, Default, Debug)]
pub struct ListWishesParam {
    pub created_at_gte: Option<DateTime<FixedOffset>>,
    pub created_at_lte: Option<DateTime<FixedOffset>>,
    pub created_at_lt: Option<DateTime<FixedOffset>>,
}

pub trait WishServiceQuery {
    fn list_wishes(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        params: ListWishesParam,
    ) -> impl Future<Output = Result<Vec<(Model, ticket::Model, bool)>, WishServiceError>>;
    fn get_with_ticket_and_replies(
        &self,
        user_id: UserId,
        wish_id: Uuid,
    ) -> impl Future<Output = Result<(Model, ticket::Model, Vec<wish_reply::Model>), WishServiceError>>;
}

impl WishServiceQuery for WishService<'_> {
    async fn list_wishes(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        params: ListWishesParam,
    ) -> Result<Vec<(Model, ticket::Model, bool)>, WishServiceError> {
        let user_relation = user_relation::Entity::find_by_id(user_relation_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(WishServiceError::UserRelationNotFound())?;

        let mut query = Entity::load()
            .with(ticket::Entity)
            .with(wish_reply::Entity)
            .filter(Column::UserRelationId.eq(user_relation.id));

        if let Some(created_at_gte) = params.created_at_gte {
            query = query.filter(Column::CreatedAt.gte(created_at_gte));
        }
        if let Some(created_at_lte) = params.created_at_lte {
            query = query.filter(Column::CreatedAt.lte(created_at_lte));
        }
        if let Some(created_at_lt) = params.created_at_lt {
            query = query.filter(Column::CreatedAt.lt(created_at_lt));
        }

        let wishes = query.order_by_desc(Column::CreatedAt).all(self.db).await?;

        Ok(wishes
            .into_iter()
            .filter(|wish| (wish.ticket.is_loaded() && !wish.ticket.is_none()) && wish.replies.is_loaded())
            .map(|wish| {
                let ticket = wish.clone().ticket.unwrap();
                let reply_count = wish.replies.iter().count();
                (wish.into(), ticket.into(), reply_count > 0)
            })
            .collect())
    }

    async fn get_with_ticket_and_replies(
        &self,
        user_id: UserId,
        wish_id: Uuid,
    ) -> Result<(Model, ticket::Model, Vec<wish_reply::Model>), WishServiceError> {
        let (wish, ticket) = Entity::find_by_id(wish_id)
            .join(LeftJoin, Relation::TicketsTicket.def())
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .select_also(ticket::Entity)
            .one(self.db)
            .await?
            .ok_or(WishServiceError::WishNotFound())?;
        let ticket = ticket.ok_or(WishServiceError::TicketNotFound())?;

        let replies = wish_reply::Entity::find()
            .filter(wish_reply::Column::WishId.eq(wish.id))
            .order_by_asc(wish_reply::Column::CreatedAt)
            .all(self.db)
            .await?;

        Ok((wish, ticket, replies))
    }
}
