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

#[cfg(test)]
mod tests {
    use general::{
        db::init_db,
        factory::{self, *},
        settings::get_test_settings,
    };
    use sea_orm::{DbErr, EntityTrait};

    use crate::constants::ARGON2_START_WORD;

    use super::*;

    #[actix_web::test]
    async fn test_convert_to_argon2_password() -> Result<(), DbErr> {
        let settings = get_test_settings();
        let db = init_db(&settings).await?;
        let password = "password";
        let django_password =
            "pbkdf2_sha256$260000$N4b3mSYc5bXPsCkD7G3eKt$4nfua4vv7GLRqeRHxCcDmjtMxB6LYZNhMf6Lqh48RDE=";
        let user = factory::user()
            .password(django_password)
            .insert(&db)
            .await?;

        let user_mutation = UserMutation { db: &db };
        let res_user = user_mutation
            .convert_to_argon2_password(user, password)
            .await
            .unwrap();

        assert!(res_user.password.starts_with(ARGON2_START_WORD));

        let user_in_db = users_user::Entity::find_by_id(res_user.id)
            .one(&db)
            .await?
            .unwrap();
        assert!(user_in_db.password.starts_with(ARGON2_START_WORD));

        Ok(())
    }
}
