use common::db::Db;
use entities::{
    users_user,
    web_push_subscription::{Column, Entity, Model},
};
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Select};

#[derive(Clone)]
pub struct WebPushSubscriptionQuery<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> WebPushSubscriptionQuery<'a> {
    pub fn init_query(db: &'a Db) -> Self {
        Self { db: &db.db, query: Entity::find() }
    }

    pub async fn get_by_user(self, user: &users_user::Model) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::UserId.eq(user.id)).one(self.db).await
    }

    pub async fn get_by_user_id(self, user_id: i64) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::UserId.eq(user_id)).one(self.db).await
    }
}
