//! Standard API response envelopes used by both backend and frontend.
//!
//! Defining these once in `lib-perlengkapan` removes the duplication that
//! existed between the service's `api/src/models.rs` and the frontend's
//! `antarmuka/perlengkapan/src/api/common.rs`. Both sides now serialize and
//! deserialize the exact same shape.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Successful single-resource response envelope.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse<T> {
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// Paginated list response envelope.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PageMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: PageMeta) -> Self {
        Self { data, pagination }
    }
}

/// Pagination metadata returned alongside list endpoints.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl PageMeta {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = if per_page == 0 {
            0
        } else {
            ((total as f64) / (per_page as f64)).ceil() as u32
        };
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// Structured error body returned for non-2xx responses.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[cfg(feature = "serde")]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub details: Option<serde_json::Value>,
}

impl ErrorBody {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetail {
                code: code.into(),
                message: message.into(),
                #[cfg(feature = "serde")]
                details: None,
            },
        }
    }

    #[cfg(feature = "serde")]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.error.details = Some(details);
        self
    }
}
