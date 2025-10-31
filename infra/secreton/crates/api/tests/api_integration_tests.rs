//! Integration tests for API endpoints

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use secreton_api::{create_api_router, ApiState, KVApiState, TransitApiState};
use serde_json::json;
use tower::ServiceExt;

fn create_test_app() -> axum::Router {
    let state = ApiState {
        transit: TransitApiState::default(),
        kv: KVApiState::default(),
    };
    
    create_api_router(state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert_eq!(json["status"], "healthy");
    assert!(json["timestamp"].is_string());
    assert_eq!(json["version"], "1.0.0");
}

#[tokio::test]
async fn test_version_endpoint() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/version")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert!(json["version"].is_string());
    assert_eq!(json["build_date"], "2024");
}

#[tokio::test]
async fn test_transit_create_key() {
    let app = create_test_app();
    
    let request_body = json!({
        "key_type": "aes256-gcm"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/transit/keys/test-key")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&request_body)))
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_transit_list_keys() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/transit/keys")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert!(json["keys"].is_array());
}

#[tokio::test]
async fn test_kv_put_and_get_secret() {
    let app = create_test_app();
    
    // Put secret
    let put_body = json!({
        "data": {
            "password": "secret123",
            "api_key": "abc-def-ghi"
        }
    });
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/secret/data/myapp")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&put_body)))
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    // Get secret
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/secret/data/myapp")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert_eq!(json["data"]["password"], "secret123");
    assert_eq!(json["data"]["api_key"], "abc-def-ghi");
}

#[tokio::test]
async fn test_kv_list_secrets() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/v1/secrets")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert!(json["secrets"].is_array());
}

#[tokio::test]
async fn test_kv_delete_secret() {
    let app = create_test_app();
    
    // First put a secret
    let put_body = json!({
        "data": {
            "temp": "value"
        }
    });
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/secret/data/temp-secret")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&put_body)))
                ,
        )
        .await
        ;
    
    // Delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/v1/secret/data/temp-secret")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_tls_metrics_endpoint() {
    let app = create_test_app();
    
    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics/tls")
                .body(Body::empty())
                ,
        )
        .await
        ;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        ;
    let json: serde_json::Value = serde_json::from_slice(&body);
    
    assert!(json["total_handshakes"].is_number());
    assert!(json["success_rate_percent"].is_number());
}
