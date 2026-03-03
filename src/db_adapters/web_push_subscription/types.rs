#[derive(Debug, Clone)]
pub struct CreateWebPushSubscriptionParams {
    pub user_id: i64,
    pub device_name: String,
    pub endpoint: String,
    pub expiration_epoch_time: Option<i64>,
    pub p256dh_key: String,
    pub auth_key: String,
}
