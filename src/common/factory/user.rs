use entities::users_user;
use sea_orm::Set;
use uuid::Uuid;

pub fn user() -> users_user::ActiveModel {
    users_user::ActiveModel {
        password: Set("password".to_string()),
        username: Set("Lynx Levin".to_string()),
        email: Set(format!("{}@test.com", Uuid::now_v7().to_string())),
        ..Default::default()
    }
}

pub trait UserFactory {
    fn password(self, hashed_password: &str) -> users_user::ActiveModel;
}

impl UserFactory for users_user::ActiveModel {
    fn password(mut self, hashed_password: &str) -> users_user::ActiveModel {
        self.password = Set(hashed_password.to_string());
        self
    }
}
