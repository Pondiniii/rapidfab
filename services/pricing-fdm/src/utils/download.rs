use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use uuid::Uuid;

pub async fn download_stl(presigned_url: &str, temp_dir: &str) -> Result<PathBuf> {
    debug!("Downloading STL from: {}", presigned_url);

    // Create unique temp file
    let file_id = Uuid::new_v4();
    let temp_path = PathBuf::from(temp_dir).join(format!("download-{}.stl", file_id));

    // Download file
    let response = reqwest::get(presigned_url)
        .await
        .context("Failed to send GET request")?;

    if !response.status().is_success() {
        bail!("Download failed with status: {}", response.status());
    }

    // Check content type (optional, but good practice)
    if let Some(content_type) = response.headers().get("content-type") {
        let ct_str = content_type.to_str().unwrap_or("");
        debug!("Content-Type: {}", ct_str);
        // Allow various STL MIME types
        if !ct_str.contains("octet-stream")
            && !ct_str.contains("stl")
            && !ct_str.contains("model")
        {
            debug!("Warning: Unexpected content-type, proceeding anyway");
        }
    }

    // Stream to file
    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .context("Failed to create temp file")?;

    file.write_all(&bytes)
        .await
        .context("Failed to write to temp file")?;

    file.flush().await.context("Failed to flush file")?;

    debug!("Downloaded {} bytes to {:?}", bytes.len(), temp_path);

    // Validate it's an STL file (basic check: starts with "solid" or binary header)
    let first_bytes = &bytes[..std::cmp::min(80, bytes.len())];
    let is_ascii_stl = first_bytes.starts_with(b"solid");
    let is_binary_stl = first_bytes.len() >= 80; // Binary STL has 80-byte header

    if !is_ascii_stl && !is_binary_stl {
        tokio::fs::remove_file(&temp_path).await.ok();
        bail!("Downloaded file does not appear to be a valid STL");
    }

    Ok(temp_path)
}
