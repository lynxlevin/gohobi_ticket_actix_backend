use common::{errors::use_case_errors::UseCaseError, settings::types::Settings};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Pool,
};

#[derive(Clone)]
pub struct UserRedis<'a> {
    pub pool: &'a Pool,
    pub settings: &'a Settings,
}

impl UserRedis<'_> {
    pub async fn validate_request_count(
        self,
        login_attempts_count_key: &str,
    ) -> Result<u64, UseCaseError> {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(_) => return Err(UseCaseError::InternalServerError),
        };
        let login_attempts_count = conn.get(login_attempts_count_key).await.unwrap_or(0);
        match login_attempts_count >= self.settings.application.max_login_attempts {
            true => Err(UseCaseError::Unauthorized),
            false => Ok(login_attempts_count),
        }
    }

    pub async fn increment_login_attempts_count(
        self,
        login_attempts_count_key: &str,
        login_attempts_count: u64,
    ) -> () {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(_) => return,
        };
        match conn
            .set_options::<String, u64, String>(
                login_attempts_count_key.to_string(),
                login_attempts_count + 1,
                SetOptions::default().with_expiration(SetExpiry::EX(
                    self.settings.application.login_attempts_cool_time_seconds,
                )),
            )
            .await
        {
            Ok(_) => {}
            Err(_) => {}
        };
    }
}
