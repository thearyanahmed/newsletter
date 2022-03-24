use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::get_configuration;
use newsletter::telemetry;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let subscriber = telemetry::get_subscriber("newsletter_prod".into(),"info".into());

    telemetry::init_subscriber(subscriber);

    let config = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}",config.application_port);

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("database connection failed.");

    let listener = TcpListener::bind(address.clone()).expect("failed to bind to port.");

    println!("running on {}",address);

    run(listener,connection_pool)?.await
}
