use common::errors::use_case_errors::UseCaseError;
use db_adapters::web_push_subscription::{WebPushSubscriptionMutation, WebPushSubscriptionQuery};
use entities::users_user;

pub async fn delete_web_push_subscription<'a>(
    user: users_user::Model,
    web_push_subscription_query: WebPushSubscriptionQuery<'a>,
    web_push_subscription_mutation: WebPushSubscriptionMutation<'a>,
) -> Result<(), UseCaseError> {
    let subscription = web_push_subscription_query
        .get_by_user(&user)
        .await
        .map_err(|_| UseCaseError::InternalServerError)?;

    if let Some(subscription) = subscription {
        web_push_subscription_mutation
            .delete(subscription)
            .await
            .map_err(|_| UseCaseError::InternalServerError)?;
    }
    Ok(())
}
