use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};

use super::prometheus;

/// Metrics module routes
/// Provides /metrics endpoint for Prometheus scraping
pub fn router() -> Router {
    Router::new().route("/metrics", get(metrics))
}

/// Prometheus metrics endpoint
/// Returns all registered metrics in Prometheus text format
async fn metrics() -> Response {
    match prometheus::encode_metrics() {
        Ok(buffer) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4")],
            buffer,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to encode metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode metrics",
            )
                .into_response()
        }
    }
}
