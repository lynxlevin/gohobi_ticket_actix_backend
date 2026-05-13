use chrono::Utc;
use entities::{
    custom_types::WishStatus,
    wish::{ActiveModel, Model},
};
use sea_orm::{ActiveModelTrait, DbConn, DbErr, Set};
use uuid::Uuid;

use crate::ticket::types::CreateWishParams;

pub struct WishMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> WishMutation<'a> {
    pub async fn create(self, params: CreateWishParams) -> Result<Model, DbErr> {
        let now = Utc::now();
        let wish = ActiveModel {
            id: Set(Uuid::now_v7()),
            description: Set(params.use_description),
            ticket_id: Set(params.ticket_id),
            user_relation_id: Set(params.user_relation_id),
            status: Set(WishStatus::Unread.to_value()),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };
        wish.insert(self.db).await
    }
}
