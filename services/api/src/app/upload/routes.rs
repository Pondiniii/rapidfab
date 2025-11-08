use axum::{
    extract::{Extension, Path, State},
    http::HeaderMap,
    routing::post,
    Json, Router,
};
use reqwest::StatusCode as ReqwestStatus;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::{app::SessionId, config::Config, error::AppError};

use super::generate_anon_ticket;

#[derive(Debug, Deserialize)]
struct UploadFile {
    filename: String,
    #[serde(default)]
    content_type: Option<String>,
    size_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct UploadInitPayload {
    files: Vec<UploadFile>,
}

pub fn router() -> Router<Arc<Config>> {
    Router::new()
        .route("/upload/init", post(init_upload))
        .route("/upload/:id/urls", post(get_signed_urls))
        .route("/upload/:id/confirm", post(confirm_upload))
}

async fn init_upload(
    Extension(session): Extension<SessionId>,
    State(config): State<Arc<Config>>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let payload: UploadInitPayload = serde_json::from_value(body.clone())
        .map_err(|_| AppError::Validation("Invalid upload payload".into()))?;

    if payload.files.is_empty() {
        return Err(AppError::Validation(
            "Payload must contain at least one file".into(),
        ));
    }

    for file in &payload.files {
        if file.filename.trim().is_empty() {
            return Err(AppError::Validation(
                "Each file must include a non-empty filename".into(),
            ));
        }
        if let Some(content_type) = &file.content_type {
            if content_type.trim().is_empty() {
                return Err(AppError::Validation(
                    "content_type cannot be empty when provided".into(),
                ));
            }
        }
    }

    let max_size = payload
        .files
        .iter()
        .map(|f| f.size_bytes)
        .max()
        .unwrap_or(0);

    if max_size == 0 {
        return Err(AppError::Validation("size_bytes must be > 0".into()));
    }

    let ticket = generate_anon_ticket(
        session.0.clone(),
        "multi".into(),
        max_size,
        &config.upload_ticket_secret,
    )
    .map_err(|err| {
        error!(error = %err, "failed to generate upload ticket");
        AppError::Internal
    })?;

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/upload/init",
            config.upload_service_url
        ))
        .header("X-Upload-Ticket", ticket)
        .header("X-Internal-Token", &config.internal_service_token)
        .header("X-Session-Id", &session.0)
        .header("X-Forwarded-For", get_client_ip(&headers))
        .json(&body)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|err| {
            error!(error = %err, "upload init proxy request failed");
            AppError::Internal
        })?;

    convert_response(response).await.map(Json)
}

async fn get_signed_urls(
    Extension(session): Extension<SessionId>,
    State(config): State<Arc<Config>>,
    Path(upload_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/upload/{}/signed-urls",
            config.upload_service_url, upload_id
        ))
        .header("X-Internal-Token", &config.internal_service_token)
        .header("X-Session-Id", &session.0)
        .header("X-Forwarded-For", get_client_ip(&headers))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|err| {
            error!(error = %err, upload_id = %upload_id, "signed urls proxy request failed");
            AppError::Internal
        })?;

    convert_response(response).await.map(Json)
}

async fn confirm_upload(
    Extension(session): Extension<SessionId>,
    State(config): State<Arc<Config>>,
    Path(upload_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/internal/upload/{}/confirm",
            config.upload_service_url, upload_id
        ))
        .header("X-Internal-Token", &config.internal_service_token)
        .header("X-Session-Id", &session.0)
        .header("X-Forwarded-For", get_client_ip(&headers))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|err| {
            error!(
                error = %err,
                upload_id = %upload_id,
                "confirm upload proxy request failed"
            );
            AppError::Internal
        })?;

    convert_response(response).await.map(Json)
}

fn get_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|value| {
            value
                .split(',')
                .map(str::trim)
                .find(|entry| !entry.is_empty())
        })
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .unwrap_or("unknown")
        .to_string()
}

async fn convert_response(response: reqwest::Response) -> Result<Value, AppError> {
    match response.status() {
        status if status == ReqwestStatus::FORBIDDEN => Err(AppError::Forbidden),
        status if status == ReqwestStatus::NOT_FOUND => Err(AppError::NotFound),
        status if !status.is_success() => {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<failed to read body>".to_string());
            error!(%status, body = %body, "upload proxy returned error");
            Err(AppError::Internal)
        }
        _ => response.json::<Value>().await.map_err(|err| {
            error!(error = %err, "failed to parse upload proxy response");
            AppError::Internal
        }),
    }
}
