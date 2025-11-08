use axum::{middleware, Extension, Router};
use std::sync::Arc;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use rapidfab_api::{app, config::Config, db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with JSON logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().json())
        .init();

    // Register Prometheus metrics
    app::metrics::prometheus::register_metrics();
    tracing::info!("Prometheus metrics registered");

    // Load configuration from environment
    let config = Config::from_env()?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        env = %config.rust_env,
        "Starting RapidFab API"
    );

    // Create database connection pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    let config_arc = Arc::new(config);
    let pool_arc = Arc::new(pool);

    let upload_router = app::upload::routes::router().with_state(config_arc.clone());

    let app = Router::new()
        .nest("/health", app::health::routes::router())
        .nest("/auth", app::auth::routes::router())
        .nest("/users", app::users::routes::router())
        .nest("/files", upload_router)
        .merge(app::metrics::routes::router())
        .layer(middleware::from_fn(app::session_middleware))
        .layer(middleware::from_fn(
            rapidfab_api::middleware::metrics::track_metrics,
        ))
        .layer(Extension(pool_arc));

    // Start server
    let addr = format!("{}:{}", config_arc.api_host, config_arc.api_port);
    tracing::info!(addr = %addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
