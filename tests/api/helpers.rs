use std::net::TcpListener;
use newsletter::email_client::EmailClient;
use newsletter::startup::run;
use newsletter::telemetry;
use newsletter::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, Executor, PgConnection, Connection};
use uuid::Uuid;
use once_cell::sync::Lazy;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(||{
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level,std::io::stdout);
        telemetry::init_subscriber(subscriber)
    } else {
        let subscriber = telemetry::get_subscriber(subscriber_name, default_filter_level,std::io::sink);
        telemetry::init_subscriber(subscriber)
    }
});

// Spawns an instance of the app. It binds to a random port.
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("could not load config");

    config.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    let db_pool = configure_database(&config.database).await;

    let sender_email = config.email_client.sender()
        .expect("invalid sender email address.");

    let timeout = config.email_client.timeout();

    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token,
        timeout
    );

    let server = run(
        listener, 
        db_pool.clone(),
        email_client
    )
    .expect("failed to bind address.");

    let _ = tokio::spawn(server);

    let address = format!("http://127.0.0.1:{}",port);

    TestApp {
        address,
        db_pool
    }
}

// Configures the database. Creates a connection pool and runs migration.
pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to postgres.");

    // create a database
    connection.execute(
        format!(
            r#"CREATE DATABASE "{}";"#,
            config.database_name
        ).as_str()
    )
        .await
        .expect("failed to created database.");

    let connection_pool = PgPool::connect_with(config.with_db()).await.expect("failed to connect to pool.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("failed to migrate.");

    return connection_pool;
}