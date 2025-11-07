use chrono::{DateTime, Utc};
use serde::Serialize;

/// Health check response
/// Returns service status, timestamp and version
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

/// Readiness check response
/// Returns service readiness with dependency checks
#[derive(Debug, Serialize)]
pub struct ReadyResponse {
    pub status: String,
    pub checks: ReadyChecks,
}

/// Health checks for dependencies
#[derive(Debug, Serialize)]
pub struct ReadyChecks {
    pub database: String,
}
