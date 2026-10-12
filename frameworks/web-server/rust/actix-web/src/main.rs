use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/plaintext")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Hello, World!")
}

#[get("/json")]
async fn json() -> impl Responder {
    web::Json(Message {
        message: "Hello, World!",
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(healthz).service(plaintext).service(json))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
