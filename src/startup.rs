use std::net::TcpListener;
use actix_web::{HttpServer, web, App};
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use crate::routes::{subscribe,health_check};
use sqlx::PgPool;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check",web::get().to(health_check))
            .route("/subscriptions",web::post().to(subscribe))
            .app_data(db_pool.clone())
        })
        .listen(listener)?
        .run();

    Ok(server)
}
