use pricing_fdm::*;

use anyhow::Result;
use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .json()
        .init();

    // Load configuration
    let config = config::Config::from_env()?;
    info!("Loaded configuration: {:?}", config.masked());

    // Create app state
    let app_state = AppState {
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/internal/pricing/fdm/quote", post(app::handlers::quote))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("FDM Pricing Service listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn metrics() -> &'static str {
    // TODO: Implement Prometheus metrics
    "# Placeholder for metrics\n"
}
