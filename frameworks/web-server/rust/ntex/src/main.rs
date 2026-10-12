use ntex::web::{self, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

#[web::get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[web::get("/plaintext")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Hello, World!")
}

#[web::get("/json")]
async fn json() -> impl Responder {
    web::types::Json(Message {
        message: "Hello, World!",
    })
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(healthz).service(plaintext).service(json))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
