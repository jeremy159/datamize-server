use anyhow::Context;
use datamize::{
    config::{self, DatabaseSettings},
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "datamize".into(),
        "datamize=debug,tower_http=debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = config::Settings::build()?;
    let db_conn_pool = get_connection_pool(&configuration.database);
    let application = Application::build(configuration.clone(), db_conn_pool)
        .await
        .context("failed to build application")?;

    application
        .run()
        .await
        .context("failed to run application")?;

    Ok(())
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
