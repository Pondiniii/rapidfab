// Upload Service - File upload management and S3 integration

mod auth;
mod config;
mod metrics;
mod storage;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::storage::{QuotaLimits, S3Client};

// Allow dead code until endpoints are implemented
#[allow(dead_code)]
#[derive(Clone)]
struct AppState {
    s3_client: Arc<S3Client>,
    db_pool: sqlx::PgPool,
    quota_limits: QuotaLimits,
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

/// Prometheus metrics endpoint
async fn metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
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

    // Initialize database connection pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!("Database connection pool initialized");

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

    // Create quota limits from config
    let quota_limits = QuotaLimits {
        anon_daily_mb: config.limits.quota_anon_daily_mb,
        user_monthly_gb: config.limits.quota_user_monthly_gb,
        user_hourly_gb: config.limits.user_hourly_gb,
        ip_daily_mb: config.limits.ip_daily_mb,
    };

    // Create shared app state
    let _app_state = AppState {
        s3_client: Arc::new(s3_client),
        db_pool,
        quota_limits,
        config: Arc::new(config.clone()),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics));
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
