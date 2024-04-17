use anyhow::Context;
use datamize_server::{
    config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};
use db_postgres::get_connection_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "datamize".into(),
        "datamize=debug,tower_http=debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = config::Settings::build()?;
    let db_conn_pool = get_connection_pool(configuration.database.with_db());
    // TODO: Check https://willcrichton.net/rust-api-type-patterns/registries.html for a different way of build a server with different dependencies
    let application = Application::build(configuration, db_conn_pool)
        .await
        .context("failed to build application")?;

    application
        .run()
        .await
        .context("failed to run application")?;

    Ok(())
}
