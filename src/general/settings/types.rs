use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub debug: bool,
    pub redis: RedisSettings,
    pub secret: Secret,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub protocol: String,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RedisSettings {
    pub url: String,
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout_seconds: u64,
    pub pool_expire_seconds: u64,
}

#[derive(Deserialize, Clone)]
pub struct Secret {
    pub secret_key: String,
    pub token_expiration: i64,
    pub hmac_secret: String,
}

pub enum Environment {
    Testing,
    Development,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Testing => "testing",
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "testing" => Ok(Self::Testing),
            "development" => Ok(Self::Development),
            "production" => Ok(Self::Production),
            other => Err(format!("{} is not a supported environment.", other)),
        }
    }
}
