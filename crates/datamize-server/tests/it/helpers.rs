use datamize_server::{
    config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use db_redis::{get_connection_pool, RedisPool};
use once_cell::sync::Lazy;
use sqlx::PgPool;
use wiremock::MockServer;

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "warn".to_string();
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
    pub redis_pool: RedisPool,
    pub api_client: reqwest::Client,
    pub ynab_server: MockServer,
    pub ynab_client: ynab::Client,
}

pub async fn spawn_app(db_pool: PgPool) -> TestApp {
    Lazy::force(&TRACING);

    // Launch a mock server to stand in for YNAB's API
    let ynab_server = MockServer::start().await;

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = config::Settings::build().expect("Failed to read configuration.");
        // Use a random OS port
        c.application.port = 0;
        // Use the mock server as ynab API
        c.ynab_client.base_url = ynab_server.uri();
        c
    };

    // Launch the application as a background task
    let application = Application::build(configuration.clone(), db_pool.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    tokio::spawn(application.run());

    // Give time for the app to start before sending in requests. Not ideal at all...
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address: format!("http://localhost:{}", application_port),
        port: application_port,
        db_pool,
        redis_pool: get_connection_pool(&configuration.redis.connection_string())
            .await
            .expect("Failed to start connection to redis."),
        api_client: client,
        ynab_server,
        ynab_client: configuration.ynab_client.client(),
    }
}
