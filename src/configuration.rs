use config::{Config, ConfigError, File};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connetion_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}/{}",
            self.username, self.password, self.host, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    // Initialize our configuration reader
    let settings = Config::builder()
        // Add configuration values from a file named `configuration`.
        // It will look for any top-level file with an extenstion
        // that `config` knows how to parse: yaml, json, etc.
        .add_source(File::with_name("configuration"))
        .build()
        .unwrap();

    // Try to convert the configuration values it read into
    // our Settings type
    Ok(settings.try_deserialize().unwrap())
}
