use serde::{Deserialize, Serialize};
use uuid::Uuid;

// POST /internal/upload/init
#[derive(Debug, Deserialize)]
pub struct InitUploadRequest {
    pub files: Vec<FileMetadata>,
}

#[derive(Debug, Deserialize)]
pub struct FileMetadata {
    pub filename: String,
    pub size_bytes: u64,
    pub content_type: String,
}

#[derive(Debug, Serialize)]
pub struct InitUploadResponse {
    pub upload_id: Uuid,
    pub status: String,
}

// POST /internal/upload/{id}/signed-urls
#[derive(Debug, Serialize)]
pub struct SignedUrlsResponse {
    pub urls: Vec<FileUploadUrl>,
}

#[derive(Debug, Serialize)]
pub struct FileUploadUrl {
    pub file_id: Uuid,
    pub filename: String,
    pub upload_url: String,
    pub expires_at: String,
}

// POST /internal/upload/{id}/confirm
#[derive(Debug, Serialize)]
pub struct ConfirmUploadResponse {
    pub status: String,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub file_id: Uuid,
    pub filename: String,
    pub size_bytes: u64,
    pub s3_key: String,
}

// GET /internal/upload/file/{id}/read-url
#[derive(Debug, Serialize)]
pub struct ReadUrlResponse {
    pub url: String,
    pub expires_at: String,
}

// POST /internal/upload/transfer
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub session_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize)]
pub struct TransferResponse {
    pub transferred_count: usize,
    pub files_moved: usize,
}

// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}
