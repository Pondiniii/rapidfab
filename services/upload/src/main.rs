// Upload Service - File upload management and S3 integration

mod auth;
mod config;
mod storage;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::storage::S3Client;

// Allow dead code until endpoints are implemented
#[allow(dead_code)]
#[derive(Clone)]
struct AppState {
    s3_client: Arc<S3Client>,
    config: Arc<Config>,
}

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

    // Initialize S3 client
    let s3_client = S3Client::new(
        &config.s3.endpoint,
        &config.s3.bucket,
        &config.s3.region,
        &config.s3.access_key_id,
        &config.s3.secret_access_key,
    )
    .await?;

    info!("S3 client initialized: bucket={}", config.s3.bucket);

    // Create shared app state
    let _app_state = AppState {
        s3_client: Arc::new(s3_client),
        config: Arc::new(config.clone()),
    };

    // Build router
    let app = Router::new().route("/health", get(health));
    // TODO: Add upload routes with app_state:
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
