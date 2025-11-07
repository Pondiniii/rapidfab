use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

use super::dto::*;
use super::repository;

/// Register a new user
/// Hashes password with Argon2, creates user record, and returns session token
pub async fn register(pool: &PgPool, req: RegisterRequest) -> Result<AuthResponse, AppError> {
    // Check if user already exists
    if repository::find_user_by_email(pool, &req.email)
        .await?
        .is_some()
    {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash password using Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::Internal)?
        .to_string();

    // Create user
    let user =
        repository::create_user(pool, &req.email, &password_hash, req.full_name.as_deref()).await?;

    // Create session
    let token = Uuid::new_v4().to_string();
    repository::create_session(pool, user.id, &token).await?;

    Ok(AuthResponse {
        token,
        user_id: user.id.to_string(),
    })
}

/// Login user
/// Verifies password and creates new session token
pub async fn login(pool: &PgPool, req: LoginRequest) -> Result<AuthResponse, AppError> {
    // Find user by email
    let user = repository::find_user_by_email(pool, &req.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| AppError::Internal)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    // Create new session
    let token = Uuid::new_v4().to_string();
    repository::create_session(pool, user.id, &token).await?;

    Ok(AuthResponse {
        token,
        user_id: user.id.to_string(),
    })
}

/// Logout user by deleting their session
pub async fn logout(pool: &PgPool, token: &str) -> Result<(), AppError> {
    repository::delete_session(pool, token).await
}
