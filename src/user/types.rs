use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct UserVisible {
    pub id: i64,
    pub username: String,
    pub email: String,
}
