use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub ynab_client: YnabClientSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct YnabClientSettings {
    pat: String,
    base_url: String,
}

impl YnabClientSettings {
    pub fn client(self) -> ynab::Client {
        ynab::Client::new(&self.pat, &self.base_url).expect("Failed to build ynab client.")
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
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
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RedisSettings {
    pub host: String,
    pub port: u16,
}

impl RedisSettings {
    pub fn connection_string(&self) -> String {
        format!("redis://{}:{}/", self.host, self.port)
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
                pat: "".into(),
                base_url: String::from("https://api.youneedabudget.com/v1/"),
            },
            database: DatabaseSettings {
                username: String::from("postgres"),
                password: String::from("password"),
                port: 5432,
                host: String::from("127.0.0.1"),
                database_name: String::from("budget_data"),
                require_ssl: false,
            },
            redis: RedisSettings {
                host: String::from("127.0.0.1"),
                port: 6379,
            },
        }
    }
}

// TODO: Add ability to change config on local vs prod. Check zeroToProd project.
// TODO: Make use of Secrecy crate for sensible configs.
impl Settings {
    pub fn build() -> Self {
        Figment::from(Serialized::defaults(Settings::default()))
            .merge(Toml::file("budget-data-config.toml"))
            .merge(Env::prefixed("BUDGET_DATA_"))
            .extract()
            .expect("Failed to extract config files and environment variables into a rust struct.")
    }
}
