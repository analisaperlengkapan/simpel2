//! Search Engine for Perlengkapan
//!
//! Provides full-text search with filters, pagination, and relevance ranking

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Search query with filters and pagination
#[derive(Debug, Clone, Validate)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchQuery {
    /// Search term (full-text search)
    #[validate(length(min = 1, max = 255))]
    pub query: String,

    /// Filters
    pub filters: SearchFilters,

    /// Pagination
    pub pagination: Pagination,

    /// Sort options
    pub sort: SortOptions,
}

impl SearchQuery {
    /// Create a new search query with defaults
    pub fn new(query: String) -> Self {
        Self {
            query,
            filters: SearchFilters::default(),
            pagination: Pagination::default(),
            sort: SortOptions::default(),
        }
    }

    /// Set filters
    pub fn with_filters(mut self, filters: SearchFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, pagination: Pagination) -> Self {
        self.pagination = pagination;
        self
    }

    /// Set sort options
    pub fn with_sort(mut self, sort: SortOptions) -> Self {
        self.sort = sort;
        self
    }
}

/// Search filters
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchFilters {
    /// Filter by satker ID
    pub satker_id: Option<Uuid>,

    /// Filter by tahun anggaran
    pub tahun_anggaran: Option<i32>,

    /// Filter by status
    pub status: Option<Vec<String>>,

    /// Filter by kode barang
    pub kode_barang: Option<String>,

    /// Filter by priority level
    pub priority_level: Option<Vec<String>>,

    /// Filter by date range (created_at)
    pub date_from: Option<String>,
    pub date_to: Option<String>,

    /// Filter by is_sbsk
    pub is_sbsk: Option<bool>,
}

impl SearchFilters {
    /// Create empty filters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set satker filter
    pub fn with_satker(mut self, satker_id: Uuid) -> Self {
        self.satker_id = Some(satker_id);
        self
    }

    /// Set tahun anggaran filter
    pub fn with_tahun_anggaran(mut self, tahun: i32) -> Self {
        self.tahun_anggaran = Some(tahun);
        self
    }

    /// Set status filter
    pub fn with_status(mut self, status: Vec<String>) -> Self {
        self.status = Some(status);
        self
    }

    /// Set kode barang filter
    pub fn with_kode_barang(mut self, kode: String) -> Self {
        self.kode_barang = Some(kode);
        self
    }

    /// Check if any filters are active
    pub fn has_filters(&self) -> bool {
        self.satker_id.is_some()
            || self.tahun_anggaran.is_some()
            || self.status.is_some()
            || self.kode_barang.is_some()
            || self.priority_level.is_some()
            || self.date_from.is_some()
            || self.date_to.is_some()
            || self.is_sbsk.is_some()
    }
}

/// Pagination options
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Pagination {
    pub page: i32,
    pub per_page: i32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 20,
        }
    }
}

impl Pagination {
    /// Create new pagination
    pub fn new(page: i32, per_page: i32) -> Self {
        Self { page, per_page }
    }

    /// Calculate offset for SQL queries
    pub fn offset(&self) -> i32 {
        (self.page - 1) * self.per_page
    }

    /// Calculate limit for SQL queries
    pub fn limit(&self) -> i32 {
        self.per_page
    }
}

/// Sort options
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SortOptions {
    pub field: SortField,
    pub direction: SortDirection,
}

impl Default for SortOptions {
    fn default() -> Self {
        Self {
            field: SortField::Relevance,
            direction: SortDirection::Descending,
        }
    }
}

impl SortOptions {
    /// Create new sort options
    pub fn new(field: SortField, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Get SQL ORDER BY clause
    pub fn to_sql(&self) -> String {
        let field_sql = match self.field {
            SortField::Relevance => "relevance_score",
            SortField::CreatedAt => "created_at",
            SortField::UpdatedAt => "updated_at",
            SortField::Priority => "priority_score",
            SortField::Gap => "gap",
            SortField::TahunAnggaran => "tahun_anggaran",
        };

        let direction_sql = match self.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        format!("{} {}", field_sql, direction_sql)
    }
}

/// Sort field
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SortField {
    Relevance,
    CreatedAt,
    UpdatedAt,
    Priority,
    Gap,
    TahunAnggaran,
}

impl SortField {
    /// Parse from string
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "relevance" => Some(SortField::Relevance),
            "created_at" | "created" => Some(SortField::CreatedAt),
            "updated_at" | "updated" => Some(SortField::UpdatedAt),
            "priority" => Some(SortField::Priority),
            "gap" => Some(SortField::Gap),
            "tahun_anggaran" | "tahun" => Some(SortField::TahunAnggaran),
            _ => None,
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Parse from string
    pub fn parse_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "asc" | "ascending" => Some(SortDirection::Ascending),
            "desc" | "descending" => Some(SortDirection::Descending),
            _ => None,
        }
    }
}

/// Search result with relevance score
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchResult<T> {
    pub item: T,
    pub relevance_score: f64,
    pub highlights: Vec<String>,
}

impl<T> SearchResult<T> {
    /// Create a new search result
    pub fn new(item: T, relevance_score: f64) -> Self {
        Self {
            item,
            relevance_score,
            highlights: Vec::new(),
        }
    }

    /// Add highlights
    pub fn with_highlights(mut self, highlights: Vec<String>) -> Self {
        self.highlights = highlights;
        self
    }
}

/// Paginated search results
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchResults<T> {
    pub results: Vec<SearchResult<T>>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
    pub query: String,
}

impl<T> SearchResults<T> {
    /// Create new search results
    pub fn new(
        results: Vec<SearchResult<T>>,
        total: i64,
        page: i32,
        per_page: i32,
        query: String,
    ) -> Self {
        let total_pages = if per_page > 0 {
            ((total as f64) / (per_page as f64)).ceil() as i32
        } else {
            0
        };

        Self {
            results,
            total,
            page,
            per_page,
            total_pages,
            query,
        }
    }

    /// Check if there are more pages
    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there is a previous page
    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_search_query_creation() {
        let query = SearchQuery::new("meja kerja".to_string());
        assert_eq!(query.query, "meja kerja");
        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
    }

    #[test]
    fn test_search_filters() {
        let satker_id = Uuid::new_v4();
        let filters = SearchFilters::new()
            .with_satker(satker_id)
            .with_tahun_anggaran(2024)
            .with_status(vec!["APPROVED".to_string()]);

        assert_eq!(filters.satker_id, Some(satker_id));
        assert_eq!(filters.tahun_anggaran, Some(2024));
        assert!(filters.has_filters());
    }

    #[test]
    fn test_pagination_offset() {
        let pagination = Pagination::new(1, 20);
        assert_eq!(pagination.offset(), 0);
        assert_eq!(pagination.limit(), 20);

        let pagination = Pagination::new(3, 20);
        assert_eq!(pagination.offset(), 40);
    }

    #[test]
    fn test_sort_options_sql() {
        let sort = SortOptions::new(SortField::CreatedAt, SortDirection::Descending);
        assert_eq!(sort.to_sql(), "created_at DESC");

        let sort = SortOptions::new(SortField::Priority, SortDirection::Ascending);
        assert_eq!(sort.to_sql(), "priority_score ASC");
    }

    #[test]
    fn test_sort_field_parsing() {
        assert_eq!(
            SortField::parse_str("relevance"),
            Some(SortField::Relevance)
        );
        assert_eq!(
            SortField::parse_str("created_at"),
            Some(SortField::CreatedAt)
        );
        assert_eq!(SortField::parse_str("priority"), Some(SortField::Priority));
        assert_eq!(SortField::parse_str("invalid"), None);
    }

    #[test]
    fn test_sort_direction_parsing() {
        assert_eq!(
            SortDirection::parse_str("asc"),
            Some(SortDirection::Ascending)
        );
        assert_eq!(
            SortDirection::parse_str("desc"),
            Some(SortDirection::Descending)
        );
        assert_eq!(SortDirection::parse_str("invalid"), None);
    }

    #[test]
    fn test_search_results_pagination() {
        let results: Vec<SearchResult<String>> = vec![];
        let search_results = SearchResults::new(results, 100, 1, 20, "test".to_string());

        assert_eq!(search_results.total_pages, 5);
        assert!(search_results.has_next_page());
        assert!(!search_results.has_prev_page());

        let results: Vec<SearchResult<String>> = vec![];
        let search_results = SearchResults::new(results, 100, 3, 20, "test".to_string());
        assert!(search_results.has_next_page());
        assert!(search_results.has_prev_page());
    }
}
