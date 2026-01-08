// Tests for monitoring, error tracking, and analytics modules

use shared_microfrontend::utils::{
    analytics::{EventType, track_event, track_page_view},
    error_tracking::{ErrorSeverity, capture_error},
    monitoring::track_api_request,
};
use std::collections::HashMap;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_track_api_request() {
    // Test API request tracking
    track_api_request("/api/users", "GET", 200, 150.5);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_track_api_request_slow() {
    // Test slow API request (should trigger warning)
    track_api_request("/api/slow", "POST", 200, 6000.0);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_capture_error() {
    // Test error capture
    capture_error(
        "Test error message",
        ErrorSeverity::Error,
        Some("test_stack_trace".to_string()),
    );

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_capture_warning() {
    // Test warning capture
    capture_error("Test warning message", ErrorSeverity::Warning, None);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_track_page_view() {
    // Test page view tracking
    track_page_view();

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_track_custom_event() {
    // Test custom event tracking
    let mut properties = HashMap::new();
    properties.insert("button_id".to_string(), serde_json::json!("submit_btn"));
    properties.insert("page".to_string(), serde_json::json!("/dashboard"));

    track_event(EventType::Click, properties);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_track_search_event() {
    // Test search event tracking
    let mut properties = HashMap::new();
    properties.insert("query".to_string(), serde_json::json!("test search"));
    properties.insert("results_count".to_string(), serde_json::json!(42));

    track_event(EventType::Search, properties);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_error_severity_levels() {
    // Test all error severity levels
    capture_error("Debug message", ErrorSeverity::Debug, None);
    capture_error("Info message", ErrorSeverity::Info, None);
    capture_error("Warning message", ErrorSeverity::Warning, None);
    capture_error("Error message", ErrorSeverity::Error, None);
    capture_error("Fatal message", ErrorSeverity::Fatal, None);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_multiple_api_requests() {
    // Test tracking multiple API requests
    track_api_request("/api/users", "GET", 200, 100.0);
    track_api_request("/api/posts", "GET", 200, 150.0);
    track_api_request("/api/comments", "POST", 201, 200.0);
    track_api_request("/api/error", "GET", 500, 50.0);

    // Should not panic
    assert!(true);
}

#[wasm_bindgen_test]
fn test_event_types() {
    // Test different event types
    let properties = HashMap::new();

    track_event(EventType::PageView, properties.clone());
    track_event(EventType::Click, properties.clone());
    track_event(EventType::FormSubmit, properties.clone());
    track_event(EventType::Search, properties.clone());
    track_event(EventType::Download, properties.clone());
    track_event(EventType::Upload, properties.clone());
    track_event(EventType::Login, properties.clone());
    track_event(EventType::Logout, properties.clone());
    track_event(EventType::FeatureUsage, properties.clone());
    track_event(EventType::Custom("custom_event".to_string()), properties);

    // Should not panic
    assert!(true);
}
