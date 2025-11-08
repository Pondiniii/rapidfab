// Upload Service - File upload management and S3 integration

mod auth;
mod config;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;

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

    // Load configuration
    let config = Config::from_env()?;

    // Log masked config (hides secrets)
    info!("Upload service configuration loaded: {:?}", config.masked());

    // Build router
    let app = Router::new().route("/health", get(health));
    // TODO: Add upload routes:
    //   POST /internal/upload/init
    //   POST /internal/upload/{id}/signed-urls
    //   POST /internal/upload/{id}/confirm
    //   POST /internal/upload/transfer
    //   GET /internal/upload/file/{id}/read-url

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    info!("Starting upload service on {}:{}", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
