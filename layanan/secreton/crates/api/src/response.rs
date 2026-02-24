//! Common API response types

use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
    pub metadata: ResponseMetadata,
}

impl<T> ApiResponse<T> {
    /// Create successful response
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata::new(),
        }
    }

    /// Create error response
    pub fn error(error: ErrorDetails) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: ResponseMetadata::new(),
        }
    }
}

/// Error details in API response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Response metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl ResponseMetadata {
    pub fn new() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
            trace_id: None,
        }
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

impl Default for ResponseMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub dependencies: HealthCheckDependencies,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_unseal: Option<AutoUnsealStatus>,
}

/// Auto-unseal status information
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoUnsealStatus {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_endpoint: Option<String>,
    pub provider_healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_unseal: Option<chrono::DateTime<chrono::Utc>>,
    pub fallback_enabled: bool,
}

/// Health check dependencies status
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckDependencies {
    pub storage: DependencyStatus,
    pub crypto: DependencyStatus,
    pub audit: DependencyStatus,
}

/// Individual dependency status
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub healthy: bool,
    pub message: Option<String>,
    pub response_time_ms: Option<u64>,
}

impl DependencyStatus {
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            message: None,
            response_time_ms: None,
        }
    }

    pub fn unhealthy(message: String) -> Self {
        Self {
            healthy: false,
            message: Some(message),
            response_time_ms: None,
        }
    }
}
