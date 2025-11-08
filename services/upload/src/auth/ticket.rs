use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::{Result, bail};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadTicket {
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub file_name: String,
    pub max_size_bytes: u64,
    pub expires_at: DateTime<Utc>,
    pub iat: i64, // issued at
}

impl UploadTicket {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_anonymous(&self) -> bool {
        self.user_id.is_none() && self.session_id.is_some()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some() && self.session_id.is_none()
    }

    pub fn validate(&self) -> Result<()> {
        // Ensure exactly one of session_id OR user_id
        if self.session_id.is_some() && self.user_id.is_some() {
            bail!("ticket cannot have both session_id and user_id");
        }
        if self.session_id.is_none() && self.user_id.is_none() {
            bail!("ticket must have either session_id or user_id");
        }

        // Check expiration
        if self.is_expired() {
            bail!("ticket expired at {}", self.expires_at);
        }

        // Validate max_size
        if self.max_size_bytes == 0 {
            bail!("max_size_bytes must be > 0");
        }

        Ok(())
    }
}

/// Validate upload ticket from X-Upload-Ticket header
/// Returns the ticket if valid, error otherwise
pub fn validate_ticket(token: &str, secret: &str) -> Result<UploadTicket> {
    // TODO: Implement JWT validation with EdDSA signature
    // For now, use basic HMAC (can upgrade to EdDSA later)

    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false; // We check manually via ticket.is_expired()

    let token_data = decode::<UploadTicket>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    let ticket = token_data.claims;
    ticket.validate()?;

    Ok(ticket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_ticket_validation_anonymous() {
        let ticket = UploadTicket {
            session_id: Some("test-session".to_string()),
            user_id: None,
            file_name: "test.stl".to_string(),
            max_size_bytes: 1024 * 1024, // 1MB
            expires_at: Utc::now() + Duration::minutes(2),
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_ok());
        assert!(ticket.is_anonymous());
        assert!(!ticket.is_authenticated());
    }

    #[test]
    fn test_ticket_validation_authenticated() {
        let ticket = UploadTicket {
            session_id: None,
            user_id: Some("user-123".to_string()),
            file_name: "test.stl".to_string(),
            max_size_bytes: 1024 * 1024,
            expires_at: Utc::now() + Duration::minutes(2),
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_ok());
        assert!(!ticket.is_anonymous());
        assert!(ticket.is_authenticated());
    }

    #[test]
    fn test_ticket_expired() {
        let ticket = UploadTicket {
            session_id: Some("test".to_string()),
            user_id: None,
            file_name: "test.stl".to_string(),
            max_size_bytes: 1024,
            expires_at: Utc::now() - Duration::minutes(1), // Expired!
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_ticket_invalid_both_ids() {
        let ticket = UploadTicket {
            session_id: Some("session".to_string()),
            user_id: Some("user".to_string()),
            file_name: "test.stl".to_string(),
            max_size_bytes: 1024,
            expires_at: Utc::now() + Duration::minutes(2),
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_ticket_invalid_no_ids() {
        let ticket = UploadTicket {
            session_id: None,
            user_id: None,
            file_name: "test.stl".to_string(),
            max_size_bytes: 1024,
            expires_at: Utc::now() + Duration::minutes(2),
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_err());
    }

    #[test]
    fn test_ticket_invalid_zero_size() {
        let ticket = UploadTicket {
            session_id: Some("test".to_string()),
            user_id: None,
            file_name: "test.stl".to_string(),
            max_size_bytes: 0,
            expires_at: Utc::now() + Duration::minutes(2),
            iat: Utc::now().timestamp(),
        };

        assert!(ticket.validate().is_err());
    }
}
