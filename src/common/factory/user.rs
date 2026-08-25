use entities::users_user;
use sea_orm::{DbConn, DbErr, EntityTrait, Set};
use uuid::Uuid;

pub async fn get_users(db: &DbConn) -> Result<[users_user::Model; 3], DbErr> {
    let user_0 = user().username("user_0");
    let user_1 = user().username("user_1");
    let user_2 = user().username("user_2");
    let users = users_user::Entity::insert_many([user_0, user_1, user_2])
        .exec_with_returning(db)
        .await?;
    Ok(users.try_into().unwrap())
}

pub fn user() -> users_user::ActiveModel {
    users_user::ActiveModel {
        password: Set("password".to_string()),
        username: Set("Lynx Levin".to_string()),
        email: Set(format!("{}@test.com", Uuid::now_v7().to_string())),
        ..Default::default()
    }
}

pub trait UserFactory {
    fn username(self, name: &str) -> users_user::ActiveModel;
    fn password(self, hashed_password: &str) -> users_user::ActiveModel;
}

impl UserFactory for users_user::ActiveModel {
    fn username(mut self, name: &str) -> users_user::ActiveModel {
        self.username = Set(name.to_string());
        self
    }

    fn password(mut self, hashed_password: &str) -> users_user::ActiveModel {
        self.password = Set(hashed_password.to_string());
        self
    }
}
