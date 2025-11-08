use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // Service
    pub host: String,
    pub port: u16,

    // Orca Slicer
    pub orca_profiles_dir: String,
    pub orca_binary: String,
    pub temp_dir: String,

    // Pricing parameters
    pub base_fee_usd: f64,
    pub machine_rate_usd_per_hour: f64,
    pub margin_multiplier: f64,

    // Material costs (per gram)
    pub material_costs: MaterialCosts,

    // Request limits
    pub max_file_size_mb: u64,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterialCosts {
    pub pla: f64,
    pub abs: f64,
    pub petg: f64,
    pub abs_esd: f64,
    pub asa: f64,
    pub nylon: f64,
    pub pc: f64,
    pub tpu: f64,
}

impl MaterialCosts {
    pub fn get(&self, material: &str) -> Option<f64> {
        match material.to_lowercase().as_str() {
            "pla" => Some(self.pla),
            "abs" => Some(self.abs),
            "petg" => Some(self.petg),
            "abs-esd" => Some(self.abs_esd),
            "asa" => Some(self.asa),
            "nylon" => Some(self.nylon),
            "pc" => Some(self.pc),
            "tpu" => Some(self.tpu),
            _ => None,
        }
    }

    pub fn all_materials() -> Vec<&'static str> {
        vec!["pla", "abs", "petg", "abs-esd", "asa", "nylon", "pc", "tpu"]
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Config {
            host: std::env::var("PRICING_FDM_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PRICING_FDM_PORT")
                .unwrap_or_else(|_| "8083".to_string())
                .parse()
                .context("PRICING_FDM_PORT must be a valid u16")?,

            orca_profiles_dir: std::env::var("ORCA_PROFILES_DIR")
                .unwrap_or_else(|_| "/app/profiles".to_string()),
            orca_binary: std::env::var("ORCA_BINARY")
                .unwrap_or_else(|_| "orca-slicer".to_string()),
            temp_dir: std::env::var("TEMP_DIR")
                .unwrap_or_else(|_| "/tmp".to_string()),

            base_fee_usd: std::env::var("BASE_FEE_USD")
                .unwrap_or_else(|_| "5.00".to_string())
                .parse()
                .context("BASE_FEE_USD must be a valid f64")?,
            machine_rate_usd_per_hour: std::env::var("MACHINE_RATE_USD_PER_HOUR")
                .unwrap_or_else(|_| "10.00".to_string())
                .parse()
                .context("MACHINE_RATE_USD_PER_HOUR must be a valid f64")?,
            margin_multiplier: std::env::var("MARGIN_MULTIPLIER")
                .unwrap_or_else(|_| "1.30".to_string())
                .parse()
                .context("MARGIN_MULTIPLIER must be a valid f64")?,

            material_costs: MaterialCosts {
                pla: Self::parse_env_f64("MATERIAL_PLA_COST_PER_G", 0.02)?,
                abs: Self::parse_env_f64("MATERIAL_ABS_COST_PER_G", 0.025)?,
                petg: Self::parse_env_f64("MATERIAL_PETG_COST_PER_G", 0.03)?,
                abs_esd: Self::parse_env_f64("MATERIAL_ABS_ESD_COST_PER_G", 0.035)?,
                asa: Self::parse_env_f64("MATERIAL_ASA_COST_PER_G", 0.028)?,
                nylon: Self::parse_env_f64("MATERIAL_NYLON_COST_PER_G", 0.04)?,
                pc: Self::parse_env_f64("MATERIAL_PC_COST_PER_G", 0.045)?,
                tpu: Self::parse_env_f64("MATERIAL_TPU_COST_PER_G", 0.035)?,
            },

            max_file_size_mb: std::env::var("MAX_FILE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .context("MAX_FILE_SIZE_MB must be a valid u64")?,
            request_timeout_secs: std::env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("REQUEST_TIMEOUT_SECS must be a valid u64")?,
        };

        Ok(config)
    }

    fn parse_env_f64(var_name: &str, default: f64) -> Result<f64> {
        std::env::var(var_name)
            .unwrap_or_else(|_| default.to_string())
            .parse()
            .with_context(|| format!("{} must be a valid f64", var_name))
    }

    /// Masked config for logging (hide nothing sensitive here, but keep pattern)
    pub fn masked(&self) -> MaskedConfig {
        MaskedConfig {
            host: self.host.clone(),
            port: self.port,
            orca_profiles_dir: self.orca_profiles_dir.clone(),
            orca_binary: self.orca_binary.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaskedConfig {
    pub host: String,
    pub port: u16,
    pub orca_profiles_dir: String,
    pub orca_binary: String,
}
