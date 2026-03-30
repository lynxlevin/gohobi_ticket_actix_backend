use common::errors::use_case_errors::UseCaseError;
use db_adapters::web_push_subscription::WebPushSubscriptionQuery;
use entities::users_user;

use crate::types::WebPushSubscriptionVisible;

pub async fn list_web_push_subscription<'a>(
    user: users_user::Model,
    web_push_subscription_adapter: WebPushSubscriptionQuery<'a>,
) -> Result<Option<WebPushSubscriptionVisible>, UseCaseError> {
    web_push_subscription_adapter
        .get_by_user(&user)
        .await
        .map(|subscription| subscription.map(|sub| WebPushSubscriptionVisible::from(sub)))
        .map_err(|_| UseCaseError::InternalServerError)
}
