mod web_push_message_encryptor;
mod web_push_messenger;
mod web_push_vapid_signature_builder;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
pub use web_push_messenger::WebPushMessenger;

use common::settings::types::Settings;
use entities::{user_relations_userrelation::UserRelationId, web_push_subscription};

use crate::web_push::message_sender::web_push_messenger::WebPushMessengerError;

#[derive(Debug, Serialize, PartialEq, Deserialize, Clone)]
pub enum MessageType {
    MakeWish,
    WishReply,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Message {
    pub title: Option<String>,
    pub body: String,
    pub message_type: MessageType,
    pub user_relation_id: Option<UserRelationId>,
    pub wish_id: Option<Uuid>,
    pub wish_reply_id: Option<Uuid>,
}

#[derive(Debug, Error)]
pub enum SendWebPushError {
    #[error("")]
    NotSent,
    #[error("")]
    Invalid,
}

pub async fn send_web_push(
    message: Message,
    web_push_subscription: &web_push_subscription::Model,
    settings: &Settings,
) -> Result<(), SendWebPushError> {
    let messenger =
        WebPushMessenger::new(web_push_subscription, settings).map_err(|_| SendWebPushError::NotSent)?;

    messenger.send_message(message).await.map_err(|e| match e {
        WebPushMessengerError::InvalidSubscription() => SendWebPushError::Invalid,
        _ => SendWebPushError::NotSent,
    })
}
