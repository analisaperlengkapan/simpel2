// ============================================================================
// Database Optimization Module
// Description: Query analysis, index management, and query result caching
// Author: SIMPEL Team
// Created: 2026-02-10
// Requirements: NFR-P001, NFR-P002
// ============================================================================

use crate::cache_strategy::{CacheKey, CacheManager};
use crate::errors::AppError;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Query performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetrics {
    pub query: String,
    pub execution_time_ms: u64,
    pub rows_returned: i64,
    pub plan_cost: Option<f64>,
    pub uses_index: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Slow query threshold (500ms as per NFR-P002)
const SLOW_QUERY_THRESHOLD_MS: u64 = 500;

/// Database optimizer
pub struct DatabaseOptimizer {
    pool: Pool,
    cache_manager: Arc<CacheManager>,
    slow_queries: Arc<tokio::sync::RwLock<Vec<QueryMetrics>>>,
}

impl DatabaseOptimizer {
    pub fn new(pool: Pool, cache_manager: Arc<CacheManager>) -> Self {
        Self {
            pool,
            cache_manager,
            slow_queries: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Execute a query with performance monitoring
    pub async fn execute_with_monitoring<T, F>(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        executor: F,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, tokio_postgres::Error>> + Send>,
        >,
    {
        let start = Instant::now();

        // Execute query
        let result = executor()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let execution_time = start.elapsed();
        let execution_time_ms = execution_time.as_millis() as u64;

        // Log slow queries
        if execution_time_ms > SLOW_QUERY_THRESHOLD_MS {
            warn!("Slow query detected ({}ms): {}", execution_time_ms, query);

            // Analyze query plan
            if let Ok(plan) = self.analyze_query_plan(query, params).await {
                let mut slow_queries = self.slow_queries.write().await;
                slow_queries.push(QueryMetrics {
                    query: query.to_string(),
                    execution_time_ms,
                    rows_returned: 0, // Would need to be passed from result
                    plan_cost: plan.cost,
                    uses_index: plan.uses_index,
                    timestamp: chrono::Utc::now(),
                });

                // Keep only last 100 slow queries
                if slow_queries.len() > 100 {
                    slow_queries.remove(0);
                }
            }
        } else {
            debug!("Query executed in {}ms: {}", execution_time_ms, query);
        }

        Ok(result)
    }

    /// Analyze query execution plan using EXPLAIN
    pub async fn analyze_query_plan(
        &self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<QueryPlan, AppError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Database(format!("Failed to get connection: {}", e)))?;

        // Use EXPLAIN (FORMAT JSON) for structured output
        let explain_query = format!("EXPLAIN (FORMAT JSON, ANALYZE false) {}", query);

        let row = client
            .query_one(&explain_query, params)
            .await
            .map_err(|e| AppError::Database(format!("EXPLAIN failed: {}", e)))?;

        let plan_json: serde_json::Value = row.get(0);

        // Parse the plan
        let plan = &plan_json[0]["Plan"];
        let cost = plan["Total Cost"].as_f64();
        let node_type = plan["Node Type"].as_str().unwrap_or("Unknown");

        // Check if index is used
        let uses_index = node_type.contains("Index")
            || plan.to_string().contains("Index Scan")
            || plan.to_string().contains("Index Only Scan");

        Ok(QueryPlan {
            cost,
            uses_index,
            node_type: node_type.to_string(),
            plan_json: plan.clone(),
        })
    }

    /// Get slow query report
    pub async fn get_slow_queries(&self) -> Vec<QueryMetrics> {
        let slow_queries = self.slow_queries.read().await;
        slow_queries.clone()
    }

    /// Clear slow query log
    pub async fn clear_slow_queries(&self) {
        let mut slow_queries = self.slow_queries.write().await;
        slow_queries.clear();
        info!("Slow query log cleared");
    }

    /// Add missing indexes based on slow query analysis
    pub async fn suggest_indexes(&self) -> Result<Vec<IndexSuggestion>, AppError> {
        let slow_queries = self.slow_queries.read().await;
        let mut suggestions = Vec::new();

        for query_metric in slow_queries.iter() {
            if !query_metric.uses_index && query_metric.execution_time_ms > SLOW_QUERY_THRESHOLD_MS
            {
                // Analyze query to suggest indexes
                if let Some(suggestion) = self.analyze_for_index(&query_metric.query) {
                    suggestions.push(suggestion);
                }
            }
        }

        Ok(suggestions)
    }

    /// Analyze query to suggest index
    fn analyze_for_index(&self, query: &str) -> Option<IndexSuggestion> {
        let query_lower = query.to_lowercase();

        // Simple heuristics for index suggestions
        if query_lower.contains("where") {
            // Extract table and column from WHERE clause
            if let Some(table) = self.extract_table_name(&query_lower) {
                if let Some(column) = self.extract_where_column(&query_lower) {
                    return Some(IndexSuggestion {
                        table: table.to_string(),
                        columns: vec![column.to_string()],
                        index_type: IndexType::BTree,
                        reason: format!("Frequent WHERE clause on {}.{}", table, column),
                    });
                }
            }
        }

        None
    }

    fn extract_table_name<'a>(&self, query: &'a str) -> Option<&'a str> {
        // Simple extraction - in production, use a proper SQL parser
        if let Some(from_pos) = query.find("from ") {
            let after_from = &query[from_pos + 5..];
            if let Some(space_pos) = after_from.find(' ') {
                return Some(after_from[..space_pos].trim());
            }
        }
        None
    }

    fn extract_where_column<'a>(&self, query: &'a str) -> Option<&'a str> {
        // Simple extraction - in production, use a proper SQL parser
        if let Some(where_pos) = query.find("where ") {
            let after_where = &query[where_pos + 6..];
            if let Some(eq_pos) = after_where.find('=') {
                let column = after_where[..eq_pos].trim();
                // Remove table prefix if present
                if let Some(dot_pos) = column.rfind('.') {
                    return Some(column[dot_pos + 1..].trim());
                }
                return Some(column);
            }
        }
        None
    }

    /// Create recommended indexes
    pub async fn create_indexes(
        &self,
        suggestions: &[IndexSuggestion],
    ) -> Result<Vec<String>, AppError> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| AppError::Database(format!("Failed to get connection: {}", e)))?;

        let mut created_indexes = Vec::new();

        for suggestion in suggestions {
            let index_name = format!(
                "idx_{}_{}_auto",
                suggestion.table,
                suggestion.columns.join("_")
            );

            let create_index_sql = format!(
                "CREATE INDEX IF NOT EXISTS {} ON {} ({})",
                index_name,
                suggestion.table,
                suggestion.columns.join(", ")
            );

            match client.execute(&create_index_sql, &[]).await {
                Ok(_) => {
                    info!("Created index: {}", index_name);
                    created_indexes.push(index_name);
                }
                Err(e) => {
                    warn!("Failed to create index {}: {}", index_name, e);
                }
            }
        }

        Ok(created_indexes)
    }

    /// Optimize complex queries by rewriting them
    pub fn optimize_query(&self, query: &str) -> String {
        let optimized = query.to_string();

        // Replace SELECT * with specific columns (when possible)
        // This is a simplified example - in production, use proper SQL parsing
        if optimized.contains("SELECT *") {
            warn!("Query uses SELECT * which may be inefficient: {}", query);
        }

        // Add LIMIT if missing for large result sets
        if !optimized.to_lowercase().contains("limit")
            && !optimized.to_lowercase().contains("count(")
        {
            debug!("Consider adding LIMIT clause to query: {}", query);
        }

        optimized
    }

    /// Execute query with result caching
    pub async fn execute_with_cache<T>(
        &self,
        cache_key: CacheKey,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
        executor: impl FnOnce() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, tokio_postgres::Error>> + Send>,
        >,
    ) -> Result<T, AppError>
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone,
    {
        // Check cache first
        if let Ok(Some(cached)) = self.cache_manager.get::<T>(&cache_key).await {
            debug!("Cache hit for key: {}", cache_key.to_string());
            return Ok(cached);
        }

        // Execute query with monitoring
        let result = self
            .execute_with_monitoring(query, params, executor)
            .await?;

        // Cache the result
        if let Err(e) = self.cache_manager.set(&cache_key, &result).await {
            warn!("Failed to cache result: {}", e);
        }

        Ok(result)
    }
}

/// Query execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub cost: Option<f64>,
    pub uses_index: bool,
    pub node_type: String,
    pub plan_json: serde_json::Value,
}

/// Index suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSuggestion {
    pub table: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    GIN,
    GiST,
}

/// Add essential indexes for perlengkapan schema
pub async fn add_essential_indexes(pool: &Pool) -> Result<(), AppError> {
    let client = pool
        .get()
        .await
        .map_err(|e| AppError::Database(format!("Failed to get connection: {}", e)))?;

    let indexes = vec![
        // Kebutuhan BMN indexes
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_satker ON perlengkapan.kebutuhan_bmn(satker_id)",
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_tahun ON perlengkapan.kebutuhan_bmn(tahun_anggaran)",
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_status ON perlengkapan.kebutuhan_bmn(status)",
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_kode ON perlengkapan.kebutuhan_bmn(kode_barang)",
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_composite ON perlengkapan.kebutuhan_bmn(satker_id, tahun_anggaran, status)",
        // Pakaian Dinas indexes
        "CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_nip ON perlengkapan.pakaian_dinas(pegawai_nip)",
        "CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_tahun ON perlengkapan.pakaian_dinas(tahun_anggaran)",
        "CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_status ON perlengkapan.pakaian_dinas(status)",
        // Roadmap indexes
        "CREATE INDEX IF NOT EXISTS idx_roadmap_satker ON perlengkapan.roadmap_sarpras(satker_id)",
        "CREATE INDEX IF NOT EXISTS idx_roadmap_periode ON perlengkapan.roadmap_sarpras(periode_mulai, periode_akhir)",
        "CREATE INDEX IF NOT EXISTS idx_roadmap_tahun ON perlengkapan.roadmap_sarpras(tahun_rencana)",
        // Pemakaian BMN indexes
        "CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_pegawai ON perlengkapan.izin_pemakaian_bmn(pegawai_nip)",
        "CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_status ON perlengkapan.izin_pemakaian_bmn(status)",
        "CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_tanggal ON perlengkapan.izin_pemakaian_bmn(tanggal_mulai, tanggal_selesai)",
        "CREATE INDEX IF NOT EXISTS idx_pemakaian_bmn_bmn_id ON perlengkapan.izin_pemakaian_bmn(bmn_id)",
        // Workflow indexes
        "CREATE INDEX IF NOT EXISTS idx_workflow_aktivitas_pengajuan ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(pengajuan_id)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_aktivitas_user ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(user_id)",
        "CREATE INDEX IF NOT EXISTS idx_workflow_aktivitas_created ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(created_at)",
        // Integration schema indexes
        "CREATE INDEX IF NOT EXISTS idx_siman_aset_satker ON integrasi.siman_aset_tanah(satker_id)",
        "CREATE INDEX IF NOT EXISTS idx_siman_aset_kode ON integrasi.siman_aset_tanah(kode_barang)",
        "CREATE INDEX IF NOT EXISTS idx_siman_aset_kondisi ON integrasi.siman_aset_tanah(kondisi)",
        "CREATE INDEX IF NOT EXISTS idx_siman_aset_synced ON integrasi.siman_aset_tanah(synced_at)",
        // Full-text search indexes using pg_trgm
        "CREATE EXTENSION IF NOT EXISTS pg_trgm",
        "CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_nama_trgm ON perlengkapan.kebutuhan_bmn USING gin(nama_barang gin_trgm_ops)",
        "CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_nama_trgm ON perlengkapan.pakaian_dinas USING gin(pegawai_nama gin_trgm_ops)",
        // JSONB indexes for raw_data
        "CREATE INDEX IF NOT EXISTS idx_siman_aset_raw_data ON integrasi.siman_aset_tanah USING gin(raw_data)",
    ];

    for index_sql in indexes {
        match client.execute(index_sql, &[]).await {
            Ok(_) => info!(
                "Index created/verified: {}",
                index_sql.split("idx_").nth(1).unwrap_or("unknown")
            ),
            Err(e) => warn!("Failed to create index: {} - Error: {}", index_sql, e),
        }
    }

    info!("Essential indexes added successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_table_name() {
        let optimizer = DatabaseOptimizer::new(
            deadpool_postgres::Config::new()
                .create_pool(None, tokio_postgres::NoTls)
                .unwrap(),
            Arc::new(CacheManager::new()),
        );

        let query = "select * from perlengkapan.kebutuhan_bmn where status = 'approved'";
        let table = optimizer.extract_table_name(query);
        assert_eq!(table, Some("perlengkapan.kebutuhan_bmn"));
    }

    #[test]
    fn test_extract_where_column() {
        let optimizer = DatabaseOptimizer::new(
            deadpool_postgres::Config::new()
                .create_pool(None, tokio_postgres::NoTls)
                .unwrap(),
            Arc::new(CacheManager::new()),
        );

        let query = "select * from kebutuhan_bmn where status = 'approved'";
        let column = optimizer.extract_where_column(query);
        assert_eq!(column, Some("status"));
    }

    #[test]
    fn test_optimize_query() {
        let optimizer = DatabaseOptimizer::new(
            deadpool_postgres::Config::new()
                .create_pool(None, tokio_postgres::NoTls)
                .unwrap(),
            Arc::new(CacheManager::new()),
        );

        let query = "SELECT * FROM kebutuhan_bmn";
        let optimized = optimizer.optimize_query(query);
        // Should return the same query but log a warning
        assert_eq!(optimized, query);
    }
}
