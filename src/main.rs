use std::net::TcpListener;
use newsletter::startup::run;
use newsletter::configuration::get_configuration;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("failed to read configuration.");

    let address = format!("127.0.0.1:{}",config.application_port);

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("database connection failed.");

    let listener = TcpListener::bind(address.clone()).expect("failed to bind to port.");

    println!("running on {}",address);

    run(listener,connection_pool)?.await
}
