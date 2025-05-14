use serde::Deserialize;

#[derive(Deserialize, Clone, Default, Debug)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub debug: bool,
    pub redis: RedisSettings,
    pub secret: SecretSettings,
}

impl Settings {
    pub fn base_settings() -> Self {
        Self {
            application: ApplicationSettings {
                port: 5000,
                max_login_attempts: 5,
                ..Default::default()
            },
            redis: RedisSettings {
                pool_max_open: 16,
                pool_max_idle: 8,
                pool_timeout_seconds: 1,
                pool_expire_seconds: 60,
                ..Default::default()
            },
            secret: SecretSettings {
                token_expiration: 30,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct ApplicationSettings {
    pub port: u32,
    pub host: String,
    pub protocol: String,
    pub max_login_attempts: u32,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct DatabaseSettings {
    pub url: String,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct RedisSettings {
    pub url: String,
    pub pool_max_open: u32,
    pub pool_max_idle: u32,
    pub pool_timeout_seconds: u32,
    pub pool_expire_seconds: u32,
}

#[derive(Deserialize, Clone, Default, Debug)]
pub struct SecretSettings {
    pub secret_key: String,
    pub token_expiration: u32,
    pub hmac_secret: String,
}

pub enum Environment {
    Testing,
    Development,
    Production,
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
