use super::SliceMetrics;
use anyhow::{bail, Context, Result};
use regex::Regex;
use std::path::Path;
use tracing::debug;

pub async fn extract_metrics(three_mf_path: &Path) -> Result<SliceMetrics> {
    // 3MF is a ZIP archive, extract it
    let temp_extract_dir = three_mf_path.with_extension("extracted");
    tokio::fs::create_dir_all(&temp_extract_dir).await?;

    // Extract 3MF using zip crate
    let file = std::fs::File::open(three_mf_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_extract_dir.join(file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    // Look for G-code file in Metadata/plate_1.gcode
    let gcode_path = temp_extract_dir.join("Metadata/plate_1.gcode");
    if !gcode_path.exists() {
        bail!("G-code not found in 3MF archive");
    }

    debug!("Parsing G-code from: {:?}", gcode_path);

    // Read G-code and extract metrics from comments
    let gcode_content = tokio::fs::read_to_string(&gcode_path).await?;

    let metrics = parse_gcode_comments(&gcode_content)?;

    // Cleanup temp extraction directory
    if let Err(e) = tokio::fs::remove_dir_all(&temp_extract_dir).await {
        debug!("Failed to cleanup extraction directory: {}", e);
    }

    Ok(metrics)
}

fn parse_gcode_comments(gcode: &str) -> Result<SliceMetrics> {
    // Regex patterns for common slicer comment formats
    let re_time = Regex::new(r"; estimated printing time.*?=\s*(?:(\d+)h\s*)?(?:(\d+)m)?\s*(?:(\d+)s)?")
        .unwrap();
    let re_filament_g = Regex::new(r"; filament used \[g\]\s*=\s*([\d.]+)").unwrap();
    let re_filament_mm = Regex::new(r"; filament used \[mm\]\s*=\s*([\d.]+)").unwrap();
    let re_filament_cm3 = Regex::new(r"; filament used \[cm3\]\s*=\s*([\d.]+)").unwrap();

    // Alternative patterns (OrcaSlicer may use different formats)
    let re_filament_m = Regex::new(r"; filament used.*?\[m\]\s*=\s*([\d.]+)").unwrap();

    // Extract print time
    let print_time_hours = if let Some(cap) = re_time.captures(gcode) {
        let hours = cap.get(1).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);
        let minutes = cap.get(2).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);
        let seconds = cap.get(3).and_then(|m| m.as_str().parse::<u32>().ok()).unwrap_or(0);

        hours as f64 + (minutes as f64 / 60.0) + (seconds as f64 / 3600.0)
    } else {
        bail!("Could not extract print time from G-code");
    };

    // Extract filament weight (grams)
    let filament_weight_g = re_filament_g
        .captures(gcode)
        .and_then(|cap| cap.get(1))
        .and_then(|m| m.as_str().parse::<f64>().ok())
        .context("Could not extract filament weight from G-code")?;

    // Extract filament length (mm or m)
    let filament_length_mm = if let Some(cap) = re_filament_mm.captures(gcode) {
        cap.get(1)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0)
    } else if let Some(cap) = re_filament_m.captures(gcode) {
        // Convert meters to millimeters
        cap.get(1)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or(0.0)
            * 1000.0
    } else {
        // Estimate from weight if not available (assume PLA density 1.24 g/cm³, 1.75mm filament)
        let filament_radius_cm = 0.175 / 2.0; // 1.75mm diameter
        let filament_area_cm2 = std::f64::consts::PI * filament_radius_cm * filament_radius_cm;
        let filament_density = 1.24; // g/cm³
        let volume_cm3 = filament_weight_g / filament_density;
        let length_cm = volume_cm3 / filament_area_cm2;
        length_cm * 10.0 // Convert cm to mm
    };

    // Extract volume
    let volume_cm3 = if let Some(cap) = re_filament_cm3.captures(gcode) {
        cap.get(1)
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .unwrap_or_else(|| {
                // Estimate from weight (assume PLA density)
                filament_weight_g / 1.24
            })
    } else {
        // Estimate from weight
        filament_weight_g / 1.24
    };

    debug!(
        "Extracted metrics: time={}h, weight={}g, length={}mm, volume={}cm³",
        print_time_hours, filament_weight_g, filament_length_mm, volume_cm3
    );

    Ok(SliceMetrics {
        print_time_hours,
        filament_weight_g,
        filament_length_mm,
        volume_cm3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gcode_comments() {
        let gcode = r#"
; estimated printing time (normal mode) = 2h 30m 45s
; filament used [g] = 125.5
; filament used [mm] = 41234.56
; filament used [cm3] = 101.2
"#;

        let metrics = parse_gcode_comments(gcode).unwrap();
        assert!((metrics.print_time_hours - 2.5125).abs() < 0.001);
        assert!((metrics.filament_weight_g - 125.5).abs() < 0.001);
        assert!((metrics.filament_length_mm - 41234.56).abs() < 0.001);
        assert!((metrics.volume_cm3 - 101.2).abs() < 0.001);
    }

    #[test]
    fn test_parse_gcode_minimal() {
        let gcode = r#"
; estimated printing time (normal mode) = 45m 30s
; filament used [g] = 50.0
"#;

        let metrics = parse_gcode_comments(gcode).unwrap();
        assert!((metrics.print_time_hours - 0.758).abs() < 0.01);
        assert!((metrics.filament_weight_g - 50.0).abs() < 0.001);
    }
}
