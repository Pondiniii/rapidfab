use axum::{extract::Request, middleware::Next, response::Response, Extension};
use sqlx::PgPool;
use std::sync::Arc;

use crate::app::auth::repository;
use crate::error::AppError;

/// Middleware that requires valid authentication
/// Extracts Bearer token from Authorization header, validates session,
/// and adds user_id to request extensions
pub async fn require_auth(
    Extension(pool): Extension<Arc<PgPool>>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract token from Authorization header
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    // Validate session token
    let session = repository::find_session_by_token(&pool, token)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Add user_id to request extensions for downstream handlers
    req.extensions_mut().insert(session.user_id);

    Ok(next.run(req).await)
}
