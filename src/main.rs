use budget_data_server::{config, startup::Application};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration = config::Settings::build();
    let application = Application::build(configuration.clone()).await?;
    application.run().await?;

    Ok(())
}
