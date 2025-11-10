use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer,
    trace::TraceLayer,
    set_header::SetResponseHeaderLayer,
};
use tracing_subscriber;
use http::HeaderValue;

mod routes;
mod handlers;
mod models;
mod services;
mod middleware;

use routes::{generate, static_files};
use handlers::generate::GenerateImageHandler;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let generate_handler = Arc::new(
        GenerateImageHandler::new().expect("Failed to create generate handler")
    );

    let app = Router::new()
        .route("/", get(static_files::serve_index))
        .route("/generate", post(generate::generate_image))
        .route("/static/{*path}", get(static_files::serve_static_files))
        .fallback(static_files::serve_index)
        .with_state(generate_handler)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(SetResponseHeaderLayer::overriding(
            http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server running on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
