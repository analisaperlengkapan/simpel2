//! Database-backed full-text search engine.
//!
//! Moved out of `lib-perlengkapan` (F0-C): the SQL/`tokio-postgres` search
//! implementation belongs in the service's infra layer, keeping
//! `lib-perlengkapan` a pure WASM-safe DTO/domain crate. The WASM-safe query
//! and result DTOs (`SearchQuery`, `SearchResults`, `SearchResult`) still come
//! from [`lib_perlengkapan::search`].

use deadpool_postgres::Pool;
use tokio_postgres::Row;

use lib_perlengkapan::search::{SearchQuery, SearchResult, SearchResults};

/// Search engine with database operations (PostgreSQL Indonesian full-text).
pub struct SearchEngineDb {
    db_pool: Pool,
}

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

        if let Some(ref status_list) = query.filters.status
            && !status_list.is_empty()
        {
            sql.push_str(&format!(" AND k.status_kode = ANY(${})", param_idx));
            params.push(Box::new(status_list.clone()));
            param_idx += 1;
        }
        let _ = param_idx;

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
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

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

        if let Some(ref priority_levels) = query.filters.priority_level
            && !priority_levels.is_empty()
        {
            sql.push_str(&format!(" AND b.prioritas = ANY(${})", param_idx));
            // Convert priority levels to integers
            let priority_ints: Vec<i32> = priority_levels
                .iter()
                .filter_map(|p| p.parse().ok())
                .collect();
            params.push(Box::new(priority_ints));
            param_idx += 1;
        }
        let _ = param_idx;

        // Add sorting
        sql.push_str(&format!(" ORDER BY {}", query.sort.to_sql()));

        // Add pagination
        sql.push_str(&format!(
            " LIMIT {} OFFSET {}",
            query.pagination.limit(),
            query.pagination.offset()
        ));

        // Execute query
        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
            .iter()
            .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect();

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

        // Combine terms into a single regex to avoid loop compilation
        if !terms.is_empty() {
            let escaped_terms: Vec<String> = terms.iter().map(|t| regex::escape(t)).collect();
            let pattern_str = format!("(?i)({})", escaped_terms.join("|"));
            let pattern =
                regex::Regex::new(&pattern_str).unwrap_or_else(|_| regex::Regex::new("").unwrap());

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
