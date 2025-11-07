use chrono::{DateTime, Utc};
use serde::Serialize;

/// User profile response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub created_at: DateTime<Utc>,
}
