use lazy_static::lazy_static;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry, TextEncoder,
};

lazy_static! {
    /// Global Prometheus registry for all metrics
    pub static ref REGISTRY: Registry = Registry::new();

    /// Total number of HTTP requests by method, endpoint and status
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = IntCounterVec::new(
        Opts::new("http_requests_total", "Total number of HTTP requests"),
        &["method", "endpoint", "status"]
    )
    .expect("metric can be created");

    /// HTTP request duration in seconds by method and endpoint
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration in seconds"
        )
        .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        &["method", "endpoint"]
    )
    .expect("metric can be created");

    /// Number of active database connections
    pub static ref DB_CONNECTIONS_ACTIVE: IntGauge = IntGauge::new(
        "db_connections_active",
        "Number of active database connections"
    )
    .expect("metric can be created");
}

/// Register all Prometheus metrics with the global registry
/// Must be called once at application startup
pub fn register_metrics() {
    REGISTRY
        .register(Box::new(HTTP_REQUESTS_TOTAL.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(DB_CONNECTIONS_ACTIVE.clone()))
        .expect("collector can be registered");
}

/// Encode all registered metrics to Prometheus text format
pub fn encode_metrics() -> Result<Vec<u8>, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(buffer)
}
