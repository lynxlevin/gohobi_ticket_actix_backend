pub mod types;

pub fn get_settings(env_name: &str) -> Result<types::Settings, String> {
    dotenvy::from_filename(env_name).ok();
    let base_path = match std::env::current_dir() {
        Ok(path) => path,
        Err(e) => {
            return Err(format!(
                "Failed to determine the current directory: {}",
                e.to_string()
            ))
        }
    };
    let settings_directory = base_path.join("src/general/settings/yaml");

    let environment: types::Environment = match std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "development".into())
        .try_into()
    {
        Ok(env) => env,
        Err(e) => return Err(format!("Failed to parse APP_ENVIRONMENT: {}", e)),
    };
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = match config::Config::builder()
        .add_source(config::File::from(settings_directory.join("base.yaml")))
        .add_source(config::File::from(
            settings_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. 'APP_APPLICATION__PORT=5001' would set 'Settings.application.port'
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
    {
        Ok(settings) => settings,
        Err(e) => return Err(format!("Failed to build config: {}", e.to_string())),
    };

    settings
        .try_deserialize::<types::Settings>()
        .map_err(|e| format!("Failed to read settings: {}", e.to_string()))
}
