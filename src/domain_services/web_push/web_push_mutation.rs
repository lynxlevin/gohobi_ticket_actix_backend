use std::future::Future;

use entities::web_push_subscription::Model;
use sea_orm::ModelTrait;

use crate::web_push::{
    WebPushService,
    WebPushServiceError::{self},
};

pub trait WebPushServiceMutation {
    fn delete_subscription(
        &self,
        web_push_subscription: Model,
    ) -> impl Future<Output = Result<(), WebPushServiceError>>;
}

impl WebPushServiceMutation for WebPushService<'_> {
    async fn delete_subscription(&self, web_push_subscription: Model) -> Result<(), WebPushServiceError> {
        web_push_subscription
            .delete(self.db)
            .await
            .map(|_| ())
            .map_err(|e| e.into())
    }
}
