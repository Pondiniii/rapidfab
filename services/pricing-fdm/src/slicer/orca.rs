use super::{parser, SliceMetrics};
use crate::config::Config;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};
use uuid::Uuid;

pub async fn slice(
    stl_path: &Path,
    material: &str,
    _infill: u8,
    _layer_height: f32,
    config: &Config,
) -> Result<SliceMetrics> {
    // Validate input file exists
    if !stl_path.exists() {
        bail!("STL file not found: {:?}", stl_path);
    }

    // Create unique output directory
    let output_id = Uuid::new_v4();
    let output_dir = Path::new(&config.temp_dir).join(format!("slice-{}", output_id));
    tokio::fs::create_dir_all(&output_dir).await?;

    let output_3mf = output_dir.join("result.3mf");

    // Build Orca Slicer command
    // For MVP: use dummy/default profiles
    // TODO: Map material/infill/layer_height to specific profile
    let machine_profile = Path::new(&config.orca_profiles_dir).join("machine.json");
    let process_profile = Path::new(&config.orca_profiles_dir).join("process_standard.json");
    let filament_profile = Path::new(&config.orca_profiles_dir).join(format!("filament_{}.json", material));

    // Fallback to generic PLA if material-specific profile doesn't exist
    let filament_profile = if filament_profile.exists() {
        filament_profile
    } else {
        Path::new(&config.orca_profiles_dir).join("filament_pla.json")
    };

    debug!(
        "Slicing with profiles: machine={:?}, process={:?}, filament={:?}",
        machine_profile, process_profile, filament_profile
    );

    // Execute xvfb-run orca-slicer
    let output = Command::new("xvfb-run")
        .arg("-a") // Auto-select display number
        .arg(&config.orca_binary)
        .arg("--datadir")
        .arg(&config.orca_profiles_dir)
        .arg("--load-settings")
        .arg(format!(
            "{};{}",
            machine_profile.display(),
            process_profile.display()
        ))
        .arg("--load-filaments")
        .arg(filament_profile.display().to_string())
        .arg("--arrange")
        .arg("1")
        .arg("--orient")
        .arg("1")
        .arg("--slice")
        .arg("0")
        .arg("--export-3mf")
        .arg(&output_3mf)
        .arg("--outputdir")
        .arg(&output_dir)
        .arg(stl_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("Failed to execute orca-slicer command")?;

    // Check if slicing succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Orca Slicer failed: {}", stderr);
    }

    // Check result.json for return code
    let result_json_path = output_dir.join("result.json");
    if result_json_path.exists() {
        let result_json = tokio::fs::read_to_string(&result_json_path).await?;
        let result: serde_json::Value = serde_json::from_str(&result_json)?;

        if let Some(return_code) = result.get("return_code").and_then(|v| v.as_i64()) {
            if return_code != 0 {
                let error_msg = result
                    .get("error_string")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                bail!("Slicing failed with code {}: {}", return_code, error_msg);
            }
        }
    }

    // Check if 3MF was created
    if !output_3mf.exists() {
        bail!("Orca Slicer did not produce 3MF output");
    }

    info!("Slicing completed, extracting metrics from 3MF");

    // Extract 3MF and parse G-code
    let metrics = parser::extract_metrics(&output_3mf).await?;

    // Cleanup
    if let Err(e) = tokio::fs::remove_dir_all(&output_dir).await {
        debug!("Failed to cleanup temp directory: {}", e);
    }

    Ok(metrics)
}
