// Upload Service - File upload management and S3 integration

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    service: String,
}

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "upload".to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Build router
    let app = Router::new().route("/health", get(health));
    // TODO: Add upload routes:
    //   POST /internal/upload/init
    //   POST /internal/upload/{id}/signed-urls
    //   POST /internal/upload/{id}/confirm
    //   POST /internal/upload/transfer
    //   GET /internal/upload/file/{id}/read-url

    // Start server
    let host = std::env::var("UPLOAD_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("UPLOAD_PORT")
        .unwrap_or_else(|_| "8082".to_string())
        .parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!("Starting upload service on {}:{}", host, port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
