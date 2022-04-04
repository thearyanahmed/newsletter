use newsletter::startup::{get_connection_pool, Application};
use newsletter::telemetry;
use newsletter::configuration::{get_configuration, DatabaseSettings};
use sqlx::{PgPool, Executor, PgConnection, Connection};
use uuid::Uuid;
use once_cell::sync::Lazy;
use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub port: u16,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions",&self.address))
            .header("Content-Type","application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let html = self.get_link(&body["html_body"].as_str().unwrap());
        let plain_text = self.get_link(&body["text_body"].as_str().unwrap());

        ConfirmationLinks { html, plain_text }
    }

    fn get_link(&self, s: &str) -> reqwest::Url {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(),1);

        let raw_link = links[0].as_str().to_owned();
        let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

        assert_eq!(confirmation_link.host_str().unwrap(),"127.0.0.1");

        confirmation_link.set_port(Some(self.port)).unwrap();

        confirmation_link
    }
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

    let email_server = MockServer::start().await;

    let config = {
        let mut config = get_configuration().expect("could not load config");

        config.database.database_name = Uuid::new_v4().to_string();
        config.application.port = 0;

        config.email_client.base_url = email_server.uri();
        
        config
    };

    configure_database(&config.database).await;

    let app = Application::build(&config)
        .await
        .expect("failed to build application.");

    let port = app.port();

    let address = format!("http://localhost:{}",port);

    let _ = tokio::spawn(app.run_until_stopped());

    let db_pool = get_connection_pool(&config.database);

    println!("address is {}",address);

    TestApp {
        port,
        address,
        db_pool,
        email_server,
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