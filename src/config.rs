use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub ynab_client: YnabClientSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub webdriver: WebDriverSettings,
}

#[derive(Clone, Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Clone, Deserialize)]
pub struct YnabClientSettings {
    pub pat: Secret<String>,
    pub base_url: String,
}

impl YnabClientSettings {
    pub fn client(self) -> ynab::Client {
        ynab::Client::new(self.pat.expose_secret(), &self.base_url)
            .expect("Failed to build ynab client.")
    }
}

#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

#[derive(Clone, Deserialize)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}/", self.host, self.port)
    }
}

#[derive(Clone, Deserialize)]
pub struct WebDriverSettings {
    pub host: String,
    pub port: u16,
}

impl WebDriverSettings {
    pub fn connection_string(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            application: ApplicationSettings {
                port: 3000,
                host: String::from("localhost"),
                base_url: String::from(""),
            },
            ynab_client: YnabClientSettings {
                pat: String::from("").into(),
                base_url: String::from("https://api.youneedabudget.com/v1/"),
            },
            database: DatabaseSettings {
                username: String::from("postgres"),
                password: String::from("password").into(),
                port: 5432,
                host: String::from("127.0.0.1"),
                database_name: String::from("datamize"),
                require_ssl: false,
            },
            redis: RedisSettings {
                host: String::from("127.0.0.1"),
                port: 6379,
            },
            webdriver: WebDriverSettings {
                host: String::from("localhost"),
                port: 4444,
            },
        }
    }
}

impl Settings {
    pub fn build() -> Result<Self, config::ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_directory = base_path.join("configuration");

        // Detect the running environment.
        // Default to `local` if unspecified.
        let environment: Environment = std::env::var("DATAMIZE_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse DATAMIZE_ENVIRONMENT.");
        let environment_filename = format!("{}.toml", environment.as_str());

        let settings = config::Config::builder()
            .add_source(config::File::from(
                configuration_directory.join("base.toml"),
            ))
            .add_source(config::File::from(
                configuration_directory.join(environment_filename),
            ))
            // Add in settings from environment variables (with a prefix of DATAMIZE and '__' as separator)
            // E.g. `DATAMIZE_APPLICATION__PORT=5001 would set `Settings.application.port`
            .add_source(
                config::Environment::with_prefix("DATAMIZE")
                    .prefix_separator("_")
                    .separator("__"),
            )
            .build()?;

        settings.try_deserialize::<Self>()
    }
}

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Staging => "staging",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
