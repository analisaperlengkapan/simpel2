//! OAuth2/OIDC client implementation
//!
//! Handles OAuth2 authorization code flow for Authenc integration

use serde::{Deserialize, Serialize};

/// OAuth2 authorization request parameters
#[derive(Clone, Debug, Serialize)]
pub struct AuthorizationRequest {
    /// OAuth2 response type (always "code" for authorization code flow)
    pub response_type: String,
    /// Client ID
    pub client_id: String,
    /// Redirect URI where authorization code will be sent
    pub redirect_uri: String,
    /// OAuth2 scope (space-separated)
    pub scope: String,
    /// State parameter for CSRF protection
    pub state: String,
    /// Optional code challenge for PKCE
    pub code_challenge: Option<String>,
    /// Code challenge method (S256 or plain)
    pub code_challenge_method: Option<String>,
}

/// OAuth2 token exchange request
#[derive(Clone, Debug, Serialize)]
pub struct TokenExchangeRequest {
    /// Grant type (always "authorization_code")
    pub grant_type: String,
    /// Authorization code from callback
    pub code: String,
    /// Redirect URI (must match the one used in authorization request)
    pub redirect_uri: String,
    /// Client ID
    pub client_id: String,
    /// Optional client secret
    pub client_secret: Option<String>,
    /// Optional code verifier for PKCE
    pub code_verifier: Option<String>,
}

/// OAuth2 error response
#[derive(Clone, Debug, Deserialize)]
pub struct OAuthError {
    /// Error code
    pub error: String,
    /// Error description
    pub error_description: Option<String>,
}

/// OAuth2 client
pub struct OAuthClient {
    /// Authenc base URL
    pub base_url: String,
    /// Realm name
    pub realm: String,
    /// Client ID
    pub client_id: String,
    /// Redirect URI
    pub redirect_uri: String,
}

impl OAuthClient {
    /// Create new OAuth client
    pub fn new(base_url: String, realm: String, client_id: String, redirect_uri: String) -> Self {
        Self {
            base_url,
            realm,
            client_id,
            redirect_uri,
        }
    }

    /// Get authorization endpoint URL
    pub fn authorization_endpoint(&self) -> String {
        format!("{}/api/v1/oauth2/authorize", self.base_url)
    }

    /// Get token endpoint URL
    pub fn token_endpoint(&self) -> String {
        format!("{}/api/v1/oauth2/token", self.base_url)
    }

    /// Generate authorization URL
    pub fn get_authorization_url(&self, state: String, scope: Option<String>) -> String {
        let scope = scope.unwrap_or_else(|| "openid profile email".to_string());

        format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            self.authorization_endpoint(),
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scope),
            urlencoding::encode(&state)
        )
    }

    /// Generate random state for CSRF protection
    pub fn generate_state() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<crate::features::auth::TokenResponse, String> {
        #[cfg(target_arch = "wasm32")]
        {
            use gloo_net::http::Request;

            let form_data = format!(
                "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}",
                urlencoding::encode(code),
                urlencoding::encode(&self.redirect_uri),
                urlencoding::encode(&self.client_id)
            );

            let request = Request::post(&self.token_endpoint())
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(form_data)
                .map_err(|e| format!("Failed to build request: {}", e))?;

            let response = request
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if response.ok() {
                response
                    .json::<crate::features::auth::TokenResponse>()
                    .await
                    .map_err(|e| format!("Failed to parse response: {}", e))
            } else {
                // Try to parse error response
                if let Ok(error) = response.json::<OAuthError>().await {
                    Err(format!(
                        "OAuth error: {} - {}",
                        error.error,
                        error.error_description.unwrap_or_default()
                    ))
                } else {
                    Err(format!("Token exchange failed: HTTP {}", response.status()))
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = code; // Use variable to silence warning
            Err("OAuth not available in non-WASM environment".to_string())
        }
    }

    /// Parse authorization callback URL
    pub fn parse_callback_url(url: &str) -> Result<(String, String), String> {
        let url = web_sys::Url::new(url).map_err(|_| "Invalid URL")?;
        let params = url.search_params();

        let code = params.get("code").ok_or("Missing authorization code")?;
        let state = params.get("state").ok_or("Missing state parameter")?;

        // Check for error
        if let Some(error) = params.get("error") {
            let error_description = params.get("error_description").unwrap_or_default();
            return Err(format!("OAuth error: {} - {}", error, error_description));
        }

        Ok((code, state))
    }

    /// Store state in session storage for verification
    #[cfg(target_arch = "wasm32")]
    pub fn store_state(state: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("oauth_state", state);
        }
    }

    /// Verify state from callback matches stored state
    #[cfg(target_arch = "wasm32")]
    pub fn verify_state(state: &str) -> bool {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
            && let Ok(Some(stored_state)) = storage.get_item("oauth_state")
        {
            let _ = storage.remove_item("oauth_state"); // Remove after verification
            return stored_state == state;
        }
        false
    }

    /// Store return URL for post-authentication redirect
    #[cfg(target_arch = "wasm32")]
    pub fn store_return_url(url: &str) {
        if let Some(storage) = web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
        {
            let _ = storage.set_item("oauth_return_url", url);
        }
    }

    /// Get stored return URL
    #[cfg(target_arch = "wasm32")]
    pub fn get_return_url() -> Option<String> {
        web_sys::window()
            .and_then(|w| w.session_storage().ok())
            .flatten()
            .and_then(|storage| {
                let url = storage.get_item("oauth_return_url").ok().flatten();
                let _ = storage.remove_item("oauth_return_url"); // Remove after retrieval
                url
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_client_creation() {
        let client = OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal".to_string(),
            "http://localhost:8080/callback".to_string(),
        );

        assert_eq!(client.base_url, "http://localhost:8088");
        assert_eq!(client.realm, "simpel");
        assert_eq!(client.client_id, "portal");
    }

    #[test]
    fn test_authorization_endpoint() {
        let client = OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal".to_string(),
            "http://localhost:8080/callback".to_string(),
        );

        assert_eq!(
            client.authorization_endpoint(),
            "http://localhost:8088/api/v1/oauth2/authorize"
        );
    }

    #[test]
    fn test_token_endpoint() {
        let client = OAuthClient::new(
            "http://localhost:8088".to_string(),
            "simpel".to_string(),
            "portal".to_string(),
            "http://localhost:8080/callback".to_string(),
        );

        assert_eq!(
            client.token_endpoint(),
            "http://localhost:8088/api/v1/oauth2/token"
        );
    }

    #[test]
    fn test_generate_state() {
        let state1 = OAuthClient::generate_state();
        let state2 = OAuthClient::generate_state();

        assert_ne!(state1, state2);
        assert!(!state1.is_empty());
    }
}
