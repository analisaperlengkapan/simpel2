// ============================================================================
// Cache Strategy Implementation
// Description: Caching strategy for perlengkapan service with TTL and invalidation
// Author: SIMPEL Team
// Created: 2026-02-10
// Requirements: NFR-P006
// ============================================================================

use lib_backend::cache::{AsyncLruCache, SensitivityLevel};
use lib_core::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Cache keys for different data types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CacheKey {
    /// Reference data (master data)
    /// Format: "ref:{table}:{id}"
    ReferenceData { table: String, id: String },

    /// Gap analysis results
    /// Format: "gap:{satker_id}:{kode_barang}"
    GapAnalysis {
        satker_id: Uuid,
        kode_barang: String,
    },

    /// Dashboard metrics
    /// Format: "dashboard:{type}:{tahun}:{filter}"
    DashboardMetrics {
        metric_type: String,
        tahun: i32,
        filter: String,
    },

    /// Kebutuhan BMN list
    /// Format: "kebutuhan:list:{tahun}:{status}:{page}"
    KebutuhanList {
        tahun: i32,
        status: Option<String>,
        page: i32,
    },

    /// Pakaian Dinas list
    /// Format: "pakaian:list:{tahun}:{status}:{page}"
    PakaianList {
        tahun: i32,
        status: Option<String>,
        page: i32,
    },

    /// Roadmap data
    /// Format: "roadmap:{satker_id}:{periode}"
    Roadmap {
        satker_id: Uuid,
        periode_mulai: i32,
        periode_akhir: i32,
    },

    /// SIMAN asset count
    /// Format: "siman:count:{satker_code}:{kode_barang}:{kondisi}"
    SimanAssetCount {
        satker_code: String,
        kode_barang: Option<String>,
        kondisi: Option<String>,
    },

    /// MySIMKARI employee count
    /// Format: "mysimkari:count:{satker_code}:{status}"
    MySIMKARIEmployeeCount {
        satker_code: String,
        status: Option<String>,
    },
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CacheKey::ReferenceData { table, id } => format!("ref:{}:{}", table, id),
            CacheKey::GapAnalysis {
                satker_id,
                kode_barang,
            } => format!("gap:{}:{}", satker_id, kode_barang),
            CacheKey::DashboardMetrics {
                metric_type,
                tahun,
                filter,
            } => format!("dashboard:{}:{}:{}", metric_type, tahun, filter),
            CacheKey::KebutuhanList {
                tahun,
                status,
                page,
            } => format!(
                "kebutuhan:list:{}:{}:{}",
                tahun,
                status.as_deref().unwrap_or("all"),
                page
            ),
            CacheKey::PakaianList {
                tahun,
                status,
                page,
            } => format!(
                "pakaian:list:{}:{}:{}",
                tahun,
                status.as_deref().unwrap_or("all"),
                page
            ),
            CacheKey::Roadmap {
                satker_id,
                periode_mulai,
                periode_akhir,
            } => format!("roadmap:{}:{}-{}", satker_id, periode_mulai, periode_akhir),
            CacheKey::SimanAssetCount {
                satker_code,
                kode_barang,
                kondisi,
            } => format!(
                "siman:count:{}:{}:{}",
                satker_code,
                kode_barang.as_deref().unwrap_or("all"),
                kondisi.as_deref().unwrap_or("all")
            ),
            CacheKey::MySIMKARIEmployeeCount {
                satker_code,
                status,
            } => format!(
                "mysimkari:count:{}:{}",
                satker_code,
                status.as_deref().unwrap_or("all")
            ),
        };
        write!(f, "{s}")
    }
}

/// Cache configuration with TTL settings
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Reference data TTL (1 hour)
    pub reference_data_ttl: Duration,

    /// Gap analysis TTL (1 hour)
    pub gap_analysis_ttl: Duration,

    /// Dashboard metrics TTL (5 minutes)
    pub dashboard_metrics_ttl: Duration,

    /// List queries TTL (15 minutes)
    pub list_queries_ttl: Duration,

    /// Integration data TTL (30 minutes)
    pub integration_data_ttl: Duration,

    /// Cache capacity (number of entries)
    pub capacity: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            reference_data_ttl: Duration::from_secs(3600), // 1 hour
            gap_analysis_ttl: Duration::from_secs(3600),   // 1 hour
            dashboard_metrics_ttl: Duration::from_secs(300), // 5 minutes
            list_queries_ttl: Duration::from_secs(900),    // 15 minutes
            integration_data_ttl: Duration::from_secs(1800), // 30 minutes
            capacity: 10000,                               // 10k entries
        }
    }
}

/// Cache manager for perlengkapan service
pub struct CacheManager {
    cache: AsyncLruCache<String, Vec<u8>>,
    config: CacheConfig,
}

impl CacheManager {
    /// Create a new cache manager with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new cache manager with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            cache: AsyncLruCache::new(config.capacity),
            config,
        }
    }

    /// Get TTL and sensitivity for a cache key
    fn get_ttl_and_sensitivity(&self, key: &CacheKey) -> (Duration, SensitivityLevel) {
        match key {
            CacheKey::ReferenceData { .. } => {
                (self.config.reference_data_ttl, SensitivityLevel::Low)
            }
            CacheKey::GapAnalysis { .. } => {
                (self.config.gap_analysis_ttl, SensitivityLevel::Medium)
            }
            CacheKey::DashboardMetrics { .. } => {
                (self.config.dashboard_metrics_ttl, SensitivityLevel::High)
            }
            CacheKey::KebutuhanList { .. } | CacheKey::PakaianList { .. } => {
                (self.config.list_queries_ttl, SensitivityLevel::Medium)
            }
            CacheKey::Roadmap { .. } => (self.config.reference_data_ttl, SensitivityLevel::Low),
            CacheKey::SimanAssetCount { .. } | CacheKey::MySIMKARIEmployeeCount { .. } => {
                (self.config.integration_data_ttl, SensitivityLevel::Medium)
            }
        }
    }

    /// Get a value from cache
    pub async fn get<T>(&self, key: &CacheKey) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key_str = key.to_string();

        if let Some(bytes) = self.cache.get(&key_str).await {
            match serde_json::from_slice::<T>(&bytes) {
                Ok(value) => Ok(Some(value)),
                Err(e) => {
                    tracing::warn!(
                        "Failed to deserialize cached value for key {}: {}",
                        key_str,
                        e
                    );
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Set a value in cache
    pub async fn set<T>(&self, key: &CacheKey, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let key_str = key.to_string();
        let (ttl, sensitivity) = self.get_ttl_and_sensitivity(key);

        let bytes = serde_json::to_vec(value).map_err(|e| {
            lib_core::error::CommonError::Cache(format!("Failed to serialize value: {}", e))
        })?;

        self.cache
            .insert_with_sensitivity(key_str, bytes, ttl, sensitivity)
            .await;

        Ok(())
    }

    /// Remove a value from cache
    pub async fn remove(&self, key: &CacheKey) -> Result<()> {
        let key_str = key.to_string();
        self.cache.remove(&key_str).await;
        Ok(())
    }

    /// Invalidate cache entries by pattern
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<()> {
        // Note: This is a simplified implementation
        // In production, you might want to use Redis SCAN for pattern matching
        tracing::info!("Cache invalidation requested for pattern: {}", pattern);

        // For now, we'll clear the entire cache if pattern matching is needed
        // A more sophisticated implementation would track keys and match patterns
        if pattern.contains('*') {
            tracing::warn!("Pattern matching not fully implemented, clearing entire cache");
            self.cache.clear().await;
        }

        Ok(())
    }

    /// Invalidate all kebutuhan BMN related caches
    pub async fn invalidate_kebutuhan(&self, tahun: Option<i32>) -> Result<()> {
        if let Some(tahun) = tahun {
            self.invalidate_pattern(&format!("kebutuhan:*:{}:*", tahun))
                .await?;
            self.invalidate_pattern(&format!("dashboard:*:{}:*", tahun))
                .await?;
        } else {
            self.invalidate_pattern("kebutuhan:*").await?;
            self.invalidate_pattern("dashboard:*").await?;
        }
        Ok(())
    }

    /// Invalidate all pakaian dinas related caches
    pub async fn invalidate_pakaian(&self, tahun: Option<i32>) -> Result<()> {
        if let Some(tahun) = tahun {
            self.invalidate_pattern(&format!("pakaian:*:{}:*", tahun))
                .await?;
            self.invalidate_pattern(&format!("dashboard:*:{}:*", tahun))
                .await?;
        } else {
            self.invalidate_pattern("pakaian:*").await?;
            self.invalidate_pattern("dashboard:*").await?;
        }
        Ok(())
    }

    /// Invalidate gap analysis cache for a specific satker
    pub async fn invalidate_gap_analysis(&self, satker_id: Option<Uuid>) -> Result<()> {
        if let Some(satker_id) = satker_id {
            self.invalidate_pattern(&format!("gap:{}:*", satker_id))
                .await?;
        } else {
            self.invalidate_pattern("gap:*").await?;
        }
        Ok(())
    }

    /// Invalidate roadmap cache for a specific satker
    pub async fn invalidate_roadmap(&self, satker_id: Option<Uuid>) -> Result<()> {
        if let Some(satker_id) = satker_id {
            self.invalidate_pattern(&format!("roadmap:{}:*", satker_id))
                .await?;
        } else {
            self.invalidate_pattern("roadmap:*").await?;
        }
        Ok(())
    }

    /// Invalidate all dashboard metrics
    pub async fn invalidate_dashboard(&self) -> Result<()> {
        self.invalidate_pattern("dashboard:*").await
    }

    /// Invalidate reference data cache
    pub async fn invalidate_reference_data(&self, table: &str) -> Result<()> {
        self.invalidate_pattern(&format!("ref:{}:*", table)).await
    }

    /// Clear all cache entries
    pub async fn clear_all(&self) -> Result<()> {
        self.cache.clear().await;
        tracing::info!("All cache entries cleared");
        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> lib_backend::cache::CacheStats {
        self.cache.stats().await
    }

    /// Health check for cache connectivity
    pub async fn health_check(&self) -> Result<()> {
        // Try to perform a simple cache operation
        let test_key = CacheKey::ReferenceData {
            table: "health_check".to_string(),
            id: "test".to_string(),
        };

        // Try to set and get a value
        self.set(&test_key, &"health_check_value").await?;
        let _value: Option<String> = self.get(&test_key).await?;
        self.remove(&test_key).await?;

        Ok(())
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            config: self.config.clone(),
        }
    }
}

/// Cache middleware for automatic invalidation on data changes
pub struct CacheInvalidationMiddleware {
    cache_manager: Arc<CacheManager>,
}

impl CacheInvalidationMiddleware {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self { cache_manager }
    }

    /// Invalidate cache after kebutuhan BMN create/update/delete
    pub async fn on_kebutuhan_changed(&self, tahun: i32) -> Result<()> {
        self.cache_manager.invalidate_kebutuhan(Some(tahun)).await?;
        self.cache_manager.invalidate_gap_analysis(None).await?;
        self.cache_manager.invalidate_dashboard().await?;
        Ok(())
    }

    /// Invalidate cache after pakaian dinas create/update/delete
    pub async fn on_pakaian_changed(&self, tahun: i32) -> Result<()> {
        self.cache_manager.invalidate_pakaian(Some(tahun)).await?;
        self.cache_manager.invalidate_dashboard().await?;
        Ok(())
    }

    /// Invalidate cache after roadmap create/update/delete
    pub async fn on_roadmap_changed(&self, satker_id: Uuid) -> Result<()> {
        self.cache_manager
            .invalidate_roadmap(Some(satker_id))
            .await?;
        self.cache_manager.invalidate_dashboard().await?;
        Ok(())
    }

    /// Invalidate cache after reference data changes
    pub async fn on_reference_data_changed(&self, table: &str) -> Result<()> {
        self.cache_manager.invalidate_reference_data(table).await?;
        Ok(())
    }

    /// Invalidate cache after SIMAN sync
    pub async fn on_siman_sync_completed(&self) -> Result<()> {
        self.cache_manager.invalidate_gap_analysis(None).await?;
        self.cache_manager.invalidate_dashboard().await?;
        self.cache_manager
            .invalidate_pattern("siman:count:*")
            .await?;
        Ok(())
    }

    /// Invalidate cache after MySIMKARI sync
    pub async fn on_mysimkari_sync_completed(&self) -> Result<()> {
        self.cache_manager
            .invalidate_pattern("mysimkari:count:*")
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_key_to_string() {
        let key = CacheKey::ReferenceData {
            table: "ms_barang".to_string(),
            id: "123".to_string(),
        };
        assert_eq!(key.to_string(), "ref:ms_barang:123");

        let key = CacheKey::GapAnalysis {
            satker_id: Uuid::nil(),
            kode_barang: "1.2.3.4".to_string(),
        };
        assert_eq!(key.to_string(), format!("gap:{}:1.2.3.4", Uuid::nil()));
    }

    #[tokio::test]
    async fn test_cache_manager_basic_operations() {
        let manager = CacheManager::new();

        let key = CacheKey::ReferenceData {
            table: "test".to_string(),
            id: "1".to_string(),
        };

        // Test set and get
        let value = vec!["test1".to_string(), "test2".to_string()];
        manager.set(&key, &value).await.unwrap();

        let cached: Option<Vec<String>> = manager.get(&key).await.unwrap();
        assert_eq!(cached, Some(value));

        // Test remove
        manager.remove(&key).await.unwrap();
        let cached: Option<Vec<String>> = manager.get(&key).await.unwrap();
        assert_eq!(cached, None);
    }

    #[tokio::test]
    async fn test_cache_ttl_and_sensitivity() {
        let manager = CacheManager::new();

        // Reference data should have 1 hour TTL and Low sensitivity
        let key = CacheKey::ReferenceData {
            table: "test".to_string(),
            id: "1".to_string(),
        };
        let (ttl, sensitivity) = manager.get_ttl_and_sensitivity(&key);
        assert_eq!(ttl, Duration::from_secs(3600));
        assert_eq!(sensitivity, SensitivityLevel::Low);

        // Dashboard metrics should have 5 minutes TTL and High sensitivity
        let key = CacheKey::DashboardMetrics {
            metric_type: "test".to_string(),
            tahun: 2024,
            filter: "all".to_string(),
        };
        let (ttl, sensitivity) = manager.get_ttl_and_sensitivity(&key);
        assert_eq!(ttl, Duration::from_secs(300));
        assert_eq!(sensitivity, SensitivityLevel::High);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let manager = Arc::new(CacheManager::new());
        let middleware = CacheInvalidationMiddleware::new(manager.clone());

        // Add some test data
        let key = CacheKey::KebutuhanList {
            tahun: 2024,
            status: None,
            page: 1,
        };
        manager.set(&key, &vec!["test"]).await.unwrap();

        // Invalidate kebutuhan cache
        middleware.on_kebutuhan_changed(2024).await.unwrap();

        // Cache should be cleared
        let cached: Option<Vec<String>> = manager.get(&key).await.unwrap();
        assert_eq!(cached, None);
    }
}
