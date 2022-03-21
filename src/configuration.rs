#[derive(serde::Deserialize)]
pub struct Settings {
    pub application_port: u16,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub driver: String,
    pub username: String,
    pub password: String,
    pub port: String,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings,config::ConfigError> {
    let mut settings = config::Config::default();

    settings.merge(config::File::with_name("config"))?;

    return settings.try_into();
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn connection_string_without_database(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password,
            self.host,
            self.port
        )
    }
}
