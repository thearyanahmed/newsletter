use newsletter::startup::{build};
use newsletter::configuration::get_configuration;
use newsletter::telemetry;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("newsletter_prod".into(),"info".into(),std::io::stdout);

    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration.");

    let server = build(config).await?;

    server.await?;

    Ok(())
}
