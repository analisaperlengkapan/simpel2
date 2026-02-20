//! Integration tests for authenc-grpc service

use authenc_grpc::{
    proto::authenc::v1::*,
    AuthencGrpcService, AuthencServiceServer, GrpcServerBuilder, GrpcServerConfig,
};
use std::net::SocketAddr;
use tonic::transport::Channel;
use tonic::Request;

// Mock implementations for testing
// In real tests, these would use actual service implementations

/// Helper to create a test gRPC server
async fn create_test_server() -> SocketAddr {
    // TODO: Create mock services
    // For now, return a placeholder address
    "127.0.0.1:50051".parse().unwrap()
}

/// Helper to create a test gRPC client
async fn create_test_client(addr: SocketAddr) -> authenc_service_client::AuthencServiceClient<Channel> {
    let endpoint = format!("http://{}", addr);
    authenc_service_client::AuthencServiceClient::connect(endpoint)
        .await
        .expect("Failed to connect to test server")
}

#[tokio::test]
#[ignore] // Ignore until mock services are implemented
async fn test_authenticate_rpc() {
    // Start test server
    let addr = create_test_server().await;

    // Create client
    let mut client = create_test_client(addr).await;

    // Test authentication
    let request = Request::new(AuthenticateRequest {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        mfa_code: None,
        device_id: None,
        metadata: std::collections::HashMap::new(),
        captcha_token: None,
    });

    let response = client.authenticate(request).await;
    assert!(response.is_ok());

    let auth_response = response.unwrap().into_inner();
    assert!(!auth_response.access_token.is_empty());
    assert!(!auth_response.refresh_token.is_empty());
    assert_eq!(auth_response.token_type, "Bearer");
}

#[tokio::test]
#[ignore] // Ignore until mock services are implemented
async fn test_validate_token_rpc() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    // First authenticate to get a token
    let auth_request = Request::new(AuthenticateRequest {
        username: "testuser".to_string(),
        password: "testpass".to_string(),
        mfa_code: None,
        device_id: None,
        metadata: std::collections::HashMap::new(),
        captcha_token: None,
    });

    let auth_response = client
        .authenticate(auth_request)
        .await
        .unwrap()
        .into_inner();

    // Validate the token
    let validate_request = Request::new(ValidateTokenRequest {
        token: auth_response.access_token,
        required_scopes: vec![],
    });

    let response = client.validate_token(validate_request).await;
    assert!(response.is_ok());

    let validate_response = response.unwrap().into_inner();
    assert!(validate_response.valid);
    assert!(validate_response.user_id.is_some());
}

#[tokio::test]
#[ignore] // Ignore until mock services are implemented
async fn test_create_user_rpc() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    let request = Request::new(CreateUserRequest {
        username: "newuser".to_string(),
        email: "newuser@example.com".to_string(),
        password: "securepass123".to_string(),
        full_name: Some("New User".to_string()),
        roles: vec!["user".to_string()],
        metadata: std::collections::HashMap::new(),
    });

    let response = client.create_user(request).await;
    assert!(response.is_ok());

    let create_response = response.unwrap().into_inner();
    assert!(!create_response.user_id.is_empty());
    assert_eq!(create_response.username, "newuser");
}

#[tokio::test]
#[ignore] // Ignore until mock services are implemented
async fn test_get_user_rpc() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    // Create a user first
    let create_request = Request::new(CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "pass123".to_string(),
        full_name: Some("Test User".to_string()),
        roles: vec![],
        metadata: std::collections::HashMap::new(),
    });

    let create_response = client
        .create_user(create_request)
        .await
        .unwrap()
        .into_inner();

    // Get the user
    let get_request = Request::new(GetUserRequest {
        user_id: create_response.user_id.clone(),
    });

    let response = client.get_user(get_request).await;
    assert!(response.is_ok());

    let get_response = response.unwrap().into_inner();
    assert!(get_response.user.is_some());

    let user = get_response.user.unwrap();
    assert_eq!(user.user_id, create_response.user_id);
    assert_eq!(user.username, "testuser");
}

#[tokio::test]
#[ignore] // Ignore until mock services are implemented
async fn test_health_check_rpc() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    let request = Request::new(authenc_grpc::proto::common::v1::HealthCheckRequest {
        service: "authenc-grpc".to_string(),
    });

    let response = client.health_check(request).await;
    assert!(response.is_ok());

    let health_response = response.unwrap().into_inner();
    assert!(health_response.info.is_some());

    let info = health_response.info.unwrap();
    assert_eq!(info.service_name, "authenc-grpc");
    assert_eq!(
        info.status,
        authenc_grpc::proto::common::v1::HealthStatus::Healthy as i32
    );
}

#[tokio::test]
#[ignore] // Ignore until mTLS is configured
async fn test_mtls_connection() {
    // TODO: Test mTLS connection with client certificates
    // This requires setting up test certificates and configuring both
    // server and client with TLS
}

#[tokio::test]
#[ignore] // Ignore until error handling is complete
async fn test_error_handling() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    // Test authentication with invalid credentials
    let request = Request::new(AuthenticateRequest {
        username: "invalid".to_string(),
        password: "wrong".to_string(),
        mfa_code: None,
        device_id: None,
        metadata: std::collections::HashMap::new(),
        captcha_token: None,
    });

    let response = client.authenticate(request).await;
    assert!(response.is_err());

    let status = response.unwrap_err();
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
#[ignore] // Ignore until token validation is complete
async fn test_invalid_token_validation() {
    let addr = create_test_server().await;
    let mut client = create_test_client(addr).await;

    let request = Request::new(ValidateTokenRequest {
        token: "invalid.token.here".to_string(),
        required_scopes: vec![],
    });

    let response = client.validate_token(request).await;
    assert!(response.is_ok());

    let validate_response = response.unwrap().into_inner();
    assert!(!validate_response.valid);
    assert!(validate_response.error.is_some());
}

#[test]
fn test_grpc_server_config_default() {
    let config = GrpcServerConfig::default();
    assert_eq!(config.bind_address.port(), 50051);
    assert!(config.enable_logging);
    assert!(config.enable_auth);
}
