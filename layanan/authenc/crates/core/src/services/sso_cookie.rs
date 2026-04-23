use authenc_types::config::SsoCookieConfig;
use authenc_types::{AuthencError, Result};
use axum::http::{HeaderMap, HeaderValue, header};

pub use lib_core::auth::SsoSession;

/// SSO Cookie Manager for secure cookie operations
pub struct SsoCookieManager {
    config: SsoCookieConfig,
}

impl SsoCookieManager {
    /// Create a new SSO cookie manager
    pub fn new(config: SsoCookieConfig) -> Self {
        Self { config }
    }

    /// Generate a Set-Cookie header value for SSO session
    pub fn create_cookie(&self, session: &SsoSession) -> Result<String> {
        let session_json = session
            .to_json()
            .map_err(|e| AuthencError::internal(e.to_string()))?;

        // Base64 encode the session data for cookie storage
        let encoded_session = lib_core::encoding::base64_encode(&session_json);

        // Build cookie string with all security attributes
        let mut cookie_parts = vec![
            format!("{}={}", self.config.name, encoded_session),
            format!("Path={}", self.config.path),
            format!("Max-Age={}", self.config.max_age),
        ];

        // Add domain if configured
        if let Some(ref domain) = self.config.domain {
            cookie_parts.push(format!("Domain={}", domain));
        }

        // Add security flags
        if self.config.secure {
            cookie_parts.push("Secure".to_string());
        }

        if self.config.http_only {
            cookie_parts.push("HttpOnly".to_string());
        }

        // Add SameSite attribute
        cookie_parts.push(format!("SameSite={}", self.config.same_site));

        Ok(cookie_parts.join("; "))
    }

    /// Create a Set-Cookie header for deletion (logout)
    pub fn delete_cookie(&self) -> String {
        let mut cookie_parts = vec![
            format!("{}=", self.config.name),
            format!("Path={}", self.config.path),
            "Max-Age=0".to_string(),
        ];

        if let Some(ref domain) = self.config.domain {
            cookie_parts.push(format!("Domain={}", domain));
        }

        if self.config.secure {
            cookie_parts.push("Secure".to_string());
        }

        if self.config.http_only {
            cookie_parts.push("HttpOnly".to_string());
        }

        cookie_parts.push(format!("SameSite={}", self.config.same_site));

        cookie_parts.join("; ")
    }

    /// Extract SSO session from request headers
    pub fn extract_session(&self, headers: &HeaderMap) -> Result<Option<SsoSession>> {
        // Get Cookie header
        let cookie_header = match headers.get(header::COOKIE) {
            Some(value) => value,
            None => return Ok(None),
        };

        // Parse cookie header
        let cookie_str = cookie_header
            .to_str()
            .map_err(|e| AuthencError::internal(format!("Invalid cookie header: {}", e)))?;

        // Find our SSO cookie
        for cookie in cookie_str.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix(&format!("{}=", self.config.name)) {
                // Decode base64
                let decoded = lib_core::encoding::base64_decode(value).map_err(|e| {
                    AuthencError::internal(format!("Failed to decode cookie: {}", e))
                })?;

                let session_json = String::from_utf8(decoded).map_err(|e| {
                    AuthencError::internal(format!("Invalid UTF-8 in cookie: {}", e))
                })?;

                // Deserialize session
                let session = SsoSession::from_json(&session_json)
                    .map_err(|e| AuthencError::internal(e.to_string()))?;

                // Validate session
                if session.is_valid() {
                    return Ok(Some(session));
                } else {
                    return Err(AuthencError::unauthorized("SSO session expired"));
                }
            }
        }

        Ok(None)
    }

    /// Add Set-Cookie header to response headers
    pub fn add_cookie_header(&self, headers: &mut HeaderMap, session: &SsoSession) -> Result<()> {
        let cookie_value = self.create_cookie(session)?;
        let header_value = HeaderValue::from_str(&cookie_value)
            .map_err(|e| AuthencError::internal(format!("Invalid cookie header value: {}", e)))?;

        headers.insert(header::SET_COOKIE, header_value);
        Ok(())
    }

    /// Add delete cookie header to response headers
    pub fn add_delete_cookie_header(&self, headers: &mut HeaderMap) -> Result<()> {
        let cookie_value = self.delete_cookie();
        let header_value = HeaderValue::from_str(&cookie_value)
            .map_err(|e| AuthencError::internal(format!("Invalid cookie header value: {}", e)))?;

        headers.insert(header::SET_COOKIE, header_value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SsoCookieConfig {
        SsoCookieConfig {
            name: "AUTHENC_SSO".to_string(),
            domain: Some("simpel.kejaksaan.go.id".to_string()),
            path: "/".to_string(),
            max_age: 3600,
            secure: true,
            http_only: true,
            same_site: "Lax".to_string(),
        }
    }

    fn create_test_session() -> SsoSession {
        SsoSession::new(
            "user123".to_string(),
            "testuser".to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string(), "admin".to_string()],
            3600,
            Some("192.168.1.1".to_string()),
            Some("Mozilla/5.0".to_string()),
        )
    }

    #[test]
    fn test_sso_session_creation() {
        let session = create_test_session();

        assert_eq!(session.user_id, "user123");
        assert_eq!(session.username, "testuser");
        assert_eq!(session.email, Some("test@example.com".to_string()));
        assert_eq!(session.roles.len(), 2);
        assert!(!session.is_expired());
        assert!(session.is_valid());
    }

    #[test]
    fn test_sso_session_serialization() {
        let session = create_test_session();

        let json = session.to_json().unwrap();
        let deserialized = SsoSession::from_json(&json).unwrap();

        assert_eq!(session.user_id, deserialized.user_id);
        assert_eq!(session.username, deserialized.username);
        assert_eq!(session.email, deserialized.email);
    }

    #[test]
    fn test_cookie_creation() {
        let config = create_test_config();
        let manager = SsoCookieManager::new(config);
        let session = create_test_session();

        let cookie = manager.create_cookie(&session).unwrap();

        assert!(cookie.contains("AUTHENC_SSO="));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("Max-Age=3600"));
        assert!(cookie.contains("Domain=simpel.kejaksaan.go.id"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
    }

    #[test]
    fn test_cookie_deletion() {
        let config = create_test_config();
        let manager = SsoCookieManager::new(config);

        let cookie = manager.delete_cookie();

        assert!(cookie.contains("AUTHENC_SSO="));
        assert!(cookie.contains("Max-Age=0"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("HttpOnly"));
    }

    #[test]
    fn test_cookie_extraction() {
        let config = create_test_config();
        let manager = SsoCookieManager::new(config);
        let session = create_test_session();

        // Create cookie
        let cookie_value = manager.create_cookie(&session).unwrap();

        // Extract just the cookie value part
        let cookie_parts: Vec<&str> = cookie_value.split(';').collect();
        let cookie_name_value = cookie_parts[0].trim();

        // Create headers with cookie
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(cookie_name_value).unwrap(),
        );

        // Extract session
        let extracted = manager.extract_session(&headers).unwrap();

        assert!(extracted.is_some());
        let extracted_session = extracted.unwrap();
        assert_eq!(extracted_session.user_id, session.user_id);
    }

    #[test]
    fn test_cookie_extraction_no_cookie() {
        let config = create_test_config();
        let manager = SsoCookieManager::new(config);

        let headers = HeaderMap::new();
        let result = manager.extract_session(&headers).unwrap();

        assert!(result.is_none());
    }
}
