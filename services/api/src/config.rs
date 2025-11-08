use serde::Deserialize;

/// S3 storage configuration for file uploads
#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
}

/// Upload quota limits configuration
#[derive(Debug, Clone, Deserialize)]
pub struct QuotaConfig {
    pub anon_daily_mb: u64,
    pub user_monthly_gb: u64,
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rust_env: String,
    pub api_host: String,
    pub api_port: u16,
    pub database_url: String,
    pub s3: S3Config,
    pub quota: QuotaConfig,
}

impl Config {
    /// Load configuration from environment variables
    /// Reads .env file if present, then reads environment variables
    pub fn from_env() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        Ok(Self {
            rust_env: std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
            api_host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")?,
            s3: S3Config {
                endpoint: std::env::var("S3_ENDPOINT")?,
                bucket: std::env::var("S3_BUCKET")?,
                region: std::env::var("S3_REGION")?,
                access_key_id: std::env::var("S3_ACCESS_KEY_ID")?,
                secret_access_key: std::env::var("S3_SECRET_ACCESS_KEY")?,
            },
            quota: QuotaConfig {
                anon_daily_mb: std::env::var("QUOTA_ANON_DAILY_MB")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                user_monthly_gb: std::env::var("QUOTA_USER_MONTHLY_GB")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
            },
        })
    }
}
