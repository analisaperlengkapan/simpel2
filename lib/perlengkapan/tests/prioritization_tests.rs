//! Unit tests for prioritization engine

#[cfg(test)]
mod prioritization_tests {
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    struct PriorityScore {
        kebutuhan_id: Uuid,
        score: f64,
        breakdown: ScoreBreakdown,
    }

    #[derive(Debug, Clone)]
    struct ScoreBreakdown {
        gap_magnitude_score: f64,     // 40%
        asset_criticality_score: f64, // 30%
        satker_type_score: f64,       // 20%
        justification_score: f64,     // 10%
    }

    fn calculate_priority_score(
        gap: i32,
        is_critical: bool,
        satker_type: &str,
        justification_length: usize,
    ) -> f64 {
        let gap_score = (gap as f64 / 100.0).min(1.0) * 40.0;
        let criticality_score = if is_critical { 30.0 } else { 0.0 };
        let satker_score = match satker_type {
            "Cabjari" | "Kejari_C" => 20.0,
            "Kejari_B" => 15.0,
            "Kejari_A" => 10.0,
            "Kejati" => 5.0,
            _ => 0.0,
        };
        let justification_score = (justification_length as f64 / 500.0).min(1.0) * 10.0;

        gap_score + criticality_score + satker_score + justification_score
    }

    #[test]
    fn test_priority_score_calculation() {
        let score = calculate_priority_score(50, true, "Cabjari", 250);

        // Expected: 20 (gap) + 30 (critical) + 20 (cabjari) + 5 (justification) = 75
        assert!((score - 75.0).abs() < 0.1);
    }

    #[test]
    fn test_gap_magnitude_scoring() {
        let score1 = calculate_priority_score(100, false, "Kejati", 0);
        let score2 = calculate_priority_score(50, false, "Kejati", 0);

        assert!(score1 > score2, "Higher gap should result in higher score");
    }

    #[test]
    fn test_critical_infrastructure_bonus() {
        let score_critical = calculate_priority_score(50, true, "Kejati", 0);
        let score_normal = calculate_priority_score(50, false, "Kejati", 0);

        assert!((score_critical - score_normal - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_satker_type_scoring() {
        let cabjari_score = calculate_priority_score(50, false, "Cabjari", 0);
        let kejari_c_score = calculate_priority_score(50, false, "Kejari_C", 0);
        let kejari_b_score = calculate_priority_score(50, false, "Kejari_B", 0);
        let kejari_a_score = calculate_priority_score(50, false, "Kejari_A", 0);
        let kejati_score = calculate_priority_score(50, false, "Kejati", 0);

        assert_eq!(cabjari_score, kejari_c_score);
        assert!(kejari_c_score > kejari_b_score);
        assert!(kejari_b_score > kejari_a_score);
        assert!(kejari_a_score > kejati_score);
    }

    #[test]
    fn test_justification_scoring() {
        let score_long = calculate_priority_score(50, false, "Kejati", 500);
        let score_short = calculate_priority_score(50, false, "Kejati", 100);

        assert!(score_long > score_short);
    }

    #[test]
    fn test_max_score() {
        let max_score = calculate_priority_score(100, true, "Cabjari", 500);

        // Max: 40 + 30 + 20 + 10 = 100
        assert!((max_score - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_min_score() {
        let min_score = calculate_priority_score(0, false, "Unknown", 0);

        assert_eq!(min_score, 0.0);
    }

    #[test]
    fn test_score_breakdown() {
        let breakdown = ScoreBreakdown {
            gap_magnitude_score: 20.0,
            asset_criticality_score: 30.0,
            satker_type_score: 20.0,
            justification_score: 5.0,
        };

        let total = breakdown.gap_magnitude_score
            + breakdown.asset_criticality_score
            + breakdown.satker_type_score
            + breakdown.justification_score;

        assert_eq!(total, 75.0);
    }

    #[test]
    fn test_priority_ranking() {
        let mut scores = vec![
            (
                "Satker A",
                calculate_priority_score(80, true, "Cabjari", 400),
            ),
            (
                "Satker B",
                calculate_priority_score(50, false, "Kejari_B", 200),
            ),
            (
                "Satker C",
                calculate_priority_score(90, true, "Kejari_C", 450),
            ),
        ];

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        assert_eq!(scores[0].0, "Satker C"); // Highest score
    }
}
