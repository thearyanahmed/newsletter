use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::get_configuration;
use newsletter::telemetry;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = telemetry::get_subscriber("newsletter_prod".into(),"info".into(),std::io::stdout);

    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}",config.application_port);

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&config.database.connection_string().expose_secret())
        .await
        .expect("database connection failed.");

    let listener = TcpListener::bind(address.clone()).expect("failed to bind to port.");

    println!("running on {}",address);

    run(listener,connection_pool)?.await
}
