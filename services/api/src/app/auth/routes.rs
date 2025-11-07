use axum::{extract::Extension, http::StatusCode, routing::post, Json, Router};
use sqlx::PgPool;
use std::sync::Arc;

use crate::error::AppError;

use super::dto::*;
use super::service;

/// Create auth router with all authentication endpoints
pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
}

/// POST /auth/register
/// Register a new user and return session token
async fn register(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = service::register(&pool, req).await?;
    Ok(Json(response))
}

/// POST /auth/login
/// Login user and return session token
async fn login(
    Extension(pool): Extension<Arc<PgPool>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = service::login(&pool, req).await?;
    Ok(Json(response))
}

/// POST /auth/logout
/// Logout user by invalidating their session token
async fn logout(
    Extension(pool): Extension<Arc<PgPool>>,
    headers: axum::http::HeaderMap,
) -> Result<StatusCode, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    service::logout(&pool, token).await?;
    Ok(StatusCode::NO_CONTENT)
}
