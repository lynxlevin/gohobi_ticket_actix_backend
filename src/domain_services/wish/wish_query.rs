use std::future::Future;

use entities::{
    tickets_ticket as ticket,
    user_relations_userrelation::{self as user_relation, UserRelationId},
    users_user::UserId,
    wish::{Column, Entity, Model, Relation},
    wish_reply,
};
use sea_orm::{
    ColumnTrait, Condition, EntityLoaderTrait, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder,
    QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::wish::{WishService, WishServiceError};

pub struct ListWishesParam {
    pub page: u64,
    pub limit: u64,
}

pub struct ListWishesResponseItem {
    pub wish: Model,
    pub ticket: ticket::Model,
    pub has_replies: bool,
}

pub struct ListWishesResponse {
    pub wishes: Vec<ListWishesResponseItem>,
    pub wish_count: u64,
    pub page_count: u64,
}

pub trait WishServiceQuery {
    fn list_wishes(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
    ) -> impl Future<Output = Result<Vec<ListWishesResponseItem>, WishServiceError>>;
    fn list_wishes_with_total_count(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        params: ListWishesParam,
    ) -> impl Future<Output = Result<ListWishesResponse, WishServiceError>>;
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
    ) -> Result<Vec<ListWishesResponseItem>, WishServiceError> {
        let user_relation = user_relation::Entity::find_by_id(user_relation_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(WishServiceError::UserRelationNotFound())?;

        let wishes = Entity::load()
            .with(ticket::Entity)
            .with(wish_reply::Entity)
            .filter(Column::UserRelationId.eq(user_relation.id))
            .order_by_desc(Column::CreatedAt)
            .all(self.db)
            .await?;

        Ok(wishes
            .into_iter()
            .filter(|wish| (wish.ticket.is_loaded() && !wish.ticket.is_none()) && wish.replies.is_loaded())
            .map(|wish| {
                let ticket = wish.clone().ticket.unwrap();
                let has_replies = wish.replies.iter().count() > 0;
                ListWishesResponseItem { wish: wish.into(), ticket: ticket.into(), has_replies }
            })
            .collect())
    }

    async fn list_wishes_with_total_count(
        &self,
        user_id: UserId,
        user_relation_id: UserRelationId,
        params: ListWishesParam,
    ) -> Result<ListWishesResponse, WishServiceError> {
        let user_relation = user_relation::Entity::find_by_id(user_relation_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(WishServiceError::UserRelationNotFound())?;

        let query = Entity::load()
            .with(ticket::Entity)
            .with(wish_reply::Entity)
            .filter(Column::UserRelationId.eq(user_relation.id))
            .order_by_desc(Column::CreatedAt)
            .paginate(self.db, params.limit);

        let count = query.num_items_and_pages().await?;
        let wishes = query.fetch_page(params.page).await?;

        Ok(ListWishesResponse {
            wishes: wishes
                .into_iter()
                .filter(|wish| (wish.ticket.is_loaded() && !wish.ticket.is_none()) && wish.replies.is_loaded())
                .map(|wish| {
                    let ticket = wish.clone().ticket.unwrap();
                    let has_replies = wish.replies.iter().count() > 0;
                    ListWishesResponseItem { wish: wish.into(), ticket: ticket.into(), has_replies }
                })
                .collect(),
            wish_count: count.number_of_items,
            page_count: count.number_of_pages,
        })
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
