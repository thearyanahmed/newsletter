use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::get_configuration;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer,JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt,EnvFilter,Registry};
use tracing_log::LogTracer;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    setup_logger();

    let config = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}",config.application_port);

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("database connection failed.");

    let listener = TcpListener::bind(address.clone()).expect("failed to bind to port.");

    println!("running on {}",address);

    run(listener,connection_pool)?.await
}

fn setup_logger() {
    LogTracer::init().expect("failed to set logger");

    // if RUST_LOG is absent, fallback to "info"
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new(
        "newsletter_dev".into(),
        std::io::stdout // output the formatting span to stdout
    );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("failed to set subscriber");
}