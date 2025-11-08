use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub file_url: String,
    pub material: String,
    pub infill: u8,           // 10-100 (percentage)
    pub layer_thickness: u16,  // micrometers (100, 200, 300)
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub quote_id: Uuid,
    pub total_usd: f64,
    pub material_cost_usd: f64,
    pub machine_cost_usd: f64,
    pub base_fee_usd: f64,
    pub lead_time_days: u8,
    pub print_time_hours: f64,
    pub filament_weight_g: f64,
    pub volume_cm3: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl QuoteRequest {
    pub fn validate(&self) -> Result<(), String> {
        // Validate material
        if !["pla", "abs", "petg", "abs-esd", "asa", "nylon", "pc", "tpu"]
            .contains(&self.material.to_lowercase().as_str())
        {
            return Err(format!("Invalid material: {}", self.material));
        }

        // Validate infill
        if self.infill < 10 || self.infill > 100 {
            return Err(format!("Infill must be between 10-100%, got: {}", self.infill));
        }

        // Validate layer thickness
        if ![100, 200, 300].contains(&self.layer_thickness) {
            return Err(format!(
                "Layer thickness must be 100, 200, or 300 micrometers, got: {}",
                self.layer_thickness
            ));
        }

        // Validate file_url
        if self.file_url.is_empty() {
            return Err("file_url cannot be empty".to_string());
        }

        Ok(())
    }

    pub fn layer_height_mm(&self) -> f32 {
        self.layer_thickness as f32 / 1000.0
    }

    pub fn quality_preset(&self) -> &'static str {
        match self.layer_thickness {
            100 => "fine",
            200 => "standard",
            300 => "economy",
            _ => "standard",
        }
    }
}
