use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

use crate::app::metrics::prometheus::{HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION_SECONDS};

/// Metrics tracking middleware
/// Records HTTP request count and duration for all requests
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();

    // Increment request counter with labels
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[&method, &path, &status])
        .inc();

    // Record request duration
    HTTP_REQUEST_DURATION_SECONDS
        .with_label_values(&[&method, &path])
        .observe(duration);

    response
}
