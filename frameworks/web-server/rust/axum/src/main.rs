use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct MessagePayload {
    message: &'static str,
}

async fn healthz_handler() -> &'static str {
    "ok"
}

async fn plaintext_handler() -> &'static str {
    "Hello, World!"
}

async fn json_handler() -> impl IntoResponse {
    Json(MessagePayload {
        message: "Hello, World!",
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/healthz", get(healthz_handler))
        .route("/plaintext", get(plaintext_handler))
        .route("/json", get(json_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
