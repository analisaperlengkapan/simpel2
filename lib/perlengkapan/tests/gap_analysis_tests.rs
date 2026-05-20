//! Unit tests for gap analysis algorithm

#[cfg(test)]
mod gap_analysis_tests {
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct GapAnalysisResult {
        kode_barang: String,
        nama_barang: String,
        standard_quantity: i32,
        existing_good_quantity: i32,
        gap: i32,
    }

    fn calculate_gap(standard: i32, existing: i32) -> i32 {
        standard - existing
    }

    #[test]
    fn test_gap_calculation_positive() {
        let standard = 10;
        let existing = 5;
        let gap = calculate_gap(standard, existing);

        assert_eq!(gap, 5);
        assert!(gap > 0, "Gap should be positive when standard > existing");
    }

    #[test]
    fn test_gap_calculation_zero() {
        let standard = 10;
        let existing = 10;
        let gap = calculate_gap(standard, existing);

        assert_eq!(gap, 0);
    }

    #[test]
    fn test_gap_calculation_negative() {
        let standard = 5;
        let existing = 10;
        let gap = calculate_gap(standard, existing);

        assert_eq!(gap, -5);
        assert!(gap < 0, "Gap should be negative when existing > standard");
    }

    #[test]
    fn test_gap_analysis_result_creation() {
        let result = GapAnalysisResult {
            kode_barang: "3.1.01.01.001".to_string(),
            nama_barang: "Kendaraan Roda 4".to_string(),
            standard_quantity: 10,
            existing_good_quantity: 5,
            gap: 5,
        };

        assert_eq!(result.gap, 5);
        assert_eq!(result.kode_barang, "3.1.01.01.001");
    }

    #[test]
    fn test_gap_analysis_with_condition_filter() {
        // Simulate filtering by condition
        let assets = [("BAIK", 5), ("RUSAK RINGAN", 2), ("RUSAK BERAT", 1)];

        let good_count: i32 = assets
            .iter()
            .filter(|(condition, _)| *condition == "BAIK")
            .map(|(_, count)| count)
            .sum();

        assert_eq!(good_count, 5);
    }

    #[test]
    fn test_multiple_satker_gap_analysis() {
        let satker_gaps = [
            ("Kejari A", 10, 5, 5),
            ("Kejari B", 8, 6, 2),
            ("Kejari C", 12, 12, 0),
        ];

        let total_gap: i32 = satker_gaps.iter().map(|(_, _, _, gap)| gap).sum();

        assert_eq!(total_gap, 7);
    }
}
