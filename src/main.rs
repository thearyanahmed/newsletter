use actix_web::{web, App, HttpRequest, HttpServer, Responder, HttpResponse};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/",web::get().to(greet))
            .route("/greet/{name}",web::get().to(greet))
            .route("/health_check",web::get().to(health_check))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("world");

    format!("Hello {}!",&name)
}

async fn health_check(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}
