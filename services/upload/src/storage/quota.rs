use anyhow::{bail, Result};
use chrono::{NaiveDate, Utc};
use sqlx::PgPool;

/// Quota limits from config
// Allow dead code until quota endpoints are implemented
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct QuotaLimits {
    pub anon_daily_mb: u64,
    pub user_monthly_gb: u64,
    pub user_hourly_gb: u64,
    pub ip_daily_mb: u64,
}

/// Check if anonymous session can upload
// Allow dead code until quota endpoints are implemented
#[allow(dead_code)]
pub async fn check_anon_quota(
    pool: &PgPool,
    session_id: &str,
    ip: &str,
    bytes: u64,
    limits: &QuotaLimits,
) -> Result<()> {
    let today = Utc::now().date_naive();

    // Check session quota (100MB/day default)
    let session_used = get_session_usage(pool, session_id, today).await?;
    let session_limit = limits.anon_daily_mb * 1024 * 1024;
    if session_used + bytes > session_limit {
        bail!(
            "session quota exceeded: {}/{} MB",
            session_used / 1024 / 1024,
            limits.anon_daily_mb
        );
    }

    // Check IP quota (500MB/day default)
    let ip_used = get_ip_usage(pool, ip, today).await?;
    let ip_limit = limits.ip_daily_mb * 1024 * 1024;
    if ip_used + bytes > ip_limit {
        bail!(
            "IP quota exceeded: {}/{} MB",
            ip_used / 1024 / 1024,
            limits.ip_daily_mb
        );
    }

    Ok(())
}

/// Check if authenticated user can upload
// Allow dead code until quota endpoints are implemented
#[allow(dead_code)]
pub async fn check_user_quota(
    pool: &PgPool,
    user_id: &str,
    bytes: u64,
    limits: &QuotaLimits,
) -> Result<()> {
    // Check total quota (20GB default)
    let total_used = get_user_total_usage(pool, user_id).await?;
    let total_limit = limits.user_monthly_gb * 1024 * 1024 * 1024;
    if total_used + bytes > total_limit {
        bail!(
            "user quota exceeded: {}/{} GB",
            total_used / 1024 / 1024 / 1024,
            limits.user_monthly_gb
        );
    }

    // Check hourly rate limit (2GB/hour default)
    let hourly_used = get_user_hourly_usage(pool, user_id).await?;
    let hourly_limit = limits.user_hourly_gb * 1024 * 1024 * 1024;
    if hourly_used + bytes > hourly_limit {
        bail!(
            "hourly rate limit exceeded: {}/{} GB/hour",
            hourly_used / 1024 / 1024 / 1024,
            limits.user_hourly_gb
        );
    }

    Ok(())
}

/// Update quota after successful upload (anon)
// Allow dead code until quota endpoints are implemented
#[allow(dead_code)]
pub async fn update_anon_quota(
    pool: &PgPool,
    session_id: &str,
    ip: &str,
    bytes: u64,
) -> Result<()> {
    let today = Utc::now().date_naive();

    // Parse session_id as UUID
    let session_uuid = uuid::Uuid::parse_str(session_id)?;

    // Update session quota
    sqlx::query(
        r#"
        INSERT INTO upload_quotas (session_id, bytes_used, period_start)
        VALUES ($1, $2, $3)
        ON CONFLICT (session_id, period_start)
        DO UPDATE SET bytes_used = upload_quotas.bytes_used + $2
        "#,
    )
    .bind(session_uuid)
    .bind(bytes as i64)
    .bind(today)
    .execute(pool)
    .await?;

    // Update IP quota
    sqlx::query(
        r#"
        INSERT INTO ip_quotas (ip_address, bytes_used, period_start)
        VALUES ($1, $2, $3)
        ON CONFLICT (ip_address, period_start)
        DO UPDATE SET bytes_used = ip_quotas.bytes_used + $2
        "#,
    )
    .bind(ip)
    .bind(bytes as i64)
    .bind(today)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update quota after successful upload (user)
// Allow dead code until quota endpoints are implemented
#[allow(dead_code)]
pub async fn update_user_quota(pool: &PgPool, user_id: &str, bytes: u64) -> Result<()> {
    let today = Utc::now().date_naive();

    // Parse user_id as UUID
    let user_uuid = uuid::Uuid::parse_str(user_id)?;

    sqlx::query(
        r#"
        INSERT INTO upload_quotas (user_id, bytes_used, period_start)
        VALUES ($1, $2, $3)
        ON CONFLICT (user_id, period_start)
        DO UPDATE SET bytes_used = upload_quotas.bytes_used + $2
        "#,
    )
    .bind(user_uuid)
    .bind(bytes as i64)
    .bind(today)
    .execute(pool)
    .await?;

    Ok(())
}

// Helper functions (private)

async fn get_session_usage(pool: &PgPool, session_id: &str, date: NaiveDate) -> Result<u64> {
    let session_uuid = uuid::Uuid::parse_str(session_id)?;

    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT bytes_used FROM upload_quotas
        WHERE session_id = $1 AND period_start = $2
        "#,
    )
    .bind(session_uuid)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(bytes,)| bytes as u64).unwrap_or(0))
}

async fn get_ip_usage(pool: &PgPool, ip: &str, date: NaiveDate) -> Result<u64> {
    let row: Option<(i64,)> = sqlx::query_as(
        r#"
        SELECT bytes_used FROM ip_quotas
        WHERE ip_address = $1 AND period_start = $2
        "#,
    )
    .bind(ip)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(bytes,)| bytes as u64).unwrap_or(0))
}

async fn get_user_total_usage(pool: &PgPool, user_id: &str) -> Result<u64> {
    let user_uuid = uuid::Uuid::parse_str(user_id)?;

    let row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(bytes_used), 0) as total
        FROM upload_quotas
        WHERE user_id = $1
        "#,
    )
    .bind(user_uuid)
    .fetch_one(pool)
    .await?;

    Ok(row.0.unwrap_or(0) as u64)
}

async fn get_user_hourly_usage(pool: &PgPool, user_id: &str) -> Result<u64> {
    let user_uuid = uuid::Uuid::parse_str(user_id)?;
    let one_hour_ago = Utc::now() - chrono::Duration::hours(1);

    let row: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(size_bytes), 0) as total
        FROM files f
        JOIN uploads u ON f.upload_id = u.id
        WHERE u.user_id = $1 AND f.created_at > $2
        "#,
    )
    .bind(user_uuid)
    .bind(one_hour_ago)
    .fetch_one(pool)
    .await?;

    Ok(row.0.unwrap_or(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_limits_construction() {
        let limits = QuotaLimits {
            anon_daily_mb: 100,
            user_monthly_gb: 20,
            user_hourly_gb: 2,
            ip_daily_mb: 500,
        };

        assert_eq!(limits.anon_daily_mb, 100);
        assert_eq!(limits.user_monthly_gb, 20);
        assert_eq!(limits.user_hourly_gb, 2);
        assert_eq!(limits.ip_daily_mb, 500);
    }

    #[test]
    fn test_quota_limit_conversions() {
        let limits = QuotaLimits {
            anon_daily_mb: 100,
            user_monthly_gb: 20,
            user_hourly_gb: 2,
            ip_daily_mb: 500,
        };

        // Test MB to bytes conversion
        let session_limit_bytes = limits.anon_daily_mb * 1024 * 1024;
        assert_eq!(session_limit_bytes, 104_857_600); // 100MB in bytes

        // Test GB to bytes conversion
        let user_limit_bytes = limits.user_monthly_gb * 1024 * 1024 * 1024;
        assert_eq!(user_limit_bytes, 21_474_836_480); // 20GB in bytes
    }
}
