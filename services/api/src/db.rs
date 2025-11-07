use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

/// Create PostgreSQL connection pool
/// Sets max_connections to 10 and acquire_timeout to 3 seconds
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

/// Run database migrations from ./migrations directory
/// Migrations are embedded at compile time using sqlx::migrate! macro
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
