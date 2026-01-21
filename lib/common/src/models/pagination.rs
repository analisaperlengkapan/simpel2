use serde::{Deserialize, Serialize};

/// Generic pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: Option<i64>,
    /// Items per page
    pub limit: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(10),
        }
    }
}

impl PaginationParams {
    /// Get offset for database queries
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        let limit = self.limit.unwrap_or(10).max(1);
        (page - 1) * limit
    }

    /// Get limit for database queries
    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(10).max(1).min(100)
    }
}

/// Generic paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Data items
    pub items: Vec<T>,
    /// Total number of items
    pub total: i64,
    /// Current page
    pub page: i64,
    /// Items per page
    pub limit: i64,
}
