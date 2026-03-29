mod web_push_message_encryptor;
mod web_push_messenger;
mod web_push_vapid_signature_builder;
pub use web_push_messenger::{Message, MessageType, WebPushMessenger, WebPushMessengerResult};

use crate::settings::types::Settings;
use entities::web_push_subscription;

// FIXME: Find a better way of handling these functions.
#[derive(PartialEq)]
pub enum SendWebPushResult {
    Sent,
    NotSent,
    Invalid,
}

pub async fn send_web_push(
    message: Message,
    web_push_subscription: &web_push_subscription::Model,
    settings: &Settings,
) -> SendWebPushResult {
    let messenger = match WebPushMessenger::new(web_push_subscription, settings) {
        Ok(messenger) => messenger,
        Err(_) => return SendWebPushResult::NotSent,
    };

    match messenger.send_message(message).await {
        Ok(result) => match result {
            WebPushMessengerResult::OK => SendWebPushResult::Sent,
            WebPushMessengerResult::InvalidSubscription => SendWebPushResult::Invalid,
        },
        Err(_) => SendWebPushResult::NotSent,
    }
}
