mod orca;
mod parser;

use crate::config::Config;
use anyhow::Result;
use std::path::Path;

pub struct SliceMetrics {
    pub print_time_hours: f64,
    pub filament_weight_g: f64,
    pub filament_length_mm: f64,
    pub volume_cm3: f64,
}

pub async fn slice_model(
    stl_path: &Path,
    material: &str,
    infill: u8,
    layer_height: f32,
    config: &Config,
) -> Result<SliceMetrics> {
    orca::slice(stl_path, material, infill, layer_height, config).await
}
