use entities::users_user;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter};

pub struct UserQuery<'a> {
    pub db: &'a DbConn,
}

impl UserQuery<'_> {
    pub async fn find_by_id(self, id: i64) -> Result<Option<users_user::Model>, DbErr> {
        users_user::Entity::find_by_id(id).one(self.db).await
    }

    pub async fn find_active_by_email(
        self,
        email: String,
    ) -> Result<Option<users_user::Model>, DbErr> {
        users_user::Entity::find()
            .filter(users_user::Column::Email.eq(email))
            .one(self.db)
            .await
    }
}
