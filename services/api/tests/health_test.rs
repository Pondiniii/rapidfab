//! Health Endpoint Integration Tests
//!
//! These tests are marked with `#[ignore]` and are NOT run during CI.
//!
//! ## Why ignored?
//! - Require running server (http://localhost:8080)
//! - CI runs these tests BEFORE Docker stack starts
//! - Alternative: E2E bash tests (coming soon) or manual testing
//!
//! ## How to run manually:
//! ```bash
//! # Start services first
//! docker-compose -f docker-compose.minimal.yml up -d
//!
//! # Run health tests
//! cargo test --test health_test -- --ignored --test-threads=1
//! ```

#[cfg(test)]
mod tests {
    use serde_json::Value;

    const BASE_URL: &str = "http://localhost:8080";

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_healthz_returns_healthy_status() {
        let res = reqwest::get(format!("{BASE_URL}/health/healthz"))
            .await
            .expect("Failed to make request");

        assert_eq!(res.status(), 200, "Expected 200 OK status");

        let body: Value = res.json().await.expect("Failed to parse JSON");

        assert_eq!(body["status"], "healthy", "Expected healthy status");
        assert!(
            body["version"].is_string(),
            "Expected version to be a string"
        );
        assert!(
            body["timestamp"].is_string(),
            "Expected timestamp to be a string"
        );
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_readyz_checks_database() {
        let res = reqwest::get(format!("{BASE_URL}/health/readyz"))
            .await
            .expect("Failed to make request");

        assert_eq!(res.status(), 200, "Expected 200 OK status");

        let body: Value = res.json().await.expect("Failed to parse JSON");

        assert_eq!(body["status"], "ready", "Expected ready status");
        assert_eq!(
            body["checks"]["database"], "ok",
            "Expected database check to be ok"
        );
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_metrics_endpoint_returns_prometheus_format() {
        let res = reqwest::get(format!("{BASE_URL}/metrics"))
            .await
            .expect("Failed to make request");

        assert_eq!(res.status(), 200, "Expected 200 OK status");

        let content_type = res
            .headers()
            .get("content-type")
            .expect("Expected content-type header");
        assert!(
            content_type.to_str().unwrap().starts_with("text/plain"),
            "Expected text/plain content type"
        );

        let body = res.text().await.expect("Failed to get response text");

        // Verify that key metrics are present
        assert!(
            body.contains("http_requests_total"),
            "Expected http_requests_total metric"
        );
        assert!(
            body.contains("http_request_duration_seconds"),
            "Expected http_request_duration_seconds metric"
        );
        assert!(
            body.contains("db_connections_active"),
            "Expected db_connections_active metric"
        );
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_metrics_track_requests() {
        // Make a request to healthz
        let _ = reqwest::get(format!("{BASE_URL}/health/healthz"))
            .await
            .expect("Failed to make request");

        // Check that metrics recorded the request
        let res = reqwest::get(format!("{BASE_URL}/metrics"))
            .await
            .expect("Failed to make request");

        let body = res.text().await.expect("Failed to get response text");

        // Verify that the healthz request was tracked
        assert!(
            body.contains("/health/healthz"),
            "Expected /health/healthz to be tracked in metrics"
        );
    }
}
