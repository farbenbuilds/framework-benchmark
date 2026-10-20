use ntex::web::{self, App, HttpResponse, HttpServer, Responder};
use ntex::SharedCfg;

const JSON: &[u8] = b"{\"message\":\"Hello, World!\"}";

#[web::get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().build()
}

#[web::get("/plaintext")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Hello, World!")
}

#[web::get("/json")]
async fn json() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(JSON)
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(async |_| App::new().service(healthz).service(plaintext).service(json))
        .bind(("0.0.0.0", 8080), SharedCfg::new("ntex-benchmark"))?
        .run()
        .await
}
