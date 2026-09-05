use std::future::Future;

use entities::{
    users_user::UserId,
    web_push_subscription::{Entity, Model},
};

use crate::web_push_subscription::{
    WebPushSubscriptionService,
    WebPushSubscriptionServiceError::{self},
};

pub trait WebPushSubscriptionServiceQuery {
    fn get_opt_by_user_id(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Model>, WebPushSubscriptionServiceError>>;
}

impl WebPushSubscriptionServiceQuery for WebPushSubscriptionService<'_> {
    async fn get_opt_by_user_id(&self, user_id: UserId) -> Result<Option<Model>, WebPushSubscriptionServiceError> {
        Entity::find_by_user_id(user_id)
            .one(self.db)
            .await
            .map_err(|e| e.into())
    }
}
