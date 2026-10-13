mod database;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use database::AppState;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use std::env;
use std::sync::atomic::AtomicU64;
use tokio_postgres::NoTls;

const PLAINTEXT: &[u8] = b"Hello, World!";
const JSON: &[u8] = b"{\"message\":\"Hello, World!\"}";

#[get("/healthz")]
async fn healthz() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/plaintext")]
async fn plaintext() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body(web::Bytes::from_static(PLAINTEXT))
}

#[get("/json")]
async fn json() -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .body(web::Bytes::from_static(JSON))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = create_pool()?;
    let state = web::Data::new(AppState {
        pool,
        sequence: AtomicU64::new(0x9e37_79b9_7f4a_7c15),
    });
    let workers = configured_workers();
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
    let url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world".into()
    });
    let config = url.parse().map_err(std::io::Error::other)?;
    let manager = Manager::from_config(
        config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );
    Pool::builder(manager)
        .max_size(configured_pool_size())
        .build()
        .map_err(std::io::Error::other)
}

fn configured_workers() -> usize {
    env::var("BENCHMARK_WORKERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &usize| *value > 0)
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
}

fn configured_pool_size() -> usize {
    env::var("BENCHMARK_DB_POOL_SIZE")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &usize| *value > 0)
        .unwrap_or_else(|| configured_workers().saturating_mul(4).clamp(4, 128))
}
