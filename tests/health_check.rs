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
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut config = get_configuration().expect("could not load config");

    config.database.database_name = Uuid::new_v4().to_string();

    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    let db_pool = configure_database(&config.database).await;

    let sender_email = config.email_client.sender()
        .expect("invalid sender email address.");

    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        config.email_client.authorization_token
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

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check",&app.address))
        .send()
        .await
        .expect("failed to execute test");

    assert!(response.status().is_success());
    assert_eq!(Some(0),response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, desc) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions",&app.address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request");

            
        assert_eq!(400,response.status().as_u16(),"api did not return 400 when the payload was {}",desc)
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin","missing the email"),
        ("email=ursula_le_guin%40gmail.com","missing the name"),
        ("","missing both"),
    ];

    for (form_body, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions",&app.address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(form_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(400,response.status().as_u16(),"the api did not fail with 400 bad request when the payload was {}",error);
    }
}
