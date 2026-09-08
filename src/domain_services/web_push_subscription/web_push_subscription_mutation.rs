use std::future::Future;

use entities::web_push_subscription::Model;
use sea_orm::ModelTrait;

use crate::web_push_subscription::{
    WebPushSubscriptionService,
    WebPushSubscriptionServiceError::{self},
};

pub trait WebPushSubscriptionServiceMutation {
    fn delete(
        &self,
        web_push_subscription: Model,
    ) -> impl Future<Output = Result<(), WebPushSubscriptionServiceError>>;
}

impl WebPushSubscriptionServiceMutation for WebPushSubscriptionService<'_> {
    async fn delete(&self, web_push_subscription: Model) -> Result<(), WebPushSubscriptionServiceError> {
        web_push_subscription
            .delete(self.db)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
