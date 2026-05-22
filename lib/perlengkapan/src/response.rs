//! Standard API response envelopes used by **both** the backend service and
//! the WASM frontend.
//!
//! Defining the wire types once here removes the duplication that used to
//! live in `layanan-perlengkapan::models` and
//! `antarmuka/perlengkapan::api::common`. Both sides now serialize and
//! deserialize the exact same shape, so adding or removing a field is a
//! single-call change.
//!
//! ## Wire shape (unchanged from the pre-refactor envelope)
//!
//! ```json
//! // ApiResponse<T>
//! { "success": true, "data": { ... }, "message": "..." }
//!
//! // PaginatedResponse<T>
//! {
//!   "success": true,
//!   "data": [ ... ],
//!   "total": 0,
//!   "page": 1,
//!   "per_page": 25,
//!   "total_pages": 0,
//!   "message": "..."
//! }
//! ```
//!
//! [`PageMeta`] + [`ErrorBody`] are forward-looking helper types — handy
//! when emitting structured error bodies or carrying pagination metadata
//! out-of-band — but are not part of the on-wire envelope above.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Successful single-resource response envelope.
///
/// Matches the wire format the existing backend handlers emit through
/// `ApiResponse::success(data, message)`, so the frontend can re-export
/// this type and `serde` deserializes it unchanged.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: String,
}

impl<T> ApiResponse<T> {
    /// Build a successful response. `success` is always `true` here —
    /// failures travel through `Result::Err` and an `ErrorBody` on the
    /// non-2xx response, not by setting `success: false` on this envelope.
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data,
            message: message.into(),
        }
    }
}

/// Paginated list response envelope.
///
/// Flat shape mirrors the wire format; `total_pages` is derived from
/// `total` + `per_page` in [`PaginatedResponse::new`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub message: String,
}

impl<T> PaginatedResponse<T> {
    pub fn new(
        data: Vec<T>,
        total: i64,
        page: i32,
        per_page: i32,
        message: impl Into<String>,
    ) -> Self {
        let total_pages = if per_page > 0 {
            ((total as f64) / (per_page as f64)).ceil() as i32
        } else {
            0
        };
        Self {
            success: true,
            data,
            total,
            page,
            per_page,
            total_pages,
            message: message.into(),
        }
    }
}

/// Pagination metadata helper. Not part of the wire envelope above — used
/// internally by repository code that wants a single `PageMeta` it can pass
/// around before flattening into a `PaginatedResponse`.
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

/// Structured error body returned for non-2xx responses. Frontend
/// `api::error::AppError::from_status` already knows how to parse a few
/// common shapes; `ErrorBody` is the canonical shape new handlers should
/// emit.
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
