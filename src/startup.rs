use std::net::TcpListener;
use actix_web::{HttpServer, web, App};
use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use crate::configuration::Settings;
use crate::routes::{subscribe,health_check};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::email_client::{EmailClient};

pub async fn build(config: Settings) -> Result<Server, std::io::Error> {
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.database.with_db());
    
    let sender_email = config
        .email_client
        .sender()
        .expect("invalid sender email address.");

    let timeout = config.email_client.timeout();

    let email_client = EmailClient::new(
            config.email_client.base_url,
            sender_email,
            config.email_client.authorization_token,
            timeout
        );

    let address = format!("127.0.0.1:{}",&config.application.port);

    let listener = TcpListener::bind(address)?;

    run(listener,connection_pool, email_client)
}

pub fn run(listener: TcpListener, connection_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check",web::get().to(health_check))
            .route("/subscriptions",web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
        })
        .listen(listener)?
        .run();

    Ok(server)
}
