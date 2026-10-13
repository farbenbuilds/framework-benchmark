use ntex::web::{self, App, HttpResponse, HttpServer, Responder};

const JSON: &[u8] = b"{\"message\":\"Hello, World!\"}";

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
    HttpResponse::Ok()
        .content_type("application/json")
        .body(JSON)
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(healthz).service(plaintext).service(json))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
