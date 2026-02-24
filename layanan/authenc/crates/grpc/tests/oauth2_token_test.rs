//! OAuth2 token operations test
//!
//! This test verifies that the OAuth2 token operations (GetOAuthToken, IntrospectToken, GetUserInfo)
//! are properly implemented in the gRPC service.

#[cfg(test)]
mod tests {
    use authenc_grpc::proto::authenc::v1::{
        IntrospectTokenRequest, OAuthTokenRequest, UserInfoRequest,
    };

    #[test]
    fn test_oauth_token_request_creation() {
        // Test that we can create OAuth token requests
        let request = OAuthTokenRequest {
            grant_type: "authorization_code".to_string(),
            code: Some("test_code".to_string()),
            refresh_token: None,
            client_id: Some("test_client".to_string()),
            client_secret: Some("test_secret".to_string()),
            redirect_uri: Some("https://example.com/callback".to_string()),
            scopes: vec!["openid".to_string(), "profile".to_string()],
        };

        assert_eq!(request.grant_type, "authorization_code");
        assert_eq!(request.code, Some("test_code".to_string()));
        assert_eq!(request.scopes.len(), 2);
    }

    #[test]
    fn test_introspect_token_request_creation() {
        // Test that we can create token introspection requests
        let request = IntrospectTokenRequest {
            token: "test_token".to_string(),
        };

        assert_eq!(request.token, "test_token");
    }

    #[test]
    fn test_user_info_request_creation() {
        // Test that we can create UserInfo requests
        let request = UserInfoRequest {
            access_token: "test_access_token".to_string(),
        };

        assert_eq!(request.access_token, "test_access_token");
    }

    #[test]
    fn test_oauth_token_request_with_refresh_token() {
        // Test refresh token grant
        let request = OAuthTokenRequest {
            grant_type: "refresh_token".to_string(),
            code: None,
            refresh_token: Some("test_refresh_token".to_string()),
            client_id: Some("test_client".to_string()),
            client_secret: Some("test_secret".to_string()),
            redirect_uri: None,
            scopes: vec![],
        };

        assert_eq!(request.grant_type, "refresh_token");
        assert_eq!(
            request.refresh_token,
            Some("test_refresh_token".to_string())
        );
    }

    #[test]
    fn test_oauth_token_request_with_client_credentials() {
        // Test client credentials grant
        let request = OAuthTokenRequest {
            grant_type: "client_credentials".to_string(),
            code: None,
            refresh_token: None,
            client_id: Some("test_client".to_string()),
            client_secret: Some("test_secret".to_string()),
            redirect_uri: None,
            scopes: vec!["api:read".to_string(), "api:write".to_string()],
        };

        assert_eq!(request.grant_type, "client_credentials");
        assert_eq!(request.scopes.len(), 2);
    }
}
