//! OAuth2/OIDC service implementation
//!
//! This module implements OAuth 2.1 compliant authorization flows:
//! - Authorization Code flow with PKCE (mandatory)
//! - Client Credentials flow
//! - Refresh Token flow
//!
//! Key security features:
//! - PKCE enforcement for all clients (OAuth 2.1 requirement)
//! - Single-use authorization codes
//! - Refresh token rotation
//! - Strict redirect URI validation

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;

use authenc_types::{
    AuthorizationCode, AuthorizationCodeStore, AuthorizationRequest, AuthorizationResponse,
    ClientStore, OAuth2Error, OAuth2Service as OAuth2ServiceTrait, OidcClient, RefreshToken,
    RefreshTokenStore, Result, TokenGenerator, TokenRequest, TokenResponse, UserId,
};

/// OAuth2 service implementation
pub struct OAuth2ServiceImpl {
    /// Client store for OAuth2 client management
    client_store: Arc<dyn ClientStore>,
    /// Authorization code store
    code_store: Arc<dyn AuthorizationCodeStore>,
    /// Refresh token store
    refresh_token_store: Arc<dyn RefreshTokenStore>,
    /// Token generator for JWT access tokens
    token_generator: Arc<dyn TokenGenerator>,
}

impl OAuth2ServiceImpl {
    /// Create a new OAuth2 service
    pub fn new(
        client_store: Arc<dyn ClientStore>,
        code_store: Arc<dyn AuthorizationCodeStore>,
        refresh_token_store: Arc<dyn RefreshTokenStore>,
        token_generator: Arc<dyn TokenGenerator>,
    ) -> Self {
        Self {
            client_store,
            code_store,
            refresh_token_store,
            token_generator,
        }
    }

    /// Generate a random authorization code
    fn generate_authorization_code() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; 32];
        rng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Generate a random refresh token
    fn generate_refresh_token_value() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = vec![0u8; 32];
        rng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Handle authorization code grant
    async fn handle_authorization_code_grant(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
        client_id: &str,
        realm_id: authenc_types::RealmId,
    ) -> Result<TokenResponse> {
        // 1. Retrieve authorization code
        let auth_code = self
            .code_store
            .get_code(code)
            .await?
            .ok_or_else(|| OAuth2Error::invalid_grant("Invalid authorization code"))?;

        // 2. Validate authorization code
        if auth_code.used {
            return Err(OAuth2Error::invalid_grant("Authorization code already used").into());
        }

        if auth_code.expires_at < Utc::now() {
            return Err(OAuth2Error::invalid_grant("Authorization code expired").into());
        }

        if auth_code.client_id != client_id {
            return Err(OAuth2Error::invalid_grant("Client ID mismatch").into());
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err(OAuth2Error::invalid_grant("Redirect URI mismatch").into());
        }

        if auth_code.realm_id != realm_id {
            return Err(OAuth2Error::invalid_grant("Realm mismatch").into());
        }

        // 3. Verify PKCE code challenge (OAuth 2.1 requirement)
        self.verify_pkce(
            code_verifier,
            &auth_code.code_challenge,
            &auth_code.code_challenge_method,
        )?;

        // 4. Mark authorization code as used
        self.code_store.mark_code_used(code).await?;

        // 5. Generate access token
        let access_token = self
            .token_generator
            .generate_access_token(auth_code.user_id, &auth_code.scope)?;

        // 6. Generate refresh token
        let refresh_token_value = Self::generate_refresh_token_value();
        let refresh_token = RefreshToken {
            token: refresh_token_value.clone(),
            user_id: auth_code.user_id,
            client_id: client_id.to_string(),
            scope: auth_code.scope.clone(),
            realm_id,
            expires_at: Utc::now() + Duration::days(7),
            created_at: Utc::now(),
            revoked: false,
        };

        self.refresh_token_store.store_token(refresh_token).await?;

        // 7. Return token response
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 900, // 15 minutes
            refresh_token: Some(refresh_token_value),
            scope: auth_code.scope,
        })
    }

    /// Handle client credentials grant
    async fn handle_client_credentials_grant(
        &self,
        client: &OidcClient,
        scope: &str,
    ) -> Result<TokenResponse> {
        // 1. Validate client is confidential (has secret)
        if client.is_public {
            return Err(
                OAuth2Error::unauthorized_client("Public clients cannot use client credentials")
                    .into(),
            );
        }

        // 2. Validate scopes
        self.validate_scopes(client, scope)?;

        // 3. Generate access token (no user context, client-only)
        // Use a special "client" user ID or service account
        let service_user_id = UserId::new(); // TODO: Use actual service account ID
        let access_token = self.token_generator.generate_access_token(service_user_id, scope)?;

        // 4. Return token response (no refresh token for client credentials)
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 900, // 15 minutes
            refresh_token: None,
            scope: scope.to_string(),
        })
    }

    /// Handle refresh token grant
    async fn handle_refresh_token_grant(
        &self,
        refresh_token_value: &str,
        client_id: &str,
        realm_id: authenc_types::RealmId,
    ) -> Result<TokenResponse> {
        // 1. Retrieve refresh token
        let refresh_token = self
            .refresh_token_store
            .get_token(refresh_token_value)
            .await?
            .ok_or_else(|| OAuth2Error::invalid_grant("Invalid refresh token"))?;

        // 2. Validate refresh token
        if refresh_token.revoked {
            return Err(OAuth2Error::invalid_grant("Refresh token revoked").into());
        }

        if refresh_token.expires_at < Utc::now() {
            return Err(OAuth2Error::invalid_grant("Refresh token expired").into());
        }

        if refresh_token.client_id != client_id {
            return Err(OAuth2Error::invalid_grant("Client ID mismatch").into());
        }

        if refresh_token.realm_id != realm_id {
            return Err(OAuth2Error::invalid_grant("Realm mismatch").into());
        }

        // 3. Revoke old refresh token (token rotation)
        self.refresh_token_store
            .revoke_token(refresh_token_value)
            .await?;

        // 4. Generate new access token
        let access_token = self
            .token_generator
            .generate_access_token(refresh_token.user_id, &refresh_token.scope)?;

        // 5. Generate new refresh token (rotation)
        let new_refresh_token_value = Self::generate_refresh_token_value();
        let new_refresh_token = RefreshToken {
            token: new_refresh_token_value.clone(),
            user_id: refresh_token.user_id,
            client_id: client_id.to_string(),
            scope: refresh_token.scope.clone(),
            realm_id,
            expires_at: Utc::now() + Duration::days(7),
            created_at: Utc::now(),
            revoked: false,
        };

        self.refresh_token_store
            .store_token(new_refresh_token)
            .await?;

        // 6. Return token response
        Ok(TokenResponse {
            access_token,
            token_type: "Bearer".to_string(),
            expires_in: 900, // 15 minutes
            refresh_token: Some(new_refresh_token_value),
            scope: refresh_token.scope,
        })
    }

    /// Authenticate client (verify client secret for confidential clients)
    async fn authenticate_client(
        &self,
        client_id: &str,
        client_secret: Option<&str>,
        realm_id: authenc_types::RealmId,
    ) -> Result<OidcClient> {
        // 1. Get client from store
        let client = self.client_store.get_client_by_client_id(client_id, realm_id).await?;

        // 2. Check if client is enabled
        if !client.enabled {
            return Err(OAuth2Error::invalid_client("Client is disabled").into());
        }

        // 3. Verify client secret for confidential clients
        if !client.is_public {
            let provided_secret = client_secret
                .ok_or_else(|| OAuth2Error::invalid_client("Client secret required"))?;

            let stored_secret_hash = client
                .client_secret_hash
                .as_ref()
                .ok_or_else(|| OAuth2Error::invalid_client("Client secret not configured"))?;

            // TODO: Use proper password hasher for client secret verification
            // For now, simple comparison (should use Argon2 in production)
            if provided_secret != stored_secret_hash {
                return Err(OAuth2Error::invalid_client("Invalid client secret").into());
            }
        }

        Ok(client)
    }
}

#[async_trait]
impl OAuth2ServiceTrait for OAuth2ServiceImpl {
    async fn authorize(&self, request: AuthorizationRequest) -> Result<AuthorizationResponse> {
        // 1. Validate response_type
        if request.response_type != "code" {
            return Err(OAuth2Error::unsupported_grant_type("Only 'code' response type is supported").into());
        }

        // 2. Get and validate client
        let client = self
            .client_store
            .get_client_by_client_id(&request.client_id, request.realm_id)
            .await?;

        if !client.enabled {
            return Err(OAuth2Error::invalid_client("Client is disabled").into());
        }

        // 3. Validate redirect URI
        self.validate_redirect_uri(&client, &request.redirect_uri)?;

        // 4. Validate scopes
        self.validate_scopes(&client, &request.scope)?;

        // 5. Validate PKCE parameters (OAuth 2.1 requirement)
        if request.code_challenge.is_empty() {
            return Err(OAuth2Error::invalid_request("code_challenge is required (OAuth 2.1)").into());
        }

        if request.code_challenge_method != "S256" && request.code_challenge_method != "plain" {
            return Err(OAuth2Error::invalid_request("code_challenge_method must be S256 or plain").into());
        }

        // Prefer S256 over plain
        if request.code_challenge_method == "plain" {
            tracing::warn!("Client using plain PKCE method, S256 is recommended");
        }

        // 6. Generate authorization code
        let code = Self::generate_authorization_code();

        // 7. Store authorization code
        let auth_code = AuthorizationCode {
            code: code.clone(),
            client_id: request.client_id.clone(),
            user_id: request.user_id,
            redirect_uri: request.redirect_uri.clone(),
            scope: request.scope.clone(),
            code_challenge: request.code_challenge.clone(),
            code_challenge_method: request.code_challenge_method.clone(),
            realm_id: request.realm_id,
            expires_at: Utc::now() + Duration::minutes(10),
            created_at: Utc::now(),
            used: false,
        };

        self.code_store.store_code(auth_code).await?;

        // 8. Return authorization response
        Ok(AuthorizationResponse {
            code,
            state: request.state,
        })
    }

    async fn token(&self, request: TokenRequest) -> Result<TokenResponse> {
        match request.grant_type.as_str() {
            "authorization_code" => {
                // Authorization Code flow
                let code = request
                    .code
                    .as_ref()
                    .ok_or_else(|| OAuth2Error::invalid_request("code is required"))?;

                let redirect_uri = request
                    .redirect_uri
                    .as_ref()
                    .ok_or_else(|| OAuth2Error::invalid_request("redirect_uri is required"))?;

                let code_verifier = request
                    .code_verifier
                    .as_ref()
                    .ok_or_else(|| OAuth2Error::invalid_request("code_verifier is required (PKCE)"))?;

                // Authenticate client (public clients don't need secret)
                let client = self
                    .authenticate_client(
                        &request.client_id,
                        request.client_secret.as_deref(),
                        request.realm_id,
                    )
                    .await?;

                self.handle_authorization_code_grant(
                    code,
                    redirect_uri,
                    code_verifier,
                    &client.client_id,
                    request.realm_id,
                )
                .await
            }
            "client_credentials" => {
                // Client Credentials flow
                let scope = request
                    .scope
                    .as_ref()
                    .ok_or_else(|| OAuth2Error::invalid_request("scope is required"))?;

                // Authenticate client (must be confidential)
                let client = self
                    .authenticate_client(
                        &request.client_id,
                        request.client_secret.as_deref(),
                        request.realm_id,
                    )
                    .await?;

                self.handle_client_credentials_grant(&client, scope).await
            }
            "refresh_token" => {
                // Refresh Token flow
                let refresh_token = request
                    .refresh_token
                    .as_ref()
                    .ok_or_else(|| OAuth2Error::invalid_request("refresh_token is required"))?;

                // Authenticate client
                let client = self
                    .authenticate_client(
                        &request.client_id,
                        request.client_secret.as_deref(),
                        request.realm_id,
                    )
                    .await?;

                self.handle_refresh_token_grant(refresh_token, &client.client_id, request.realm_id)
                    .await
            }
            _ => Err(OAuth2Error::unsupported_grant_type(format!(
                "Unsupported grant type: {}",
                request.grant_type
            ))
            .into()),
        }
    }

    fn validate_redirect_uri(&self, client: &OidcClient, redirect_uri: &str) -> Result<()> {
        // Exact match required (OAuth 2.1 security best practice)
        if !client.redirect_uris.contains(&redirect_uri.to_string()) {
            return Err(OAuth2Error::invalid_request("Invalid redirect_uri").into());
        }

        Ok(())
    }

    fn validate_scopes(&self, client: &OidcClient, requested_scopes: &str) -> Result<()> {
        let requested: Vec<&str> = requested_scopes.split_whitespace().collect();

        for scope in requested {
            if !client.allowed_scopes.contains(&scope.to_string()) {
                return Err(OAuth2Error::invalid_scope(format!("Scope '{}' not allowed", scope)).into());
            }
        }

        Ok(())
    }

    fn verify_pkce(&self, code_verifier: &str, code_challenge: &str, method: &str) -> Result<()> {
        let computed_challenge = match method {
            "S256" => {
                // SHA256 hash of code_verifier, then base64url encode
                let mut hasher = Sha256::new();
                hasher.update(code_verifier.as_bytes());
                let hash = hasher.finalize();
                URL_SAFE_NO_PAD.encode(&hash)
            }
            "plain" => {
                // Plain code_verifier
                code_verifier.to_string()
            }
            _ => {
                return Err(OAuth2Error::invalid_request("Invalid code_challenge_method").into());
            }
        };

        if computed_challenge != code_challenge {
            return Err(OAuth2Error::invalid_grant("Invalid code_verifier").into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_s256() {
        let service = OAuth2ServiceImpl::new(
            Arc::new(MockClientStore),
            Arc::new(MockCodeStore),
            Arc::new(MockRefreshTokenStore),
            Arc::new(MockTokenGenerator),
        );

        // Test vector from RFC 7636
        let code_verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let code_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        let result = service.verify_pkce(code_verifier, code_challenge, "S256");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pkce_plain() {
        let service = OAuth2ServiceImpl::new(
            Arc::new(MockClientStore),
            Arc::new(MockCodeStore),
            Arc::new(MockRefreshTokenStore),
            Arc::new(MockTokenGenerator),
        );

        let code_verifier = "test_verifier";
        let code_challenge = "test_verifier";

        let result = service.verify_pkce(code_verifier, code_challenge, "plain");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pkce_invalid() {
        let service = OAuth2ServiceImpl::new(
            Arc::new(MockClientStore),
            Arc::new(MockCodeStore),
            Arc::new(MockRefreshTokenStore),
            Arc::new(MockTokenGenerator),
        );

        let code_verifier = "wrong_verifier";
        let code_challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        let result = service.verify_pkce(code_verifier, code_challenge, "S256");
        assert!(result.is_err());
    }

    // Mock implementations for testing
    struct MockClientStore;
    struct MockCodeStore;
    struct MockRefreshTokenStore;
    struct MockTokenGenerator;

    #[async_trait]
    impl ClientStore for MockClientStore {
        async fn get_client(&self, _id: authenc_types::ClientId) -> Result<OidcClient> {
            unimplemented!()
        }

        async fn get_client_by_client_id(
            &self,
            _client_id: &str,
            _realm_id: authenc_types::RealmId,
        ) -> Result<OidcClient> {
            unimplemented!()
        }

        async fn create_client(
            &self,
            _client_id: String,
            _name: String,
            _is_public: bool,
            _realm_id: authenc_types::RealmId,
        ) -> Result<OidcClient> {
            unimplemented!()
        }

        async fn update_client(
            &self,
            _id: authenc_types::ClientId,
            _name: Option<String>,
            _redirect_uris: Option<Vec<String>>,
            _allowed_scopes: Option<Vec<String>>,
            _enabled: Option<bool>,
        ) -> Result<OidcClient> {
            unimplemented!()
        }

        async fn delete_client(&self, _id: authenc_types::ClientId) -> Result<()> {
            unimplemented!()
        }

        async fn list_clients(&self, _realm_id: authenc_types::RealmId) -> Result<Vec<OidcClient>> {
            unimplemented!()
        }

        async fn update_client_secret(
            &self,
            _id: authenc_types::ClientId,
            _secret_hash: String,
        ) -> Result<()> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl AuthorizationCodeStore for MockCodeStore {
        async fn store_code(&self, _code: AuthorizationCode) -> Result<()> {
            Ok(())
        }

        async fn get_code(&self, _code: &str) -> Result<Option<AuthorizationCode>> {
            Ok(None)
        }

        async fn mark_code_used(&self, _code: &str) -> Result<()> {
            Ok(())
        }

        async fn cleanup_expired_codes(&self) -> Result<usize> {
            Ok(0)
        }
    }

    #[async_trait]
    impl RefreshTokenStore for MockRefreshTokenStore {
        async fn store_token(&self, _token: RefreshToken) -> Result<()> {
            Ok(())
        }

        async fn get_token(&self, _token: &str) -> Result<Option<RefreshToken>> {
            Ok(None)
        }

        async fn revoke_token(&self, _token: &str) -> Result<()> {
            Ok(())
        }

        async fn revoke_user_tokens(&self, _user_id: UserId) -> Result<()> {
            Ok(())
        }

        async fn cleanup_expired_tokens(&self) -> Result<usize> {
            Ok(0)
        }
    }

    impl TokenGenerator for MockTokenGenerator {
        fn generate_access_token(&self, _user_id: UserId, _scope: &str) -> Result<String> {
            Ok("mock_access_token".to_string())
        }

        fn generate_refresh_token(&self, _user_id: UserId) -> Result<String> {
            Ok("mock_refresh_token".to_string())
        }

        fn validate_token(&self, _token: &str) -> Result<authenc_types::TokenClaims> {
            unimplemented!()
        }
    }
}
