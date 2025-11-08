use anyhow::Result;
use serde::Deserialize;

// Allow dead code until all features are implemented
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Service
    pub host: String,
    pub port: u16,

    // Database
    pub database_url: String,

    // S3
    pub s3: S3Config,

    // Upload limits
    pub limits: UploadLimits,

    // Auth
    pub upload_ticket_secret: String,
    pub internal_service_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

// Allow dead code until all features are implemented
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct UploadLimits {
    // File size limit
    pub max_file_mb: u64, // Default: 500MB

    // Quota limits
    pub quota_anon_daily_mb: u64, // Default: 100MB
    pub quota_user_monthly_gb: u64, // Default: 20GB

    // Rate limits
    pub user_hourly_gb: u64, // Default: 2GB/hour for authenticated users
    pub ip_daily_mb: u64, // Default: 500MB/day per IP (anonymous)

    // TTL
    pub anon_ttl_days: u32, // Default: 7 days
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Config {
            host: std::env::var("UPLOAD_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("UPLOAD_PORT")
                .unwrap_or_else(|_| "8082".to_string())
                .parse()?,

            database_url: std::env::var("DATABASE_URL")?,

            s3: S3Config {
                endpoint: std::env::var("S3_ENDPOINT")?,
                bucket: std::env::var("S3_BUCKET")?,
                region: std::env::var("S3_REGION")?,
                access_key_id: std::env::var("S3_ACCESS_KEY_ID")?,
                secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY")?,
            },

            limits: UploadLimits {
                max_file_mb: std::env::var("MAX_FILE_MB")
                    .unwrap_or_else(|_| "500".to_string())
                    .parse()?,
                quota_anon_daily_mb: std::env::var("QUOTA_ANON_DAILY_MB")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                quota_user_monthly_gb: std::env::var("QUOTA_USER_MONTHLY_GB")
                    .unwrap_or_else(|_| "20".to_string())
                    .parse()?,
                user_hourly_gb: std::env::var("USER_HOURLY_GB")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()?,
                ip_daily_mb: std::env::var("IP_DAILY_MB")
                    .unwrap_or_else(|_| "500".to_string())
                    .parse()?,
                anon_ttl_days: std::env::var("ANON_TTL_DAYS")
                    .unwrap_or_else(|_| "7".to_string())
                    .parse()?,
            },

            upload_ticket_secret: std::env::var("UPLOAD_TICKET_SECRET")?,
            internal_service_token: std::env::var("INTERNAL_SERVICE_TOKEN")?,
        })
    }

    /// Returns a masked version of config for logging (hides secrets)
    pub fn masked(&self) -> MaskedConfig {
        MaskedConfig {
            host: self.host.clone(),
            port: self.port,
            database_url: mask_connection_string(&self.database_url),
            s3_endpoint: self.s3.endpoint.clone(),
            s3_bucket: self.s3.bucket.clone(),
            s3_region: self.s3.region.clone(),
            limits: self.limits.clone(),
        }
    }
}

// Allow dead code until all features are implemented
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaskedConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub limits: UploadLimits,
}

fn mask_connection_string(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(scheme_end) = url.find("://") {
            format!("{}://***:***@{}", &url[..scheme_end], &url[at_pos + 1..])
        } else {
            "***".to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_connection_string() {
        let url = "postgres://user:password@localhost:5432/db";
        let masked = mask_connection_string(url);
        assert_eq!(masked, "postgres://***:***@localhost:5432/db");
        assert!(!masked.contains("password"));
    }

    #[test]
    fn test_mask_connection_string_no_credentials() {
        let url = "postgres://localhost:5432/db";
        let masked = mask_connection_string(url);
        assert_eq!(masked, url);
    }
}
