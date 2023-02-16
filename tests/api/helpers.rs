use datamize::{
    config::{self, DatabaseSettings},
    domain::NetTotalType,
    startup::{get_connection_pool, get_redis_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
    pub redis_pool: r2d2::Pool<redis::Client>,
    pub api_client: reqwest::Client,
    pub ynab_server: MockServer,
    pub ynab_client: ynab::Client,
}

impl TestApp {
    pub async fn get_years_summary(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/api/balance_sheet/years", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn create_year<B>(&self, body: &B) -> reqwest::Response
    where
        B: Serialize,
    {
        self.api_client
            .post(&format!("{}/api/balance_sheet/years", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn get_year(&self, year: i32) -> reqwest::Response {
        self.api_client
            .get(&format!(
                "{}/api/balance_sheet/years/{}",
                &self.address, year
            ))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn insert_year(&self, year: i32) -> Uuid {
        let year_id = uuid::Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_years (id, year)
            VALUES ($1, $2);
            "#,
            year_id,
            year,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert a year.");

        year_id
    }

    pub async fn insert_net_total(
        &self,
        year_id: Uuid,
        net_type: NetTotalType,
        total: i64,
        percent_var: f32,
        balance_var: i64,
    ) -> Uuid {
        let net_total_id = uuid::Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO balance_sheet_net_totals_years (id, type, total, percent_var, balance_var, year_id)
            VALUES ($1, $2, $3, $4, $5, $6);
            "#,
            net_total_id,
            net_type.to_string(),
            total,
            percent_var,
            balance_var,
            year_id,
        )
        .execute(&self.db_pool)
        .await
        .expect("Failed to insert net totals of a year.");

        net_total_id
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for YNAB's API
    let ynab_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = config::Settings::build().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as ynab API
        c.ynab_client.base_url = ynab_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Launch the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let _ = tokio::spawn(application.run());

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_app = TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        redis_pool: get_redis_connection_pool(&configuration.redis)
            .expect("Failed to start connection to redis."),
        api_client: client,
        ynab_server,
        ynab_client: configuration.ynab_client.client(),
    };

    test_app
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(&*format!(r#"CREATE DATABASE "{}";"#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
