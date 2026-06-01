use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use entities::web_push_subscription;
use sea_orm::Set;
use uuid::Uuid;

use crate::{db::encrypt_and_encode, settings::types::Settings};

pub fn web_push_subscription(user_id: i64) -> web_push_subscription::ActiveModel {
    let raw_endpoint = "endpoint".to_string();
    let raw_p256dh_key = "p256dh_key".to_string();
    let raw_auth_key = "auth_key".to_string();
    web_push_subscription::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        device_name: Set("My device".to_string()),
        endpoint: Set(raw_endpoint),
        expiration_epoch_time: Set(None),
        p256dh_key: Set(raw_p256dh_key),
        auth_key: Set(raw_auth_key),
    }
}

pub trait WebPushSubscriptionFactory {
    fn set_raw_endpoint(self, raw_endpoint: &str) -> web_push_subscription::ActiveModel;
    fn set_raw_p256dh_key(self, raw_p256dh_key: Vec<u8>) -> web_push_subscription::ActiveModel;
    fn set_raw_auth_key(self, raw_auth_key: [u8; 16]) -> web_push_subscription::ActiveModel;
    fn encrypt_and_encode_sensitive_fields(self, settings: &Settings) -> web_push_subscription::ActiveModel;
    fn get_model(self) -> web_push_subscription::Model;
}

impl WebPushSubscriptionFactory for web_push_subscription::ActiveModel {
    fn set_raw_endpoint(mut self, raw_endpoint: &str) -> web_push_subscription::ActiveModel {
        self.endpoint = Set(raw_endpoint.to_string());
        self
    }

    /// Generate keys like this.
    /// ```
    /// let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
    /// let p256dh_key = key_pair.pub_as_raw().unwrap();
    /// ```
    fn set_raw_p256dh_key(mut self, raw_p256dh_key: Vec<u8>) -> web_push_subscription::ActiveModel {
        self.p256dh_key = Set(BASE64_URL_SAFE_NO_PAD.encode(raw_p256dh_key));
        self
    }
    /// Generate keys like this.
    /// ```
    /// let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
    /// let p256dh_key = key_pair.pub_as_raw().unwrap();
    /// ```
    fn set_raw_auth_key(mut self, raw_auth_key: [u8; 16]) -> web_push_subscription::ActiveModel {
        self.auth_key = Set(BASE64_URL_SAFE_NO_PAD.encode(raw_auth_key));
        self
    }
    fn encrypt_and_encode_sensitive_fields(mut self, settings: &Settings) -> web_push_subscription::ActiveModel {
        self.endpoint = Set(encrypt_and_encode(self.endpoint.unwrap(), settings).unwrap());
        self.p256dh_key = Set(encrypt_and_encode(self.p256dh_key.unwrap(), settings).unwrap());
        self.auth_key = Set(encrypt_and_encode(self.auth_key.unwrap(), settings).unwrap());
        self
    }
    fn get_model(self) -> web_push_subscription::Model {
        web_push_subscription::Model {
            id: self.id.unwrap(),
            user_id: self.user_id.unwrap(),
            device_name: self.device_name.unwrap(),
            endpoint: self.endpoint.unwrap(),
            expiration_epoch_time: self.expiration_epoch_time.unwrap(),
            p256dh_key: self.p256dh_key.unwrap(),
            auth_key: self.auth_key.unwrap(),
        }
    }
}
