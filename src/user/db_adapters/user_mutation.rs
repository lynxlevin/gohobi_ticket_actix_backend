use entities::users_user;
use sea_orm::{ActiveModelTrait, DbConn, IntoActiveModel, Set};

use crate::password_util;

pub struct UserMutation<'a> {
    pub db: &'a DbConn,
}

impl UserMutation<'_> {
    pub async fn convert_to_argon2_password(
        self,
        user: users_user::Model,
        password: &str,
    ) -> Result<users_user::Model, String> {
        let argon2_password = match password_util::encode_argon2(password) {
            Ok(hashed_password) => hashed_password,
            Err(e) => return Err(e),
        };
        let mut user = user.into_active_model();
        user.password = Set(argon2_password);
        user.update(self.db).await.map_err(|e| e.to_string())
    }
}
