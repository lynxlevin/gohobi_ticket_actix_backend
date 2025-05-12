use entities::users_user;
use sea_orm::{ActiveModelTrait, DbConn, IntoActiveModel, Set};

use crate::password_util;

pub struct UserMutation;

impl UserMutation {
    pub async fn convert_to_argon2_password(
        db: &DbConn,
        user: users_user::Model,
        password: &str,
    ) -> Result<users_user::Model, String> {
        let argon2_password = match password_util::encode_argon2(password) {
            Ok(hashed_password) => hashed_password,
            Err(e) => return Err(e),
        };
        let mut user = user.into_active_model();
        user.password = Set(argon2_password);
        user.update(db).await.map_err(|e| e.to_string())
    }
}
