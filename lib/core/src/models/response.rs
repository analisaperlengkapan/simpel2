use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Whether the request was successful
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// Response data (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
        }
    }

    /// Create a success response without data
    pub fn ok(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: true,
            message: message.into(),
            data: None,
        }
    }
}

/// Wrapper for list responses with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

impl<T> ListResponse<T> {
    pub fn new(
        data: Vec<T>,
        total: i64,
        page: i64,
        per_page: i64,
        message: impl Into<String>,
    ) -> Self {
        let total_pages = if per_page > 0 {
            (total as f64 / per_page as f64).ceil() as i64
        } else {
            0
        };

        Self {
            success: true,
            message: message.into(),
            data,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}
