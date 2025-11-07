use axum::{routing::get, Extension, Json, Router};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;

use super::dto::*;
use crate::error::AppError;

/// Health module routes
/// Provides /healthz and /readyz endpoints
pub fn router() -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
}

/// Simple health check endpoint
/// Returns service status without checking dependencies
async fn healthz() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check endpoint
/// Verifies database connectivity before returning ready status
async fn readyz(Extension(pool): Extension<Arc<PgPool>>) -> Result<Json<ReadyResponse>, AppError> {
    // Check database connectivity with simple query
    let db_status = match sqlx::query("SELECT 1").fetch_one(pool.as_ref()).await {
        Ok(_) => "ok".to_string(),
        Err(e) => {
            tracing::error!("Database check failed: {}", e);
            return Err(AppError::Internal);
        }
    };

    Ok(Json(ReadyResponse {
        status: "ready".to_string(),
        checks: ReadyChecks {
            database: db_status,
        },
    }))
}
