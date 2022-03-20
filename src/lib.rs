use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::dev::Server;
use std::net::TcpListener;

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
            App::new()
                .route("/health_check",web::get().to(health_check))
                .route("/subscriptions",web::post().to(subscribe))
        })
        .listen(listener)?
        .run();

    Ok(server)
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}