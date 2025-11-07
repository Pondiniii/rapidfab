use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// User database model
#[derive(Debug)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Session database model
#[derive(Debug)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

/// Create a new user in the database
pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
    full_name: Option<&str>,
) -> Result<User, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, full_name)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, full_name, created_at
        "#,
        email,
        password_hash,
        full_name
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Find user by email address
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash, full_name, created_at
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(user)
}

/// Create a new session for a user
/// Session expires after 30 days
pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<Session, AppError> {
    let expires_at = Utc::now() + Duration::days(30);

    let session = sqlx::query_as!(
        Session,
        r#"
        INSERT INTO sessions (user_id, token, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, token, expires_at
        "#,
        user_id,
        token,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(session)
}

/// Delete a session by token (logout)
pub async fn delete_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    sqlx::query!(
        r#"
        DELETE FROM sessions WHERE token = $1
        "#,
        token
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Find a valid session by token
/// Returns None if session doesn't exist or has expired
pub async fn find_session_by_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<Session>, AppError> {
    let session = sqlx::query_as!(
        Session,
        r#"
        SELECT id, user_id, token, expires_at
        FROM sessions
        WHERE token = $1 AND expires_at > NOW()
        "#,
        token
    )
    .fetch_optional(pool)
    .await?;

    Ok(session)
}
