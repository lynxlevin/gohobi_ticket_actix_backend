use entities::users_user;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserVisible {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<users_user::Model> for UserVisible {
    fn from(value: users_user::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            email: value.email,
        }
    }
}
