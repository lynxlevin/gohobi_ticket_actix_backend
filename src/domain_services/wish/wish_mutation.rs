use std::future::Future;

use chrono::Utc;
use entities::{
    user_relations_userrelation::{self as user_relation},
    users_user::UserId,
    wish::{Entity, Relation},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, EntityTrait, IntoActiveModel, JoinType::LeftJoin,
    QueryFilter, QuerySelect, RelationTrait,
};
use uuid::Uuid;

use crate::wish::{WishService, WishServiceError};

pub trait WishServiceMutation {
    fn update_reactions(
        &self,
        user_id: UserId,
        wish_id: Uuid,
        reactions: String,
    ) -> impl Future<Output = Result<(), WishServiceError>>;
}

impl WishServiceMutation for WishService<'_> {
    async fn update_reactions(
        &self,
        user_id: UserId,
        wish_id: Uuid,
        reactions: String,
    ) -> Result<(), WishServiceError> {
        let wish = Entity::find_by_id(wish_id)
            .join(LeftJoin, Relation::UserRelationsUserrelation.def())
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(WishServiceError::WishNotFound())?;

        let mut wish = wish.into_active_model();
        wish.reactions = Set(reactions);
        wish.updated_at = Set(Utc::now().into());
        wish.save(self.db).await?;

        Ok(())
    }
}
