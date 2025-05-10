use std::env::{self, VarError};

use types::{
    ApplicationSettings, DatabaseSettings, Environment, RedisSettings, SecretSettings, Settings,
};

pub mod types;

pub fn get_settings(env_file_name: &str) -> Result<Settings, String> {
    dotenvy::from_filename(env_file_name)
        .map_err(|e| format!("Failed to fetch env file: {}", e.to_string()))?;

    match Environment::try_from(env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "production".into()))
    {
        Ok(env) => match env {
            Environment::Testing => get_development_settings(),
            Environment::Development => get_development_settings(),
            Environment::Production => get_production_settings(),
        },
        Err(e) => return Err(format!("Failed to parse APP_ENVIRONMENT: {}", e)),
    }
}

fn get_development_settings() -> Result<Settings, String> {
    let b = Settings::base_settings();
    merge_env(Settings {
        application: ApplicationSettings {
            host: "127.0.0.1".to_string(),
            protocol: "http".to_string(),
            ..b.application
        },
        debug: true,
        ..b
    })
    .map_err(|e| format!("Failed to get env variables: {}", e.to_string()))
}

fn get_production_settings() -> Result<Settings, String> {
    let b = Settings::base_settings();
    merge_env(Settings {
        application: ApplicationSettings {
            host: "0.0.0.0".to_string(),
            protocol: "https".to_string(),
            ..b.application
        },
        debug: false,
        ..b
    })
    .map_err(|e| format!("Failed to get env variables: {}", e.to_string()))
}

fn merge_env(s: Settings) -> Result<Settings, VarError> {
    Ok(Settings {
        database: DatabaseSettings {
            url: env::var("DATABASE_URL")?,
        },
        debug: match env::var("APP_DEBUG") {
            Ok(debug) => &debug == "true",
            Err(_) => s.debug,
        },
        redis: RedisSettings {
            url: env::var("REDIS_URL")?,
            ..s.redis
        },
        secret: SecretSettings {
            secret_key: env::var("APP_SECRET__SECRET_KEY")?,
            hmac_secret: env::var("APP_SECRET__HMAC_SECRET")?,
            ..s.secret
        },
        ..s
    })
}
