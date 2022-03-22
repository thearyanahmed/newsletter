use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::get_configuration;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer,JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt,EnvFilter,Registry};
use tracing_log::LogTracer;
use tracing::Subscriber;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(),"info".into());

    init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}",config.application_port);

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("database connection failed.");

    let listener = TcpListener::bind(address.clone()).expect("failed to bind to port.");

    println!("running on {}",address);

    run(listener,connection_pool)?.await
}

pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // if RUST_LOG is absent, fallback to "info"
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(
        "newsletter_dev".into(),
        std::io::stdout // output the formatting span to stdout
    );

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to set logger");

    set_global_default(subscriber).expect("failed to set subscriber");
}