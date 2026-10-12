mod database;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use database::AppState;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use serde::Serialize;
use std::env;
use std::sync::atomic::AtomicU64;
use tokio_postgres::NoTls;

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
    let pool = create_pool()?;
    let state = web::Data::new(AppState {
        pool,
        sequence: AtomicU64::new(0x9e37_79b9_7f4a_7c15),
    });
    let workers = std::thread::available_parallelism().map_or(1, usize::from);
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(healthz)
            .service(plaintext)
            .service(json)
            .route("/db", web::get().to(database::db_single))
            .route("/queries", web::get().to(database::db_multi))
            .route("/fortunes", web::get().to(database::fortunes))
            .route("/updates", web::get().to(database::updates))
    })
    .workers(workers)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn create_pool() -> std::io::Result<Pool> {
    let url = env::var("DATABASE_URL").map_err(std::io::Error::other)?;
    let config = url.parse().map_err(std::io::Error::other)?;
    let manager = Manager::from_config(
        config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    Pool::builder(manager)
        .max_size(128)
        .build()
        .map_err(std::io::Error::other)
}
