use std::net::TcpListener;
use actix_web::{HttpServer, web, App};
use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use crate::configuration::Settings;
use crate::routes::{subscribe,health_check,confirm};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::{EmailClient};
use crate::configuration::DatabaseSettings;

pub struct Application {
    port: u16,
    server: Server
}

impl Application {
    pub async fn build(config: &Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&config.database);

        let sender_email = config
            .email_client
            .sender()
            .expect("invalid sender email address.");

        let timeout = config.email_client.timeout();

        let email_client = EmailClient::new(
                config.email_client.base_url.clone(),
                sender_email,
                config.email_client.authorization_token.clone(),
                timeout
            );

        let address = format!("{}:{}",&config.application.host,&config.application.port);

        let listener = TcpListener::bind(&address)?;

        let port = listener.local_addr().unwrap().port();

        let server = run(listener,connection_pool, email_client)?;

        Ok(Self {port, server})
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(),std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(conf: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(conf.with_db())
}

fn run(listener: TcpListener, connection_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check",web::get().to(health_check))
            .route("/subscriptions",web::post().to(subscribe))
            .route("/subscriptions/confirm",web::post().to(confirm))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
        })
        .listen(listener)?
        .run();

    Ok(server)
}
