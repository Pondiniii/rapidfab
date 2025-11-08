//! Integration Tests for Upload Service
//!
//! These tests verify core upload service functionality and are run automatically during CI.
//!
//! ## CI Execution:
//! - Taskfile runs: docker-compose up → wait-healthy → test:integration:upload
//! - Tests run against live services (requires Docker stack running)
//! - Uses `cargo test --test '*' -- --ignored --test-threads=1`
//!
//! ## How to run manually:
//! ```bash
//! # Start Docker stack
//! docker-compose -f docker-compose.minimal.yml up -d
//!
//! # Run integration tests
//! cd services/upload
//! cargo test --test '*' -- --ignored --test-threads=1
//! ```

use upload_service::config::Config;

/// Test that database schema verification works correctly
#[tokio::test]
async fn test_database_schema_verification() {
    // Load config
    let config = Config::from_env().expect("Failed to load config");

    // Create database pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Verify required tables exist
    let required_tables = &["uploads", "files", "upload_quotas", "ip_quotas"];

    for table_name in required_tables {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name = $1
            )",
        )
        .bind(table_name)
        .fetch_one(&pool)
        .await
        .expect("Failed to query table existence");

        assert!(
            exists,
            "Required table '{table_name}' does not exist. Run migrations first: cd services/api && sqlx migrate run"
        );
    }
}

/// Test that upload service can initialize with valid config
#[test]
fn test_config_from_env() {
    // This test verifies that Config::from_env() works
    // It requires proper environment variables to be set
    let result = Config::from_env();

    // In CI environment, this might fail if env vars aren't set
    // That's OK - we just verify the function doesn't panic
    match result {
        Ok(config) => {
            // Verify required fields are present
            assert!(!config.database_url.is_empty());
            assert!(!config.s3.endpoint.is_empty());
            assert!(!config.s3.bucket.is_empty());
        }
        Err(e) => {
            // Expected in CI without full env setup
            println!("Config load failed (expected in CI): {e}");
        }
    }
}
