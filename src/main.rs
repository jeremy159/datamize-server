use anyhow::Context;
use budget_data_server::{
    config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

// TODO: Rename to Dudget
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber(
        "budget_data_server".into(),
        "budget_data_server=debug,tower_http=debug".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = config::Settings::build()?;
    let application = Application::build(configuration.clone())
        .await
        .context("failed to build application")?;

    application
        .run()
        .await
        .context("failed to run application")?;

    Ok(())
}
