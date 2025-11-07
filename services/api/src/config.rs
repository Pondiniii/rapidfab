use serde::Deserialize;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub rust_env: String,
    pub api_host: String,
    pub api_port: u16,
    pub database_url: String,
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
        })
    }
}
