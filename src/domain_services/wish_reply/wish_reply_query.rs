use std::future::Future;

use entities::{
    user_relations_userrelation,
    users_user::UserId,
    wish,
    wish_reply::{Column, Entity, Model, Relation},
};
use sea_orm::{ColumnTrait, Condition, EntityTrait, JoinType::LeftJoin, QueryFilter, QuerySelect, RelationTrait};

use crate::wish_reply::{WishReplyService, WishReplyServiceError};

pub trait WishReplyServiceQuery {
    fn get_newest(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<(Model, wish::Model), WishReplyServiceError>>;
}

impl WishReplyServiceQuery for WishReplyService<'_> {
    async fn get_newest(&self, user_id: UserId) -> Result<(Model, wish::Model), WishReplyServiceError> {
        let (wish_reply, wish) = Entity::find()
            .join(LeftJoin, Relation::Wish.def())
            .join(LeftJoin, wish::Relation::UserRelationsUserrelation.def())
            .filter(Column::PostedById.ne(user_id))
            .filter(
                Condition::any()
                    .add(user_relations_userrelation::Column::User1Id.eq(user_id))
                    .add(user_relations_userrelation::Column::User2Id.eq(user_id)),
            )
            .order_by_id_desc()
            .select_two_required(wish::Entity)
            .one(self.db)
            .await?
            .ok_or(WishReplyServiceError::WishReplyNotFound())?;

        Ok((wish_reply, wish))
    }
}
