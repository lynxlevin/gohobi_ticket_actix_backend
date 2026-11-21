mod message_sender;
mod web_push_mutation;
mod web_push_query;

use common::{db::Db, settings::types::Settings};
use entities::users_user::UserId;
use sea_orm::{DbConn, TransactionError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use web_push_mutation::*;
pub use web_push_query::*;

use crate::web_push::message_sender::{send_web_push, SendWebPushError};
pub use crate::web_push::message_sender::{Message, MessageType};

#[derive(Deserialize, Debug, Serialize, PartialEq)]
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

        let web_push_result = match send_web_push(message, &web_push_subscription, settings).await {
            Ok(_) => WebPushResult::Sent,
            Err(e) => match e {
                SendWebPushError::NotSent => WebPushResult::NotSent,
                SendWebPushError::Invalid => {
                    let _ = self.delete_subscription(web_push_subscription).await;
                    WebPushResult::NotSent
                }
            },
        };

        Ok(web_push_result)
    }
}
