use std::collections::HashMap;

use entities::{
    users_user::UserId,
    wish_reply::{ActiveModel, Entity, Model},
};
use sea_orm::{DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::db::Db;

pub fn wish_reply(wish_id: Uuid, user_id: UserId) -> ActiveModel {
    ActiveModel {
        description: Set("reply".to_string()),
        wish_id: Set(wish_id),
        posted_by_id: Set(user_id),
        ..Default::default()
    }
}

pub trait WishReplyFactory {
    fn description(self, description: String) -> ActiveModel;
}

impl WishReplyFactory for ActiveModel {
    fn description(mut self, description: String) -> ActiveModel {
        self.description = Set(description);
        self
    }
}

#[derive(Default)]
pub struct WishReplyParam<'a> {
    pub name: &'a str,
    pub wish_id: Uuid,
    pub posted_by_id: UserId,
}

pub async fn create_wish_replies(
    params: Vec<WishReplyParam<'_>>,
    db: &Db,
) -> Result<HashMap<String, Model>, DbErr> {
    let wish_replies = Entity::insert_many(
        params
            .iter()
            .map(|param| wish_reply(param.wish_id, param.posted_by_id).description(param.name.to_string())),
    )
    .exec_with_returning(&db.db)
    .await?;

    Ok(wish_replies
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (ticket, param)| {
            acc.entry(param.name.to_string()).or_insert(ticket);
            acc
        }))
}
