use common::{
    db::Db,
    settings::types::Settings,
    web_push::{send_web_push, Message, SendWebPushResult},
};
use entities::users_user::UserId;
use sea_orm::{DbConn, TransactionError};
use thiserror::Error;

mod web_push_mutation;
mod web_push_query;

pub use web_push_mutation::*;
pub use web_push_query::*;

pub enum WebPushResult {
    Sent,
    NotSent,
}

#[derive(Debug, Error)]
pub enum WebPushServiceError {
    #[error(transparent)]
    DbErr(#[from] sea_orm::DbErr),
    #[error("")]
    NotFound(),
}
impl From<TransactionError<WebPushServiceError>> for WebPushServiceError {
    fn from(value: TransactionError<WebPushServiceError>) -> Self {
        match value {
            TransactionError::Connection(e) => e.into(),
            TransactionError::Transaction(e) => e,
        }
    }
}

pub struct WebPushService<'a> {
    pub db: &'a DbConn,
}

impl<'a> WebPushService<'a> {
    pub fn init(db: &'a Db) -> Self {
        Self { db: &db.db }
    }

    pub async fn send_message(
        &self,
        message: Message,
        send_to: UserId,
        settings: &Settings,
    ) -> Result<WebPushResult, WebPushServiceError> {
        let web_push_subscription = match self.get_subscription(send_to).await? {
            Some(sub) => sub,
            None => return Err(WebPushServiceError::NotFound()),
        };
        let result = send_web_push(message, &web_push_subscription, settings).await;
        let web_push_result = match result {
            SendWebPushResult::Sent => WebPushResult::Sent,
            SendWebPushResult::NotSent => WebPushResult::NotSent,
            SendWebPushResult::Invalid => {
                let _ = self.delete_subscription(web_push_subscription).await;
                WebPushResult::NotSent
            }
        };

        Ok(web_push_result)
    }
}
