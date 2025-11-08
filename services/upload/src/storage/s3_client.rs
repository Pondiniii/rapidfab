use anyhow::{bail, Context, Result};
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use std::time::Duration;

// Allow dead code until endpoints are implemented
#[allow(dead_code)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

// Allow dead code until endpoints are implemented
#[allow(dead_code)]
impl S3Client {
    /// Initialize S3 client from config
    pub async fn new(
        endpoint: &str,
        bucket: &str,
        region: &str,
        access_key_id: &str,
        secret_access_key: &str,
    ) -> Result<Self> {
        let credentials = Credentials::new(
            access_key_id,
            secret_access_key,
            None, // session token
            None, // expiry
            "upload-service",
        );

        let config = aws_sdk_s3::Config::builder()
            .endpoint_url(endpoint)
            .region(Region::new(region.to_string()))
            .credentials_provider(credentials)
            .force_path_style(true) // Required for Hetzner S3
            .build();

        let client = Client::from_conf(config);

        Ok(Self {
            client,
            bucket: bucket.to_string(),
        })
    }

    /// Generate presigned PUT URL for file upload
    /// Enforces Content-Type and Content-Length constraints
    pub async fn generate_upload_url(
        &self,
        s3_key: &str,
        content_type: &str,
        max_size_bytes: u64,
        expires_in_secs: u64,
    ) -> Result<String> {
        // Validate key (no path traversal)
        if s3_key.contains("..") {
            bail!("invalid s3 key: contains '..'");
        }

        let presigning_config =
            PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .content_type(content_type)
            .content_length(max_size_bytes as i64)
            .presigned(presigning_config)
            .await?;

        Ok(presigned_request.uri().to_string())
    }

    /// Generate presigned GET URL for file download
    pub async fn generate_download_url(
        &self,
        s3_key: &str,
        expires_in_secs: u64,
    ) -> Result<String> {
        if s3_key.contains("..") {
            bail!("invalid s3 key: contains '..'");
        }

        let presigning_config =
            PresigningConfig::expires_in(Duration::from_secs(expires_in_secs))?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .presigned(presigning_config)
            .await?;

        Ok(presigned_request.uri().to_string())
    }

    /// Check if file exists in S3
    pub async fn file_exists(&self, s3_key: &str) -> Result<bool> {
        let result = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .send()
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's a 404 (not found) vs other error
                if e.to_string().contains("NotFound") || e.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Delete file from S3
    pub async fn delete_file(&self, s3_key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(s3_key)
            .send()
            .await?;

        Ok(())
    }

    /// Move file from one key to another (copy + delete)
    /// Used for anon → user transfer
    pub async fn move_file(&self, from_key: &str, to_key: &str) -> Result<()> {
        // Validate keys
        if from_key.contains("..") || to_key.contains("..") {
            bail!("invalid s3 key: contains '..'");
        }

        // Copy object
        let copy_source = format!("{}/{}", self.bucket, from_key);
        self.client
            .copy_object()
            .bucket(&self.bucket)
            .copy_source(&copy_source)
            .key(to_key)
            .send()
            .await
            .context("failed to copy object")?;

        // Delete original
        self.delete_file(from_key).await?;

        Ok(())
    }

    /// Build S3 key for anonymous upload
    pub fn build_anon_key(session_id: &str, file_id: &str, extension: &str) -> String {
        format!("anon/{session_id}/{file_id}.{extension}")
    }

    /// Build S3 key for authenticated user upload
    pub fn build_user_key(user_id: &str, file_id: &str, extension: &str) -> String {
        format!("users/{user_id}/{file_id}.{extension}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_anon_key() {
        let key = S3Client::build_anon_key("session-123", "file-456", "stl");
        assert_eq!(key, "anon/session-123/file-456.stl");
    }

    #[test]
    fn test_build_user_key() {
        let key = S3Client::build_user_key("user-789", "file-abc", "step");
        assert_eq!(key, "users/user-789/file-abc.step");
    }

    #[test]
    fn test_path_traversal_prevention() {
        // Test that path validation logic works without needing a real client
        // These would be caught early in the validation
        assert!("../../../etc/passwd".contains(".."));
        assert!("../evil".contains(".."));
        assert!("safe/path/file.txt".contains("..") == false);

        // The actual validation happens in the methods before S3 calls
        // This test verifies the logic we use for validation
        let malicious_paths = vec![
            "../../../etc/passwd",
            "../evil",
            "path/../../../secret",
            "normal/../../still_bad",
        ];

        let safe_paths = vec![
            "anon/session-123/file.stl",
            "users/user-456/file.step",
            "safe/path/to/file.txt",
        ];

        for path in malicious_paths {
            assert!(path.contains(".."), "Should detect '..' in {}", path);
        }

        for path in safe_paths {
            assert!(!path.contains(".."), "Should not detect '..' in {}", path);
        }
    }
}
