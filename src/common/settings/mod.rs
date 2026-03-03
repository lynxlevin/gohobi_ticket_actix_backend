use std::env;

use types::{
    ApplicationSettings, DatabaseSettings, Environment, RedisSettings, SecretSettings, Settings,
};

pub mod types;

pub fn get_test_settings() -> Settings {
    get_settings(".env.testing").expect("Error on getting settings.")
}

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
            slack_host: "https://hooks.slack.com".to_string(),
            ..b.application
        },
        debug: true,
        ..b
    })
}

fn get_production_settings() -> Result<Settings, String> {
    let b = Settings::base_settings();
    merge_env(Settings {
        application: ApplicationSettings {
            host: "0.0.0.0".to_string(),
            slack_host: "https://hooks.slack.com".to_string(),
            ..b.application
        },
        debug: false,
        ..b
    })
}

fn merge_env(s: Settings) -> Result<Settings, String> {
    Ok(Settings {
        application: ApplicationSettings {
            max_login_attempts: get_env_var("MAX_LOGIN_ATTEMPTS")?
                .parse::<u64>()
                .map_err(|e| e.to_string())?,
            login_attempts_cool_time_seconds: get_env_var("LOGIN_ATTEMPTS_COOL_TIME_SECONDS")?
                .parse::<u64>()
                .map_err(|e| e.to_string())?,
            slack_incoming_webhook_path: get_env_var("SLACK_INCOMING_WEBHOOK_PATH")?,
            ..s.application
        },
        database: DatabaseSettings {
            url: get_env_var("DATABASE_URL")?,
            encryption_key: get_env_var("DATABASE_ENCRYPTION_KEY")?,
            encryption_nonce: get_env_var("DATABASE_ENCRYPTION_NONCE")?,
        },
        debug: match env::var("APP_DEBUG") {
            Ok(debug) => &debug == "true",
            Err(_) => s.debug,
        },
        redis: RedisSettings {
            url: get_env_var("REDIS_URL")?,
            ..s.redis
        },
        secret: SecretSettings {
            hmac_secret: get_env_var("APP_SECRET__HMAC_SECRET")?,
            ..s.secret
        },
        ..s
    })
}

fn get_env_var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| e.to_string())
}
