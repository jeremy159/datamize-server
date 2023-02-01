use anyhow::Context;
use datamize::{
    config,
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
    let application = Application::build(configuration.clone())
        .await
        .context("failed to build application")?;

    application
        .run()
        .await
        .context("failed to run application")?;

    Ok(())
}
