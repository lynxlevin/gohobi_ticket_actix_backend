use std::future::Future;

use entities::{
    users_user::UserId,
    web_push_subscription::{Entity, Model},
};

use crate::web_push::{
    WebPushService,
    WebPushServiceError::{self},
};

pub trait WebPushServiceQuery {
    fn get_subscription(
        &self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Model>, WebPushServiceError>>;
}

impl WebPushServiceQuery for WebPushService<'_> {
    async fn get_subscription(&self, user_id: UserId) -> Result<Option<Model>, WebPushServiceError> {
        Entity::find_by_user_id(user_id)
            .one(self.db)
            .await
            .map_err(|e| e.into())
    }
}
