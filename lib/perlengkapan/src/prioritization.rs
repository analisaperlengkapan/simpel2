//! Prioritization Algorithm
//!
//! Calculates priority scores for BMN requirements based on multiple factors:
//! - Gap magnitude (40%)
//! - Asset criticality (30%)
//! - Satker type (20%)
//! - Justification quality (10%)

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Priority score for a BMN requirement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PriorityScore {
    pub kebutuhan_id: Uuid,
    pub score: f64,
    pub breakdown: ScoreBreakdown,
    pub priority_level: PriorityLevel,
}

impl PriorityScore {
    /// Create a new priority score
    pub fn new(kebutuhan_id: Uuid, breakdown: ScoreBreakdown) -> Self {
        let score = breakdown.total_score();
        let priority_level = PriorityLevel::from_score(score);

        Self {
            kebutuhan_id,
            score,
            breakdown,
            priority_level,
        }
    }
}

/// Breakdown of priority score components
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScoreBreakdown {
    pub gap_magnitude_score: f64,     // 40% weight
    pub asset_criticality_score: f64, // 30% weight
    pub satker_type_score: f64,       // 20% weight
    pub justification_score: f64,     // 10% weight
}

impl ScoreBreakdown {
    /// Calculate total score (0-100)
    pub fn total_score(&self) -> f64 {
        self.gap_magnitude_score
            + self.asset_criticality_score
            + self.satker_type_score
            + self.justification_score
    }
}

/// Priority level classification
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PriorityLevel {
    Critical, // 80-100
    High,     // 60-79
    Medium,   // 40-59
    Low,      // 0-39
}

impl PriorityLevel {
    /// Determine priority level from score
    pub fn from_score(score: f64) -> Self {
        if score >= 80.0 {
            PriorityLevel::Critical
        } else if score >= 60.0 {
            PriorityLevel::High
        } else if score >= 40.0 {
            PriorityLevel::Medium
        } else {
            PriorityLevel::Low
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            PriorityLevel::Critical => "CRITICAL",
            PriorityLevel::High => "HIGH",
            PriorityLevel::Medium => "MEDIUM",
            PriorityLevel::Low => "LOW",
        }
    }
}

/// Satker type for prioritization
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SatkerType {
    Cabjari,  // Cabang Kejaksaan Negeri (highest priority)
    KejariC,  // Kejaksaan Negeri Kelas C
    KejariB,  // Kejaksaan Negeri Kelas B
    KejariA,  // Kejaksaan Negeri Kelas A
    Kejati,   // Kejaksaan Tinggi
    Kejagung, // Kejaksaan Agung
}

impl SatkerType {
    /// Parse satker type from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "CABJARI" => Some(SatkerType::Cabjari),
            "KEJARI_C" | "KEJARI C" => Some(SatkerType::KejariC),
            "KEJARI_B" | "KEJARI B" => Some(SatkerType::KejariB),
            "KEJARI_A" | "KEJARI A" => Some(SatkerType::KejariA),
            "KEJATI" => Some(SatkerType::Kejati),
            "KEJAGUNG" => Some(SatkerType::Kejagung),
            _ => None,
        }
    }

    /// Get score for satker type (0-20)
    pub fn score(&self) -> f64 {
        match self {
            SatkerType::Cabjari => 20.0,
            SatkerType::KejariC => 20.0,
            SatkerType::KejariB => 15.0,
            SatkerType::KejariA => 10.0,
            SatkerType::Kejati => 5.0,
            SatkerType::Kejagung => 0.0,
        }
    }
}

/// Prioritization engine
pub struct PrioritizationEngine;

impl PrioritizationEngine {
    /// Calculate priority score for a BMN requirement
    ///
    /// # Arguments
    /// * `kebutuhan_id` - UUID of the kebutuhan BMN
    /// * `gap` - Gap between standard and existing quantity
    /// * `is_critical_infrastructure` - Whether the asset is critical infrastructure
    /// * `satker_type` - Type of satker
    /// * `justification_length` - Length of justification text
    ///
    /// # Returns
    /// Priority score with breakdown
    pub fn calculate_priority(
        kebutuhan_id: Uuid,
        gap: i32,
        is_critical_infrastructure: bool,
        satker_type: SatkerType,
        justification_length: usize,
    ) -> PriorityScore {
        // Gap magnitude (40%) - linear scale, capped at 100 units
        let gap_magnitude_score = ((gap as f64 / 100.0).min(1.0)) * 40.0;

        // Asset criticality (30%)
        let asset_criticality_score = if is_critical_infrastructure {
            30.0
        } else {
            0.0
        };

        // Satker type (20%)
        let satker_type_score = satker_type.score();

        // Justification quality (10%) - based on length
        // Good justification: >200 characters
        let justification_score = if justification_length > 200 {
            10.0
        } else {
            0.0
        };

        let breakdown = ScoreBreakdown {
            gap_magnitude_score,
            asset_criticality_score,
            satker_type_score,
            justification_score,
        };

        PriorityScore::new(kebutuhan_id, breakdown)
    }

    /// Calculate priority scores for multiple requirements
    pub fn calculate_priorities_batch(items: Vec<PrioritizationInput>) -> Vec<PriorityScore> {
        items
            .into_iter()
            .map(|input| {
                Self::calculate_priority(
                    input.kebutuhan_id,
                    input.gap,
                    input.is_critical_infrastructure,
                    input.satker_type,
                    input.justification_length,
                )
            })
            .collect()
    }

    /// Sort requirements by priority score (descending)
    pub fn sort_by_priority(mut scores: Vec<PriorityScore>) -> Vec<PriorityScore> {
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        scores
    }

    /// Filter requirements by priority level
    pub fn filter_by_level(scores: Vec<PriorityScore>, level: PriorityLevel) -> Vec<PriorityScore> {
        scores
            .into_iter()
            .filter(|s| s.priority_level == level)
            .collect()
    }
}

/// Input for prioritization calculation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PrioritizationInput {
    pub kebutuhan_id: Uuid,
    pub gap: i32,
    pub is_critical_infrastructure: bool,
    pub satker_type: SatkerType,
    pub justification_length: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_level_from_score() {
        assert_eq!(PriorityLevel::from_score(90.0), PriorityLevel::Critical);
        assert_eq!(PriorityLevel::from_score(70.0), PriorityLevel::High);
        assert_eq!(PriorityLevel::from_score(50.0), PriorityLevel::Medium);
        assert_eq!(PriorityLevel::from_score(30.0), PriorityLevel::Low);
    }

    #[test]
    fn test_satker_type_parsing() {
        assert_eq!(SatkerType::from_str("CABJARI"), Some(SatkerType::Cabjari));
        assert_eq!(SatkerType::from_str("KEJARI_C"), Some(SatkerType::KejariC));
        assert_eq!(SatkerType::from_str("KEJATI"), Some(SatkerType::Kejati));
        assert_eq!(SatkerType::from_str("INVALID"), None);
    }

    #[test]
    fn test_satker_type_scores() {
        assert_eq!(SatkerType::Cabjari.score(), 20.0);
        assert_eq!(SatkerType::KejariC.score(), 20.0);
        assert_eq!(SatkerType::KejariB.score(), 15.0);
        assert_eq!(SatkerType::KejariA.score(), 10.0);
        assert_eq!(SatkerType::Kejati.score(), 5.0);
        assert_eq!(SatkerType::Kejagung.score(), 0.0);
    }

    #[test]
    fn test_calculate_priority_critical() {
        let kebutuhan_id = Uuid::new_v4();
        let score = PrioritizationEngine::calculate_priority(
            kebutuhan_id,
            80,                  // Large gap
            true,                // Critical infrastructure
            SatkerType::Cabjari, // High priority satker
            250,                 // Good justification
        );

        // Gap: 80/100 * 40 = 32
        // Criticality: 30
        // Satker: 20
        // Justification: 10
        // Total: 92
        assert_eq!(score.score, 92.0);
        assert_eq!(score.priority_level, PriorityLevel::Critical);
    }

    #[test]
    fn test_calculate_priority_low() {
        let kebutuhan_id = Uuid::new_v4();
        let score = PrioritizationEngine::calculate_priority(
            kebutuhan_id,
            10,                   // Small gap
            false,                // Not critical
            SatkerType::Kejagung, // Low priority satker
            50,                   // Poor justification
        );

        // Gap: 10/100 * 40 = 4
        // Criticality: 0
        // Satker: 0
        // Justification: 0
        // Total: 4
        assert_eq!(score.score, 4.0);
        assert_eq!(score.priority_level, PriorityLevel::Low);
    }

    #[test]
    fn test_calculate_priority_medium() {
        let kebutuhan_id = Uuid::new_v4();
        let score = PrioritizationEngine::calculate_priority(
            kebutuhan_id,
            50,                  // Medium gap
            false,               // Not critical
            SatkerType::KejariB, // Medium priority satker
            250,                 // Good justification
        );

        // Gap: 50/100 * 40 = 20
        // Criticality: 0
        // Satker: 15
        // Justification: 10
        // Total: 45
        assert_eq!(score.score, 45.0);
        assert_eq!(score.priority_level, PriorityLevel::Medium);
    }

    #[test]
    fn test_batch_prioritization() {
        let inputs = vec![
            PrioritizationInput {
                kebutuhan_id: Uuid::new_v4(),
                gap: 80,
                is_critical_infrastructure: true,
                satker_type: SatkerType::Cabjari,
                justification_length: 250,
            },
            PrioritizationInput {
                kebutuhan_id: Uuid::new_v4(),
                gap: 10,
                is_critical_infrastructure: false,
                satker_type: SatkerType::Kejagung,
                justification_length: 50,
            },
        ];

        let scores = PrioritizationEngine::calculate_priorities_batch(inputs);
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].score, 92.0);
        assert_eq!(scores[1].score, 4.0);
    }

    #[test]
    fn test_sort_by_priority() {
        let scores = vec![
            PriorityScore::new(
                Uuid::new_v4(),
                ScoreBreakdown {
                    gap_magnitude_score: 20.0,
                    asset_criticality_score: 0.0,
                    satker_type_score: 10.0,
                    justification_score: 0.0,
                },
            ),
            PriorityScore::new(
                Uuid::new_v4(),
                ScoreBreakdown {
                    gap_magnitude_score: 32.0,
                    asset_criticality_score: 30.0,
                    satker_type_score: 20.0,
                    justification_score: 10.0,
                },
            ),
        ];

        let sorted = PrioritizationEngine::sort_by_priority(scores);
        assert_eq!(sorted[0].score, 92.0);
        assert_eq!(sorted[1].score, 30.0);
    }

    #[test]
    fn test_filter_by_level() {
        let scores = vec![
            PriorityScore::new(
                Uuid::new_v4(),
                ScoreBreakdown {
                    gap_magnitude_score: 32.0,
                    asset_criticality_score: 30.0,
                    satker_type_score: 20.0,
                    justification_score: 10.0,
                },
            ),
            PriorityScore::new(
                Uuid::new_v4(),
                ScoreBreakdown {
                    gap_magnitude_score: 20.0,
                    asset_criticality_score: 0.0,
                    satker_type_score: 10.0,
                    justification_score: 0.0,
                },
            ),
        ];

        let critical =
            PrioritizationEngine::filter_by_level(scores.clone(), PriorityLevel::Critical);
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].score, 92.0);

        let low = PrioritizationEngine::filter_by_level(scores, PriorityLevel::Low);
        assert_eq!(low.len(), 1);
        assert_eq!(low[0].score, 30.0);
    }
}
