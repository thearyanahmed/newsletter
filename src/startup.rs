use std::net::TcpListener;
use actix_web::{HttpServer, web, App};
use actix_web::dev::Server;
use crate::routes::{subscribe,health_check};
use sqlx::PgConnection;

pub fn run(listener: TcpListener, connection: PgConnection) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check",web::get().to(health_check))
            .route("/subscriptions",web::post().to(subscribe))
            .app_data(connection.clone())
        })
        .listen(listener)?
        .run();

    Ok(server)
}
