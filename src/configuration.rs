use std::{convert::TryInto, time::Duration};

#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub name: String,
    pub connection_timeout: Option<Duration>,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "mongodb://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

enum Envirorment {
    Local,
    Production,
}

impl Envirorment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Envirorment::Local => "local",
            Envirorment::Production => "production",
        }
    }
}

impl std::convert::TryFrom<String> for Envirorment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Envirorment::Local),
            "production" => Ok(Envirorment::Production),
            other => Err(format!(
                "'{}' Is not a supported envirorment. Use 'local' or 'production'",
                other
            )),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();
    let base_path = std::env::current_dir().expect("Cannot determine current directory");
    let config_directory = base_path.join("configuration");

    settings.merge(config::File::from(config_directory.join("base")).required(true))?;

    let envirorment: Envirorment = std::env::var("APP_ENVIRORMENT")
        .unwrap_or_else(|_| "local".to_owned())
        .try_into()
        .expect("Cannot parse APP_ENVIRORMENT");
    settings
        .merge(config::File::from(config_directory.join(envirorment.as_str())).required(true))?;

    settings.merge(config::Environment::with_prefix("app").separator("__"))?;

    settings.try_into()
}
