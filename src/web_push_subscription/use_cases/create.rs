use common::{
    db::encrypt_and_encode, errors::use_case_errors::UseCaseError, settings::types::Settings,
};
use db_adapters::web_push_subscription::{
    types::CreateWebPushSubscriptionParams, WebPushSubscriptionMutation,
};
use entities::users_user;

use crate::types::{WebPushSubscriptionCreateRequest, WebPushSubscriptionVisible};

pub async fn create_web_push_subscription<'a>(
    user: users_user::Model,
    settings: &Settings,
    params: WebPushSubscriptionCreateRequest,
    web_push_subscription_adapter: WebPushSubscriptionMutation<'a>,
) -> Result<WebPushSubscriptionVisible, UseCaseError> {
    let encrypted_endpoint = encrypt_and_encode(params.endpoint.clone(), settings)
        .map_err(|_| UseCaseError::InternalServerError)?;
    let encrypted_p256dh_key = encrypt_and_encode(params.p256dh_key.clone(), settings)
        .map_err(|_| UseCaseError::InternalServerError)?;
    let encrypted_auth_key = encrypt_and_encode(params.auth_key.clone(), settings)
        .map_err(|_| UseCaseError::InternalServerError)?;
    web_push_subscription_adapter
        .upsert(CreateWebPushSubscriptionParams {
            user_id: user.id,
            device_name: params.device_name.clone(),
            endpoint: encrypted_endpoint,
            expiration_epoch_time: params.expiration_epoch_time,
            p256dh_key: encrypted_p256dh_key,
            auth_key: encrypted_auth_key,
        })
        .await
        .map(|subscription| WebPushSubscriptionVisible::from(subscription))
        .map_err(|_| UseCaseError::InternalServerError)
}
