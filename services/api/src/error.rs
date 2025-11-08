use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application error types
/// Maps domain errors to HTTP responses with appropriate status codes
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Forbidden")]
    Forbidden,

    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden"),
            AppError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}
