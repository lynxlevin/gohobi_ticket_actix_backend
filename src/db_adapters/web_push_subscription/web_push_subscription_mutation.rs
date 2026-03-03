use entities::web_push_subscription::{ActiveModel, Column, Entity, Model};
use sea_orm::{sea_query::OnConflict, DbConn, DbErr, EntityTrait, ModelTrait, Set};
use uuid::Uuid;

use super::types::CreateWebPushSubscriptionParams;

pub struct WebPushSubscriptionMutation<'a> {
    pub db: &'a DbConn,
}

impl<'a> WebPushSubscriptionMutation<'a> {
    pub async fn upsert(self, params: CreateWebPushSubscriptionParams) -> Result<Model, DbErr> {
        let subscription = ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            device_name: Set(params.device_name.clone()),
            endpoint: Set(params.endpoint.clone()),
            expiration_epoch_time: Set(params.expiration_epoch_time),
            p256dh_key: Set(params.p256dh_key.clone()),
            auth_key: Set(params.auth_key.clone()),
        };
        Entity::insert(subscription)
            .on_conflict(
                OnConflict::column(Column::UserId)
                    .update_columns([
                        Column::DeviceName,
                        Column::Endpoint,
                        Column::ExpirationEpochTime,
                        Column::P256dhKey,
                        Column::AuthKey,
                    ])
                    .to_owned(),
            )
            .exec(self.db)
            .await
            .map(|res| Model {
                id: res.last_insert_id,
                user_id: params.user_id,
                device_name: params.device_name,
                endpoint: params.endpoint,
                expiration_epoch_time: params.expiration_epoch_time,
                p256dh_key: params.p256dh_key,
                auth_key: params.auth_key,
            })
    }

    pub async fn delete(self, web_push_subscription: Model) -> Result<(), DbErr> {
        web_push_subscription.delete(self.db).await.map(|_| ())
    }
}
