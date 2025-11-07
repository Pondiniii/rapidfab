use axum::{extract::Extension, middleware, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::session::require_auth;

use super::dto::*;

/// User database model for query results
struct UserFromDb {
    id: Uuid,
    email: String,
    #[allow(dead_code)]
    password_hash: String,
    full_name: Option<String>,
    created_at: DateTime<Utc>,
}

/// Create users router with all user endpoints
/// All routes require authentication
pub fn router() -> Router {
    Router::new()
        .route("/me", get(get_me))
        .layer(middleware::from_fn(require_auth))
}

/// GET /users/me
/// Get current authenticated user's profile
async fn get_me(
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    let user = sqlx::query_as!(
        UserFromDb,
        r#"
        SELECT id, email, password_hash, full_name, created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&*pool)
    .await?;

    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        full_name: user.full_name,
        created_at: user.created_at,
    }))
}
