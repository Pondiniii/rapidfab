pub mod routes;

use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct UploadTicket {
    session_id: Option<String>,
    user_id: Option<String>,
    file_name: String,
    max_size_bytes: u64,
    expires_at: String,
    #[serde(rename = "exp")]
    expires_at_epoch: i64,
    iat: i64,
}

/// Generate upload ticket for anonymous user
pub fn generate_anon_ticket(
    session_id: String,
    file_name: String,
    max_size_bytes: u64,
    secret: &str,
) -> Result<String> {
    let expires_at = Utc::now() + Duration::minutes(2);
    let expires_at_str = expires_at.to_rfc3339();

    let ticket = UploadTicket {
        session_id: Some(session_id),
        user_id: None,
        file_name,
        max_size_bytes,
        expires_at: expires_at_str,
        expires_at_epoch: expires_at.timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &ticket,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Generate upload ticket for authenticated user
pub fn generate_user_ticket(
    user_id: Uuid,
    file_name: String,
    max_size_bytes: u64,
    secret: &str,
) -> Result<String> {
    let expires_at = Utc::now() + Duration::minutes(2);
    let expires_at_str = expires_at.to_rfc3339();

    let ticket = UploadTicket {
        session_id: None,
        user_id: Some(user_id.to_string()),
        file_name,
        max_size_bytes,
        expires_at: expires_at_str,
        expires_at_epoch: expires_at.timestamp(),
        iat: Utc::now().timestamp(),
    };

    let token = encode(
        &Header::default(),
        &ticket,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}
