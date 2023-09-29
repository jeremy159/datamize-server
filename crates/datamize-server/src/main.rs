use anyhow::Context;
use datamize_server::{
    config, get_connection_pool,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

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
    let application = Application::build(configuration, db_conn_pool)
        .await
        .context("failed to build application")?;

    application
        .run()
        .await
        .context("failed to run application")?;

    Ok(())
}
