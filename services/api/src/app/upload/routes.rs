use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;

/// Upload service base URL (injected via Extension)
pub struct UploadServiceUrl(pub String);

/// Create upload router with proxy endpoints
pub fn router() -> Router {
    Router::new()
        .route("/upload/init", post(init_upload))
        .route("/upload/:id/urls", post(get_signed_urls))
        .route("/upload/:id/confirm", post(confirm_upload))
}

/// POST /files/upload/init
/// Proxy to upload service init endpoint
async fn init_upload(
    Extension(upload_url): Extension<Arc<UploadServiceUrl>>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/internal/upload/init", upload_url.0))
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    if !response.status().is_success() {
        return Err(AppError::Internal);
    }

    let body = response
        .json::<Value>()
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(Json(body))
}

/// POST /files/upload/:id/urls
/// Proxy to upload service signed URLs endpoint
async fn get_signed_urls(
    Extension(upload_url): Extension<Arc<UploadServiceUrl>>,
    Path(upload_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/upload/{}/signed-urls",
            upload_url.0, upload_id
        ))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    if !response.status().is_success() {
        return Err(AppError::Internal);
    }

    let body = response
        .json::<Value>()
        .await
        .map_err(|_| AppError::Internal)?;

    Ok(Json(body))
}

/// POST /files/upload/:id/confirm
/// Proxy to upload service confirm endpoint
async fn confirm_upload(
    Extension(upload_url): Extension<Arc<UploadServiceUrl>>,
    Path(upload_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/upload/{}/confirm",
            upload_url.0, upload_id
        ))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| AppError::Internal)?;

    if !response.status().is_success() {
        return Err(AppError::Internal);
    }

    Ok(StatusCode::OK)
}
