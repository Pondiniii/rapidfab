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
}
