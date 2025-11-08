use anyhow::{bail, Result};
use axum::http::HeaderMap;

/// Validate internal service token from X-Internal-Token header
/// This protects service-to-service endpoints from unauthorized access
pub fn require_internal_token(headers: &HeaderMap, expected_secret: &str) -> Result<()> {
    let token = headers
        .get("x-internal-token")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| anyhow::anyhow!("missing X-Internal-Token header"))?;

    if token != expected_secret {
        bail!("invalid internal token");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_valid_token_accepted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-internal-token", "secret123".parse().unwrap());

        assert!(require_internal_token(&headers, "secret123").is_ok());
    }

    #[test]
    fn test_invalid_token_rejected() {
        let mut headers = HeaderMap::new();
        headers.insert("x-internal-token", "wrong".parse().unwrap());

        assert!(require_internal_token(&headers, "secret123").is_err());
    }

    #[test]
    fn test_missing_token_rejected() {
        let headers = HeaderMap::new();
        assert!(require_internal_token(&headers, "secret123").is_err());
    }
}
