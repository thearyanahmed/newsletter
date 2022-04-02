use newsletter::startup::{Application};
use newsletter::configuration::get_configuration;
use newsletter::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("newsletter_prod".into(),"info".into(),std::io::stdout);

    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration.");

    let app = Application::build(&config).await?;

    app.run_until_stopped().await?;

    Ok(())
}
