//! gRPC Authentication Interceptor
//!
//! This module provides JWT token verification for gRPC requests.
//! The actual TokenVerifier implementation should be injected from secreton-api at runtime.

use tonic::{Request, Status};
use tracing::{debug, warn};

/// Extract bearer token from Authorization header value
pub fn extract_bearer_token(auth_header: &str) -> Option<String> {
    auth_header.strip_prefix("Bearer ").map(|s| s.to_string())
}

/// Simple authentication interceptor function
///
/// This function can be used as a tonic interceptor to validate authorization headers.
///
/// Note: For full TokenVerifier integration, configure this at the application level
/// by passing the verifier to the server builder.
pub fn auth_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let auth_header = req.metadata().get("authorization");

    if auth_header.is_none() {
        // For now, allow unauthenticated requests
        // In production, configure require_auth based on settings
        debug!("No authorization header present");
        return Ok(req);
    }

    let auth_str = auth_header
        .unwrap()
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid authorization header"))?;

    if !auth_str.starts_with("Bearer ") {
        warn!("Invalid authorization header format");
        return Err(Status::unauthenticated(
            "Invalid authorization header format",
        ));
    }

    // Token present and properly formatted
    // Actual validation happens in the service layer with TokenVerifier
    debug!("Authorization header present");
    Ok(req)
}

/// Configuration for authentication interceptor
#[derive(Debug, Clone, Default)]
/// Mewakili pub `AuthConfig`.
pub struct AuthConfig {
    /// Whether authentication is required
    pub require_auth: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        let token = extract_bearer_token("Bearer abc123");
        assert_eq!(token, Some("abc123".to_string()));

        let no_token = extract_bearer_token("Basic abc123");
        assert_eq!(no_token, None);
    }

    #[test]
    fn test_auth_interceptor_with_token() {
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert("authorization", "Bearer test-token".parse().unwrap());

        let result = auth_interceptor(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auth_interceptor_without_token() {
        let request = Request::new(());
        let result = auth_interceptor(request);
        // Should pass (permissive by default)
        assert!(result.is_ok());
    }
}
