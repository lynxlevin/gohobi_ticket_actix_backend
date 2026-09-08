use std::future::Future;

use entities::{
    user_relations_userrelation as user_relation,
    users_user::UserId,
    wish,
    wish_reply::{ActiveModel, Model},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityLoaderTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::wish_reply::{
    WishReplyService,
    WishReplyServiceError::{self},
};

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct CreateWishReplyParams {
    pub wish_id: Uuid,
    pub description: String,
}

pub trait WishReplyServiceMutation {
    fn create_wish_reply(
        &self,
        user_id: UserId,
        params: CreateWishReplyParams,
    ) -> impl Future<Output = Result<(Model, wish::Model, user_relation::Model), WishReplyServiceError>>;
}

impl WishReplyServiceMutation for WishReplyService<'_> {
    async fn create_wish_reply(
        &self,
        user_id: UserId,
        params: CreateWishReplyParams,
    ) -> Result<(Model, wish::Model, user_relation::Model), WishReplyServiceError> {
        // MYMEMO: ListWishでreplyのあるやつかないやつかわかるようにしたい。クエリするか、wish.has_repliesみたいなのを加えるか？
        let wish = wish::Entity::load()
            .with(user_relation::Entity)
            .filter_by_id(params.wish_id)
            .filter(
                Condition::any()
                    .add(user_relation::Column::User1Id.eq(user_id))
                    .add(user_relation::Column::User2Id.eq(user_id)),
            )
            .one(self.db)
            .await?
            .ok_or(WishReplyServiceError::WishNotFound())?;

        let user_relation = wish
            .clone()
            .user_relation
            .into_option()
            .ok_or(WishReplyServiceError::UserRelationNotFound())?;

        let wish_reply = ActiveModel {
            description: Set(params.description),
            wish_id: Set(params.wish_id),
            posted_by_id: Set(user_id),
            ..Default::default()
        }
        .insert(self.db)
        .await?;

        Ok((wish_reply, wish.into(), user_relation.into()))
    }
}
