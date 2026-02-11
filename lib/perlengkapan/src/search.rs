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
    pub fn from_str(s: &str) -> Option<Self> {
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
    pub fn from_str(s: &str) -> Option<Self> {
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

/// Search engine (backend only)
#[cfg(feature = "backend")]
pub struct SearchEngine;

#[cfg(feature = "backend")]
impl SearchEngine {
    /// Build full-text search SQL query
    ///
    /// Uses PostgreSQL's pg_trgm extension for fuzzy matching
    pub fn build_search_sql(
        table: &str,
        search_fields: &[&str],
        query: &SearchQuery,
    ) -> (String, Vec<String>) {
        let mut sql = format!("SELECT *, ");

        // Add relevance score calculation
        let relevance_parts: Vec<String> = search_fields
            .iter()
            .map(|field| {
                format!(
                    "similarity({}, $1)",
                    field
                )
            })
            .collect();

        sql.push_str(&format!(
            "({}) as relevance_score FROM {}",
            relevance_parts.join(" + "),
            table
        ));

        // Add WHERE clause for search
        sql.push_str(" WHERE (");
        let search_conditions: Vec<String> = search_fields
            .iter()
            .map(|field| format!("{} % $1", field))
            .collect();
        sql.push_str(&search_conditions.join(" OR "));
        sql.push(')');

        let mut params = vec![query.query.clone()];
        let mut param_index = 2;

        // Add filters
        if let Some(satker_id) = query.filters.satker_id {
            sql.push_str(&format!(" AND satker_id = ${}", param_index));
            params.push(satker_id.to_string());
            param_index += 1;
        }

        if let Some(tahun) = query.filters.tahun_anggaran {
            sql.push_str(&format!(" AND tahun_anggaran = ${}", param_index));
            params.push(tahun.to_string());
            param_index += 1;
        }

        if let Some(ref status_list) = query.filters.status {
            if !status_list.is_empty() {
                sql.push_str(&format!(" AND status = ANY(${})", param_index));
                params.push(format!("{{{}}}", status_list.join(",")));
                param_index += 1;
            }
        }

        if let Some(ref kode) = query.filters.kode_barang {
            sql.push_str(&format!(" AND kode_barang LIKE ${}", param_index));
            params.push(format!("%{}%", kode));
            param_index += 1;
        }

        if let Some(is_sbsk) = query.filters.is_sbsk {
            sql.push_str(&format!(" AND is_sbsk = ${}", param_index));
            params.push(is_sbsk.to_string());
            param_index += 1;
        }

        // Add ORDER BY
        sql.push_str(&format!(" ORDER BY {}", query.sort.to_sql()));

        // Add LIMIT and OFFSET
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            query.pagination.limit(),
            query.pagination.offset()
        ));

        (sql, params)
    }

    /// Calculate relevance score for a search result
    ///
    /// Uses Levenshtein distance for fuzzy matching
    pub fn calculate_relevance(search_term: &str, text: &str) -> f64 {
        let search_lower = search_term.to_lowercase();
        let text_lower = text.to_lowercase();

        // Exact match
        if text_lower.contains(&search_lower) {
            return 1.0;
        }

        // Calculate similarity (simple implementation)
        let max_len = search_term.len().max(text.len()) as f64;
        let distance = levenshtein_distance(&search_lower, &text_lower) as f64;

        (max_len - distance) / max_len
    }
}

/// Calculate Levenshtein distance between two strings
#[allow(dead_code)]
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
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
        assert_eq!(SortField::from_str("relevance"), Some(SortField::Relevance));
        assert_eq!(SortField::from_str("created_at"), Some(SortField::CreatedAt));
        assert_eq!(SortField::from_str("priority"), Some(SortField::Priority));
        assert_eq!(SortField::from_str("invalid"), None);
    }

    #[test]
    fn test_sort_direction_parsing() {
        assert_eq!(
            SortDirection::from_str("asc"),
            Some(SortDirection::Ascending)
        );
        assert_eq!(
            SortDirection::from_str("desc"),
            Some(SortDirection::Descending)
        );
        assert_eq!(SortDirection::from_str("invalid"), None);
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

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn test_calculate_relevance() {
        let score = SearchEngine::calculate_relevance("meja", "meja kerja");
        assert!(score > 0.5);

        let score = SearchEngine::calculate_relevance("meja", "meja");
        assert_eq!(score, 1.0);

        let score = SearchEngine::calculate_relevance("meja", "kursi");
        assert!(score < 0.5);
    }
}


// ============================================================================
// BACKEND IMPLEMENTATION (with database operations)
// ============================================================================

#[cfg(feature = "backend")]
use deadpool_postgres::Pool;

#[cfg(feature = "backend")]
use tokio_postgres::Row;

/// Search engine with database operations
#[cfg(feature = "backend")]
pub struct SearchEngineDb {
    db_pool: Pool,
}

#[cfg(feature = "backend")]
impl SearchEngineDb {
    /// Create a new search engine with database pool
    pub fn new(db_pool: Pool) -> Self {
        Self { db_pool }
    }

    /// Search kebutuhan BMN with full-text search and filters
    ///
    /// Uses PostgreSQL's Indonesian text search configuration
    pub async fn search_kebutuhan<T>(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResults<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: for<'a> TryFrom<&'a Row, Error = Box<dyn std::error::Error + Send + Sync>>,
    {
        let client = self.db_pool.get().await?;

        // Build the SQL query
        let mut sql = String::from(
            r#"
            SELECT
                k.*,
                ts_rank(
                    to_tsvector('indonesian', k.nama || ' ' || COALESCE(k.deskripsi, '')),
                    plainto_tsquery('indonesian', $1)
                ) as relevance_score
            FROM perlengkapan.pengajuan_kebutuhan_bmn k
            WHERE to_tsvector('indonesian', k.nama || ' ' || COALESCE(k.deskripsi, ''))
                  @@ plainto_tsquery('indonesian', $1)
            "#,
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(query.query.clone())];
        let mut param_idx = 2;

        // Add filters
        if let Some(satker_id) = query.filters.satker_id {
            sql.push_str(&format!(" AND k.satker_id = ${}", param_idx));
            params.push(Box::new(satker_id));
            param_idx += 1;
        }

        if let Some(tahun) = query.filters.tahun_anggaran {
            sql.push_str(&format!(" AND k.tahun = ${}", param_idx));
            params.push(Box::new(tahun));
            param_idx += 1;
        }

        if let Some(ref status_list) = query.filters.status {
            if !status_list.is_empty() {
                sql.push_str(&format!(" AND k.status_kode = ANY(${})", param_idx));
                params.push(Box::new(status_list.clone()));
                param_idx += 1;
            }
        }

        if let Some(ref kode) = query.filters.kode_barang {
            sql.push_str(&format!(" AND k.kode_barang ILIKE ${}", param_idx));
            params.push(Box::new(format!("%{}%", kode)));
            param_idx += 1;
        }

        if let Some(is_sbsk) = query.filters.is_sbsk {
            sql.push_str(&format!(" AND k.is_sbsk = ${}", param_idx));
            params.push(Box::new(is_sbsk));
            param_idx += 1;
        }

        if let Some(ref date_from) = query.filters.date_from {
            sql.push_str(&format!(" AND k.created_at >= ${}", param_idx));
            params.push(Box::new(date_from.clone()));
            param_idx += 1;
        }

        if let Some(ref date_to) = query.filters.date_to {
            sql.push_str(&format!(" AND k.created_at <= ${}", param_idx));
            params.push(Box::new(date_to.clone()));
            param_idx += 1;
        }

        // Add sorting
        sql.push_str(&format!(" ORDER BY {}", query.sort.to_sql()));

        // Add pagination
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            query.pagination.limit(),
            query.pagination.offset()
        ));

        // Execute query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let rows = client.query(&sql, &param_refs).await?;

        // Convert rows to search results
        let mut results = Vec::new();
        for row in rows {
            let relevance_score: f32 = row.try_get("relevance_score").unwrap_or(0.0);
            let item = T::try_from(&row)?;
            results.push(SearchResult::new(item, relevance_score as f64));
        }

        // Get total count (without pagination)
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) as subquery",
            sql.split("LIMIT").next().unwrap()
        );

        let count_row = client.query_one(&count_sql, &param_refs).await?;
        let total: i64 = count_row.get(0);

        Ok(SearchResults::new(
            results,
            total,
            query.pagination.page,
            query.pagination.per_page,
            query.query,
        ))
    }

    /// Search kebutuhan BMN barang (items within kebutuhan)
    pub async fn search_kebutuhan_barang<T>(
        &self,
        query: SearchQuery,
    ) -> Result<SearchResults<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        T: for<'a> TryFrom<&'a Row, Error = Box<dyn std::error::Error + Send + Sync>>,
    {
        let client = self.db_pool.get().await?;

        let mut sql = String::from(
            r#"
            SELECT
                b.*,
                ts_rank(
                    to_tsvector('indonesian', b.nama || ' ' || COALESCE(b.keterangan, '')),
                    plainto_tsquery('indonesian', $1)
                ) as relevance_score
            FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang b
            WHERE to_tsvector('indonesian', b.nama || ' ' || COALESCE(b.keterangan, ''))
                  @@ plainto_tsquery('indonesian', $1)
            "#,
        );

        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> =
            vec![Box::new(query.query.clone())];
        let mut param_idx = 2;

        // Add filters
        if let Some(ref kode) = query.filters.kode_barang {
            sql.push_str(&format!(" AND b.kode_barang ILIKE ${}", param_idx));
            params.push(Box::new(format!("%{}%", kode)));
            param_idx += 1;
        }

        if let Some(ref priority_levels) = query.filters.priority_level {
            if !priority_levels.is_empty() {
                sql.push_str(&format!(" AND b.prioritas = ANY(${})", param_idx));
                // Convert priority levels to integers
                let priority_ints: Vec<i32> = priority_levels
                    .iter()
                    .filter_map(|p| p.parse().ok())
                    .collect();
                params.push(Box::new(priority_ints));
                param_idx += 1;
            }
        }

        // Add sorting
        sql.push_str(&format!(" ORDER BY {}", query.sort.to_sql()));

        // Add pagination
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            query.pagination.limit(),
            query.pagination.offset()
        ));

        // Execute query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let rows = client.query(&sql, &param_refs).await?;

        // Convert rows to search results
        let mut results = Vec::new();
        for row in rows {
            let relevance_score: f32 = row.try_get("relevance_score").unwrap_or(0.0);
            let item = T::try_from(&row)?;
            results.push(SearchResult::new(item, relevance_score as f64));
        }

        // Get total count
        let count_sql = format!(
            "SELECT COUNT(*) FROM ({}) as subquery",
            sql.split("LIMIT").next().unwrap()
        );

        let count_row = client.query_one(&count_sql, &param_refs).await?;
        let total: i64 = count_row.get(0);

        Ok(SearchResults::new(
            results,
            total,
            query.pagination.page,
            query.pagination.per_page,
            query.query,
        ))
    }

    /// Highlight search terms in text
    ///
    /// Wraps matching terms with <mark> tags for frontend display
    pub fn highlight_terms(&self, text: &str, search_query: &str) -> String {
        let terms: Vec<&str> = search_query.split_whitespace().collect();
        let mut highlighted = text.to_string();

        for term in terms {
            // Case-insensitive replacement
            let pattern = regex::Regex::new(&format!("(?i)({})", regex::escape(term)))
                .unwrap_or_else(|_| regex::Regex::new("").unwrap());
            highlighted = pattern
                .replace_all(&highlighted, "<mark>$1</mark>")
                .to_string();
        }

        highlighted
    }

    /// Get search suggestions based on partial query
    ///
    /// Returns top 10 most relevant suggestions
    pub async fn get_suggestions(
        &self,
        partial_query: &str,
        limit: i32,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let client = self.db_pool.get().await?;

        let sql = r#"
            SELECT DISTINCT nama
            FROM perlengkapan.pengajuan_kebutuhan_bmn
            WHERE nama ILIKE $1
            ORDER BY similarity(nama, $2) DESC
            LIMIT $3
        "#;

        let pattern = format!("%{}%", partial_query);
        let rows = client
            .query(sql, &[&pattern, &partial_query, &limit])
            .await?;

        let suggestions: Vec<String> = rows.iter().map(|row| row.get("nama")).collect();

        Ok(suggestions)
    }
}

#[cfg(all(test, feature = "backend"))]
mod backend_tests {
    use super::*;

    // Note: These tests require a running PostgreSQL database
    // Run with: cargo test --features backend

    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored flag
    async fn test_search_kebutuhan_integration() {
        // This would require a test database setup
        // Implementation depends on test infrastructure
    }

    // Commented out until proper mock infrastructure is available
    // #[test]
    // fn test_highlight_terms() {
    //     let engine = SearchEngineDb {
    //         db_pool: todo!(), // Mock pool for testing
    //     };
    //
    //     let text = "Meja kerja kayu jati untuk kantor";
    //     let highlighted = engine.highlight_terms(text, "meja kerja");
    //
    //     assert!(highlighted.contains("<mark>"));
    //     assert!(highlighted.contains("</mark>"));
    // }
}
