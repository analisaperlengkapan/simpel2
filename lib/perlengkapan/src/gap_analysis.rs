//! Gap Analysis Algorithm
//!
//! Calculates the gap between standard quantity and existing good condition BMN

use chrono::{DateTime, Utc};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
