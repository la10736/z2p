use serde_with::{serde_as, DisplayFromStr, DurationSecondsWithFrac};
use std::{convert::TryInto, time::Duration};

#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[serde_as]
#[derive(serde::Deserialize, Default, Clone, Debug, PartialEq)]
pub struct ApplicationSettings {
    pub host: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
}

#[serde_as]
#[derive(serde::Deserialize, Default, Clone, Debug, PartialEq)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
    pub host: String,
    pub name: String,
    #[serde_as(as = "Option<DurationSecondsWithFrac<String>>")]
    #[serde(default)]
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

#[cfg(test)]
mod test {
    use rstest::rstest;
    use unindent::Unindent;

    use super::*;

    #[rstest(yaml, expected,
        case::port_as_string(r#"
        ---
          host: 0.0.0.0
          port: "1234"
        "#.unindent(),
        ApplicationSettings {
            host: "0.0.0.0".to_owned(),
            port: 1234
        }
        ),
        case::port_as_number(r#"
        ---
          host: 0.0.0.0
          port: 1234
        "#.unindent(),
        ApplicationSettings {
            host: "0.0.0.0".to_owned(),
            port: 1234
        }
        ),
    )]
    fn deserialize_applications_settings(yaml: String, expected: ApplicationSettings) {
        let app = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(expected, app)
    }

    #[rstest(yaml, expected,
        case::happy(r#"
        ---
          username: user
          password: pwd
          port: 1234
          host: 127.0.0.1
          name: name
          connection_timeout: 0.345
        "#.unindent(),
        DatabaseSettings {
            username: "user".to_owned(),
            password: "pwd".to_owned(),
            port: 1234,
            host: "127.0.0.1".to_owned(),
            name: "name".to_owned(),
            connection_timeout: Some(Duration::from_millis(345)),
        }
        ),
        case::no_connection_timeout(r#"
        ---
          username: user
          password: pwd
          port: 1234
          host: 127.0.0.1
          name: name
        "#.unindent(),
        DatabaseSettings {
            username: "user".to_owned(),
            password: "pwd".to_owned(),
            port: 1234,
            host: "127.0.0.1".to_owned(),
            name: "name".to_owned(),
            connection_timeout: None,
        }
        ),
        case::port_as_string(r#"
        ---
          username: user
          password: pwd
          port: "1234"
          host: 127.0.0.1
          name: name
          connection_timeout: 3
        "#.unindent(),
        DatabaseSettings {
            username: "user".to_owned(),
            password: "pwd".to_owned(),
            port: 1234,
            host: "127.0.0.1".to_owned(),
            name: "name".to_owned(),
            connection_timeout: Some(Duration::from_secs(3)),
        }
        ),
    )]
    fn deserialize_db_settings(yaml: String, expected: DatabaseSettings) {
        let app = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(expected, app)
    }
}
