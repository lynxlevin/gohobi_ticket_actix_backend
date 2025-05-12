use entities::users_user;
use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter};

pub struct UserQuery;

impl UserQuery {
    pub async fn find_active_by_email(
        db: &DbConn,
        email: String,
    ) -> Result<Option<users_user::Model>, DbErr> {
        users_user::Entity::find()
            .filter(users_user::Column::Email.eq(email))
            .one(db)
            .await
    }
}
