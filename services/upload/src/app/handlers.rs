use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use super::dto::*;
use crate::auth::{require_internal_token, validate_ticket};

// AppState defined in main.rs - re-export for handlers
pub use crate::AppState;

// POST /internal/upload/init
pub async fn init_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<InitUploadRequest>,
) -> Result<Json<InitUploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate ticket
    let token = headers
        .get("x-upload-ticket")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| error_response(StatusCode::UNAUTHORIZED, "missing upload ticket"))?;

    let ticket = validate_ticket(token, &state.ticket_secret)
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, &format!("invalid ticket: {e}")))?;

    // Get IP
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Init upload
    let response = state
        .upload_service
        .init_upload(ticket, &ip, req.files)
        .await
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    Ok(Json(response))
}

// POST /internal/upload/{id}/signed-urls
pub async fn generate_signed_urls(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(upload_id): Path<Uuid>,
) -> Result<Json<SignedUrlsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate internal service token
    require_internal_token(&headers, &state.internal_token)
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, &e.to_string()))?;

    let response = state
        .upload_service
        .generate_signed_urls(upload_id)
        .await
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    Ok(Json(response))
}

// POST /internal/upload/{id}/confirm
pub async fn confirm_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(upload_id): Path<Uuid>,
) -> Result<Json<ConfirmUploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate internal service token
    require_internal_token(&headers, &state.internal_token)
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, &e.to_string()))?;

    let response = state
        .upload_service
        .confirm_upload(upload_id)
        .await
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    Ok(Json(response))
}

// GET /internal/upload/file/{id}/read-url
pub async fn generate_read_url(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<Uuid>,
) -> Result<Json<ReadUrlResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate internal service token
    require_internal_token(&headers, &state.internal_token)
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, &e.to_string()))?;

    let response = state
        .upload_service
        .generate_read_url(file_id)
        .await
        .map_err(|e| error_response(StatusCode::NOT_FOUND, &e.to_string()))?;

    Ok(Json(response))
}

// POST /internal/upload/transfer
pub async fn transfer_uploads(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TransferRequest>,
) -> Result<Json<TransferResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate internal service token
    require_internal_token(&headers, &state.internal_token)
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, &e.to_string()))?;

    let response = state
        .upload_service
        .transfer_uploads(req.session_id, req.user_id)
        .await
        .map_err(|e| error_response(StatusCode::BAD_REQUEST, &e.to_string()))?;

    Ok(Json(response))
}

// Helper
fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: status.to_string(),
            message: message.to_string(),
        }),
    )
}
