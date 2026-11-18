use common::{
    db::Db,
    factory::{self, WebPushSubscriptionFactory},
    settings::types::Settings,
};
use ece::EcKeyComponents;
use entities::users_user::UserId;
use sea_orm::{ActiveModelTrait, DbErr};

/// returns (auth_key, private_key)
pub async fn register_web_push_subscription(
    endpoint: &str,
    settings: &Settings,
    db: &Db,
    user_id: UserId,
) -> Result<([u8; 16], EcKeyComponents), DbErr> {
    let (key_pair, auth_key) = ece::generate_keypair_and_auth_secret().unwrap();
    let private_key = key_pair.raw_components().unwrap();
    let p256dh_key = key_pair.pub_as_raw().unwrap();
    let _web_push_sub = factory::web_push_subscription(user_id)
        .set_raw_endpoint(endpoint)
        .set_raw_p256dh_key(p256dh_key.clone())
        .set_raw_auth_key(auth_key)
        .encrypt_and_encode_sensitive_fields(settings)
        .insert(&db.db)
        .await?;
    Ok((auth_key, private_key))
}
