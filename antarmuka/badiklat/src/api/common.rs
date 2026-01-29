//! Common API utilities
//!
//! Shared API helper functions and types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: usize,
    pub limit: usize,
    pub total: usize,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: 1,
            limit: 20,
            total: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMeta {
    pub pagination: Option<PaginationParams>,
    pub timestamp: i64,
}

/// Format error message for display
pub fn format_error(error: &super::ApiError) -> String {
    match error {
        super::ApiError::NetworkError(msg) => format!("Network error: {}", msg),
        super::ApiError::SerializationError(msg) => format!("Data error: {}", msg),
        super::ApiError::ServerError(code, msg) => format!("Server error {}: {}", code, msg),
        super::ApiError::Unauthorized => "Please login to continue".to_string(),
    }
}
