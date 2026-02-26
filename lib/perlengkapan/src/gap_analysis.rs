//! Gap Analysis Algorithm
//!
//! Calculates the gap between standard quantity and existing good condition BMN

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "backend")]
use std::collections::HashMap;

/// Gap analysis result for a single kode barang
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GapAnalysisResult {
    pub kode_barang: String,
    pub nama_barang: String,
    pub satker_id: Uuid,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
    pub gap: i32,
    pub gap_percentage: f64,
    pub calculated_at: DateTime<Utc>,
}

impl GapAnalysisResult {
    /// Create a new gap analysis result
    pub fn new(
        kode_barang: String,
        nama_barang: String,
        satker_id: Uuid,
        standard_quantity: i32,
        existing_good_quantity: i32,
    ) -> Self {
        let gap = standard_quantity - existing_good_quantity;
        let gap_percentage = if standard_quantity > 0 {
            (gap as f64 / standard_quantity as f64) * 100.0
        } else {
            0.0
        };

        Self {
            kode_barang,
            nama_barang,
            satker_id,
            standard_quantity,
            existing_good_quantity,
            gap,
            gap_percentage,
            calculated_at: Utc::now(),
        }
    }

    /// Check if there is a gap (need for procurement)
    pub fn has_gap(&self) -> bool {
        self.gap > 0
    }

    /// Check if gap is critical (>50% of standard)
    pub fn is_critical(&self) -> bool {
        self.gap_percentage > 50.0
    }
}

/// Gap analyzer with caching support
#[cfg(feature = "backend")]
pub struct GapAnalyzer {
    cache: HashMap<String, GapAnalysisResult>,
    cache_ttl_seconds: i64,
}

#[cfg(feature = "backend")]
impl GapAnalyzer {
    /// Create a new gap analyzer with default cache TTL (1 hour)
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl_seconds: 3600,
        }
    }

    /// Create a new gap analyzer with custom cache TTL
    pub fn with_cache_ttl(cache_ttl_seconds: i64) -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl_seconds,
        }
    }

    /// Calculate gap for a single kode barang
    ///
    /// This method checks the cache first. If not found or expired,
    /// it calculates the gap and stores it in the cache.
    pub fn calculate_gap(
        &mut self,
        satker_id: Uuid,
        kode_barang: String,
        nama_barang: String,
        standard_quantity: i32,
        existing_good_quantity: i32,
    ) -> GapAnalysisResult {
        let cache_key = format!("{}:{}", satker_id, kode_barang);

        // Check cache
        if let Some(cached_result) = self.cache.get(&cache_key) {
            let age = Utc::now()
                .signed_duration_since(cached_result.calculated_at)
                .num_seconds();

            if age < self.cache_ttl_seconds {
                return cached_result.clone();
            }
        }

        // Calculate new result
        let result = GapAnalysisResult::new(
            kode_barang,
            nama_barang,
            satker_id,
            standard_quantity,
            existing_good_quantity,
        );

        // Store in cache
        self.cache.insert(cache_key, result.clone());

        result
    }

    /// Calculate gaps for multiple kode barang
    pub fn calculate_gaps_batch(&mut self, items: Vec<GapAnalysisInput>) -> Vec<GapAnalysisResult> {
        items
            .into_iter()
            .map(|input| {
                self.calculate_gap(
                    input.satker_id,
                    input.kode_barang,
                    input.nama_barang,
                    input.standard_quantity,
                    input.existing_good_quantity,
                )
            })
            .collect()
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Invalidate cache for a specific satker
    pub fn invalidate_satker_cache(&mut self, satker_id: Uuid) {
        let satker_prefix = format!("{}:", satker_id);
        self.cache.retain(|key, _| !key.starts_with(&satker_prefix));
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let now = Utc::now();
        let mut valid_entries = 0;
        let mut expired_entries = 0;

        for result in self.cache.values() {
            let age = now
                .signed_duration_since(result.calculated_at)
                .num_seconds();

            if age < self.cache_ttl_seconds {
                valid_entries += 1;
            } else {
                expired_entries += 1;
            }
        }

        CacheStats {
            total_entries: self.cache.len(),
            valid_entries,
            expired_entries,
            ttl_seconds: self.cache_ttl_seconds,
        }
    }
}

#[cfg(feature = "backend")]
impl Default for GapAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Input for gap analysis calculation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GapAnalysisInput {
    pub satker_id: Uuid,
    pub kode_barang: String,
    pub nama_barang: String,
    pub standard_quantity: i32,
    pub existing_good_quantity: i32,
}

/// Cache statistics
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub ttl_seconds: i64,
}

/// Aggregated gap analysis by category
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GapAnalysisByCategory {
    pub category: String,
    pub total_items: i32,
    pub total_standard_quantity: i32,
    pub total_existing_quantity: i32,
    pub total_gap: i32,
    pub average_gap_percentage: f64,
    pub items: Vec<GapAnalysisResult>,
}

impl GapAnalysisByCategory {
    /// Create aggregated gap analysis from individual results
    pub fn from_results(category: String, results: Vec<GapAnalysisResult>) -> Self {
        let total_items = results.len() as i32;
        let total_standard_quantity: i32 = results.iter().map(|r| r.standard_quantity).sum();
        let total_existing_quantity: i32 = results.iter().map(|r| r.existing_good_quantity).sum();
        let total_gap: i32 = results.iter().map(|r| r.gap).sum();

        let average_gap_percentage = if !results.is_empty() {
            results.iter().map(|r| r.gap_percentage).sum::<f64>() / results.len() as f64
        } else {
            0.0
        };

        Self {
            category,
            total_items,
            total_standard_quantity,
            total_existing_quantity,
            total_gap,
            average_gap_percentage,
            items: results,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_analysis_result_creation() {
        let satker_id = Uuid::new_v4();
        let result = GapAnalysisResult::new(
            "1.01.01.01.001".to_string(),
            "Meja Kerja".to_string(),
            satker_id,
            10,
            6,
        );

        assert_eq!(result.gap, 4);
        assert_eq!(result.gap_percentage, 40.0);
        assert!(result.has_gap());
        assert!(!result.is_critical());
    }

    #[test]
    fn test_gap_analysis_critical() {
        let satker_id = Uuid::new_v4();
        let result = GapAnalysisResult::new(
            "1.01.01.01.001".to_string(),
            "Meja Kerja".to_string(),
            satker_id,
            10,
            4,
        );

        assert_eq!(result.gap, 6);
        assert_eq!(result.gap_percentage, 60.0);
        assert!(result.has_gap());
        assert!(result.is_critical());
    }

    #[test]
    fn test_gap_analysis_no_gap() {
        let satker_id = Uuid::new_v4();
        let result = GapAnalysisResult::new(
            "1.01.01.01.001".to_string(),
            "Meja Kerja".to_string(),
            satker_id,
            10,
            12,
        );

        assert_eq!(result.gap, -2);
        assert!(!result.has_gap());
    }

    #[cfg(feature = "backend")]
    #[test]
    fn test_gap_analyzer_caching() {
        let mut analyzer = GapAnalyzer::new();
        let satker_id = Uuid::new_v4();

        let result1 = analyzer.calculate_gap(
            satker_id,
            "1.01.01.01.001".to_string(),
            "Meja Kerja".to_string(),
            10,
            6,
        );

        let result2 = analyzer.calculate_gap(
            satker_id,
            "1.01.01.01.001".to_string(),
            "Meja Kerja".to_string(),
            10,
            6,
        );

        // Should return cached result
        assert_eq!(result1.calculated_at, result2.calculated_at);

        let stats = analyzer.cache_stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.valid_entries, 1);
    }

    #[cfg(feature = "backend")]
    #[test]
    fn test_gap_analyzer_batch() {
        let mut analyzer = GapAnalyzer::new();
        let satker_id = Uuid::new_v4();

        let inputs = vec![
            GapAnalysisInput {
                satker_id,
                kode_barang: "1.01.01.01.001".to_string(),
                nama_barang: "Meja Kerja".to_string(),
                standard_quantity: 10,
                existing_good_quantity: 6,
            },
            GapAnalysisInput {
                satker_id,
                kode_barang: "1.01.01.01.002".to_string(),
                nama_barang: "Kursi Kerja".to_string(),
                standard_quantity: 20,
                existing_good_quantity: 15,
            },
        ];

        let results = analyzer.calculate_gaps_batch(inputs);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].gap, 4);
        assert_eq!(results[1].gap, 5);
    }

    #[test]
    fn test_gap_analysis_by_category() {
        let satker_id = Uuid::new_v4();
        let results = vec![
            GapAnalysisResult::new(
                "1.01.01.01.001".to_string(),
                "Meja Kerja".to_string(),
                satker_id,
                10,
                6,
            ),
            GapAnalysisResult::new(
                "1.01.01.01.002".to_string(),
                "Kursi Kerja".to_string(),
                satker_id,
                20,
                15,
            ),
        ];

        let category_analysis =
            GapAnalysisByCategory::from_results("Furniture".to_string(), results);

        assert_eq!(category_analysis.total_items, 2);
        assert_eq!(category_analysis.total_standard_quantity, 30);
        assert_eq!(category_analysis.total_existing_quantity, 21);
        assert_eq!(category_analysis.total_gap, 9);
    }
}
