use std::future::Future;

use entities::{
    tickets_ticket as ticket, user_relations_userrelation as user_relation,
    users_user::UserId,
    wish::{Entity, Model, Relation},
    wish_reply,
};
use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::wish::{WishService, WishServiceError};

pub trait WishServiceQuery {
    fn get_with_ticket_and_replies(
        &self,
        user_id: UserId,
        wish_id: Uuid,
    ) -> impl Future<Output = Result<(Model, ticket::Model, Vec<wish_reply::Model>), WishServiceError>>;
}

impl WishServiceQuery for WishService<'_> {
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
