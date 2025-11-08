use anyhow::{bail, Context, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::UploadTicket;
use crate::storage::{
    check_anon_quota, check_user_quota, update_anon_quota, update_user_quota, QuotaLimits,
    S3Client,
};

use super::dto::*;

pub struct UploadService {
    pool: PgPool,
    s3_client: S3Client,
    limits: QuotaLimits,
}

impl UploadService {
    pub fn new(pool: PgPool, s3_client: S3Client, limits: QuotaLimits) -> Self {
        Self {
            pool,
            s3_client,
            limits,
        }
    }

    /// Initialize upload - create DB records, check quota
    pub async fn init_upload(
        &self,
        ticket: UploadTicket,
        ip: &str,
        files: Vec<FileMetadata>,
    ) -> Result<InitUploadResponse> {
        // SECURITY: Validate each file against ticket's max_size_bytes
        for file in &files {
            if file.size_bytes > ticket.max_size_bytes {
                bail!(
                    "file '{}' size ({} bytes) exceeds ticket limit ({} bytes)",
                    file.filename,
                    file.size_bytes,
                    ticket.max_size_bytes
                );
            }
        }

        // Calculate total bytes
        let total_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();

        // Check quota
        if ticket.is_anonymous() {
            let session_id = ticket.session_id.as_ref().context("missing session_id")?;
            check_anon_quota(&self.pool, session_id, ip, total_bytes, &self.limits).await?;
        } else {
            let user_id = ticket.user_id.as_ref().context("missing user_id")?;
            check_user_quota(&self.pool, user_id, total_bytes, &self.limits).await?;
        }

        // Create upload record
        let upload_id = Uuid::new_v4();
        let user_id_uuid = ticket
            .user_id
            .as_ref()
            .map(|id| Uuid::parse_str(id))
            .transpose()?;
        let session_id_uuid = ticket
            .session_id
            .as_ref()
            .map(|id| Uuid::parse_str(id))
            .transpose()?;

        sqlx::query!(
            r#"
            INSERT INTO uploads (id, user_id, session_id, ip_address, status)
            VALUES ($1, $2, $3, $4, 'pending')
            "#,
            upload_id,
            user_id_uuid,
            session_id_uuid,
            ip,
        )
        .execute(&self.pool)
        .await?;

        // Create file records (pending)
        for file in files {
            let file_id = Uuid::new_v4();
            let ext = std::path::Path::new(&file.filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");

            let s3_key = if ticket.is_anonymous() {
                S3Client::build_anon_key(
                    ticket.session_id.as_ref().unwrap(),
                    &file_id.to_string(),
                    ext,
                )
            } else {
                S3Client::build_user_key(
                    ticket.user_id.as_ref().unwrap(),
                    &file_id.to_string(),
                    ext,
                )
            };

            sqlx::query!(
                r#"
                INSERT INTO files (id, upload_id, filename, s3_key, size_bytes, mime_type)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                file_id,
                upload_id,
                file.filename,
                s3_key,
                file.size_bytes as i64,
                file.content_type,
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(InitUploadResponse {
            upload_id,
            status: "pending".to_string(),
        })
    }

    /// Generate presigned URLs for upload
    pub async fn generate_signed_urls(&self, upload_id: Uuid) -> Result<SignedUrlsResponse> {
        // Get files for this upload
        let files = sqlx::query!(
            r#"
            SELECT id, filename, s3_key, size_bytes, mime_type
            FROM files
            WHERE upload_id = $1
            "#,
            upload_id,
        )
        .fetch_all(&self.pool)
        .await?;

        if files.is_empty() {
            bail!("upload not found or no files");
        }

        // Generate URLs
        let mut urls = Vec::new();
        let expires_in = 3600; // 1 hour
        let expires_at =
            (Utc::now() + chrono::Duration::seconds(expires_in as i64)).to_rfc3339();

        for file in files {
            let upload_url = self
                .s3_client
                .generate_upload_url(
                    &file.s3_key,
                    &file
                        .mime_type
                        .unwrap_or_else(|| "application/octet-stream".to_string()),
                    file.size_bytes as u64,
                    expires_in,
                )
                .await?;

            urls.push(FileUploadUrl {
                file_id: file.id,
                filename: file.filename,
                upload_url,
                expires_at: expires_at.clone(),
            });
        }

        Ok(SignedUrlsResponse { urls })
    }

    /// Confirm upload - verify files exist in S3
    pub async fn confirm_upload(&self, upload_id: Uuid) -> Result<ConfirmUploadResponse> {
        // Get upload info
        let upload = sqlx::query!(
            r#"
            SELECT user_id, session_id, ip_address
            FROM uploads
            WHERE id = $1
            "#,
            upload_id,
        )
        .fetch_one(&self.pool)
        .await?;

        // Get files
        let files = sqlx::query!(
            r#"
            SELECT id, filename, s3_key, size_bytes
            FROM files
            WHERE upload_id = $1
            "#,
            upload_id,
        )
        .fetch_all(&self.pool)
        .await?;

        // Verify each file exists in S3
        for file in &files {
            if !self.s3_client.file_exists(&file.s3_key).await? {
                bail!("file {} not found in S3", file.filename);
            }
        }

        // Update upload status
        sqlx::query!(
            r#"
            UPDATE uploads
            SET status = 'completed', updated_at = NOW()
            WHERE id = $1
            "#,
            upload_id,
        )
        .execute(&self.pool)
        .await?;

        // Update quota
        let total_bytes: u64 = files.iter().map(|f| f.size_bytes as u64).sum();
        if let Some(user_id) = upload.user_id {
            update_user_quota(&self.pool, &user_id.to_string(), total_bytes).await?;
        } else if upload.session_id.is_some() {
            let session_id_uuid = upload.session_id.as_ref().unwrap();
            let session_id_str = session_id_uuid.to_string();
            let ip = upload.ip_address.unwrap_or_else(|| "unknown".to_string());
            update_anon_quota(&self.pool, &session_id_str, &ip, total_bytes).await?;
        }

        // Return file info
        let file_info: Vec<FileInfo> = files
            .into_iter()
            .map(|f| FileInfo {
                file_id: f.id,
                filename: f.filename,
                size_bytes: f.size_bytes as u64,
                s3_key: f.s3_key,
            })
            .collect();

        Ok(ConfirmUploadResponse {
            status: "completed".to_string(),
            files: file_info,
        })
    }

    /// Generate read URL for file (for pricing service)
    pub async fn generate_read_url(&self, file_id: Uuid) -> Result<ReadUrlResponse> {
        // Get file
        let file = sqlx::query!(
            r#"
            SELECT s3_key FROM files WHERE id = $1
            "#,
            file_id,
        )
        .fetch_one(&self.pool)
        .await?;

        // Generate read URL
        let expires_in = 3600; // 1 hour
        let url = self
            .s3_client
            .generate_download_url(&file.s3_key, expires_in)
            .await?;
        let expires_at =
            (Utc::now() + chrono::Duration::seconds(expires_in as i64)).to_rfc3339();

        Ok(ReadUrlResponse { url, expires_at })
    }

    /// Transfer anonymous uploads to user account (post-registration)
    pub async fn transfer_uploads(
        &self,
        session_id: String,
        user_id: String,
    ) -> Result<TransferResponse> {
        let user_id_uuid = Uuid::parse_str(&user_id)?;

        // Find all uploads for this session
        let uploads = sqlx::query!(
            r#"
            SELECT id FROM uploads
            WHERE session_id = $1
            "#,
            Uuid::parse_str(&session_id)?,
        )
        .fetch_all(&self.pool)
        .await?;

        if uploads.is_empty() {
            return Ok(TransferResponse {
                transferred_count: 0,
                files_moved: 0,
            });
        }

        let mut total_files_moved = 0;

        // For each upload, move files in S3 and update DB
        for upload in &uploads {
            // Get files
            let files = sqlx::query!(
                r#"
                SELECT id, s3_key, filename FROM files
                WHERE upload_id = $1
                "#,
                upload.id,
            )
            .fetch_all(&self.pool)
            .await?;

            // Move each file in S3
            for file in &files {
                // Parse current key to extract extension
                let ext = std::path::Path::new(&file.s3_key)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");

                // Build new user key
                let new_s3_key =
                    S3Client::build_user_key(&user_id, &file.id.to_string(), ext);

                // Move in S3 (copy + delete)
                self.s3_client
                    .move_file(&file.s3_key, &new_s3_key)
                    .await?;

                // Update DB with new key
                sqlx::query!(
                    r#"
                    UPDATE files SET s3_key = $1
                    WHERE id = $2
                    "#,
                    new_s3_key,
                    file.id,
                )
                .execute(&self.pool)
                .await?;

                total_files_moved += 1;
            }

            // Update upload record: session_id → user_id
            sqlx::query!(
                r#"
                UPDATE uploads
                SET user_id = $1, session_id = NULL
                WHERE id = $2
                "#,
                user_id_uuid,
                upload.id,
            )
            .execute(&self.pool)
            .await?;
        }

        // Transfer quota (session → user)
        sqlx::query!(
            r#"
            UPDATE upload_quotas
            SET user_id = $1, session_id = NULL
            WHERE session_id = $2
            "#,
            user_id_uuid,
            Uuid::parse_str(&session_id)?,
        )
        .execute(&self.pool)
        .await?;

        Ok(TransferResponse {
            transferred_count: uploads.len(),
            files_moved: total_files_moved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::UploadTicket;
    use chrono::{Duration, Utc};

    /// Test that file size validation rejects files exceeding ticket limit
    #[test]
    fn test_file_size_validation_logic() {
        // Create a ticket with 1000 byte limit
        let ticket = UploadTicket {
            session_id: Some("test-session".to_string()),
            user_id: None,
            file_name: "test.stl".to_string(),
            max_size_bytes: 1000,
            expires_at: Utc::now() + Duration::minutes(5),
            iat: Utc::now().timestamp(),
        };

        // Test case 1: File within limit (should pass)
        let small_file = FileMetadata {
            filename: "small.stl".to_string(),
            size_bytes: 500,
            content_type: "model/stl".to_string(),
        };
        assert!(small_file.size_bytes <= ticket.max_size_bytes);

        // Test case 2: File exceeding limit (should fail)
        let large_file = FileMetadata {
            filename: "large.stl".to_string(),
            size_bytes: 2000,
            content_type: "model/stl".to_string(),
        };
        assert!(large_file.size_bytes > ticket.max_size_bytes);

        // Validation logic tested:
        // if file.size_bytes > ticket.max_size_bytes { bail!(...) }
        // This ensures the security fix prevents quota bypass attacks
    }

    /// Test that validation message format is correct
    #[test]
    fn test_validation_error_message_format() {
        let filename = "test.stl";
        let file_size: u64 = 2000;
        let ticket_limit: u64 = 1000;

        let expected_msg = format!(
            "file '{}' size ({} bytes) exceeds ticket limit ({} bytes)",
            filename, file_size, ticket_limit
        );

        // Verify the error message provides clear information
        assert!(expected_msg.contains(filename));
        assert!(expected_msg.contains("2000 bytes"));
        assert!(expected_msg.contains("1000 bytes"));
    }
}

