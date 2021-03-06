use tracing::subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer,JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt,EnvFilter,Registry};
use tracing_log::LogTracer;
use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink
) -> impl Subscriber + Send + Sync
    where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    return subscriber;
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("failed to set logger");

    subscriber::set_global_default(subscriber).expect("failed to set subscriber");
}