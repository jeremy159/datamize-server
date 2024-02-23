use datamize_domain::secrecy::{ExposeSecret, Secret};
use db_postgres::{PgConnectOptions, PgSslMode};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use serde::Deserialize;
use sqlx::ConnectOptions;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub ynab_client: YnabClientSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub webdriver: WebDriverSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YnabClientSettings {
    pub pat: Secret<String>,
    pub pat_file: String,
    pub base_url: String,
    pub client_id: String,
    pub client_secret: Secret<String>,
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
}

impl YnabClientSettings {
    pub fn client(mut self) -> ynab::Client {
        self.pat = match self.pat_file {
            file_path if !file_path.is_empty() => {
                self.pat = Secret::new(std::fs::read_to_string(file_path).unwrap());
                self.pat
            }
            _ => self.pat,
        };

        ynab::Client::new(self.pat.expose_secret(), Some(&self.base_url))
            .expect("Failed to build ynab client.")
    }

    pub fn oauth_client(self) -> Result<BasicClient, oauth2::url::ParseError> {
        Ok(BasicClient::new(
            ClientId::new(self.client_id),
            Some(ClientSecret::new(
                self.client_secret.expose_secret().to_string(),
            )),
            AuthUrl::new(self.auth_url)?,
            Some(TokenUrl::new(self.token_url)?),
        )
        .set_redirect_uri(RedirectUrl::new(self.redirect_url)?))
    }
}

#[derive(Debug, Clone, Deserialize)]
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
        self.without_db()
            .database(&self.database_name)
            .log_statements(tracing::log::LevelFilter::Trace)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}/", self.host, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebDriverSettings {
    pub host: String,
    pub port: u16,
}

impl WebDriverSettings {
    pub fn connection_string(&self) -> String {
        format!("http://{}:{}/", self.host, self.port)
    }
}

impl Settings {
    pub fn build() -> Result<Self, config::ConfigError> {
        let base_path = {
            let p = std::env::current_dir().expect("Failed to determine the current directory");
            if !p.ends_with("crates/datamize-server") {
                p.join("crates/datamize-server")
            } else {
                p
            }
        };
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
    Test,
    Staging,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Test => "test",
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
            "test" => Ok(Self::Test),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
