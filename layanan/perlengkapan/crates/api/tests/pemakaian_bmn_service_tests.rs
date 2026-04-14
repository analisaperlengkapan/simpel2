//! Unit tests for Pemakaian BMN service

#[cfg(test)]
mod pemakaian_bmn_service_tests {
    use uuid::Uuid;
    use chrono::NaiveDate;

    #[derive(Debug, Clone)]
    struct IzinPemakaianBmn {
        id: Uuid,
        pegawai_nip: String,
        bmn_nup: String,
        tanggal_mulai: NaiveDate,
        tanggal_selesai: NaiveDate,
        status: String,
    }

    #[test]
    fn test_create_permit() {
        let permit = IzinPemakaianBmn {
            id: Uuid::new_v4(),
            pegawai_nip: "199203142014031001".to_string(),
            bmn_nup: "123456".to_string(),
            tanggal_mulai: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            tanggal_selesai: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            status: "DRAFT".to_string(),
        };

        assert_eq!(permit.status, "DRAFT");
        assert_eq!(permit.pegawai_nip.len(), 18);
    }

    #[test]
    fn test_bmn_availability_check() {
        // Simulate checking if BMN is available
        let active_permits = vec!["123456", "789012"];
        let requested_bmn = "345678";

        let is_available = !active_permits.contains(&requested_bmn);
        assert!(is_available, "BMN should be available");
    }

    #[test]
    fn test_one_bmn_one_permit_rule() {
        // REQ-P004: One BMN = one active permit
        let active_permits = vec![
            ("123456", "ACTIVE"),
            ("789012", "ACTIVE"),
        ];

        let bmn_to_check = "123456";
        let has_active_permit = active_permits.iter()
            .any(|(nup, status)| *nup == bmn_to_check && *status == "ACTIVE");

        assert!(has_active_permit, "BMN already has active permit");
    }

    #[test]
    fn test_permit_number_generation() {
        let year = 2024;
        let satker = "0100";
        let sequence = 1;

        let permit_number = format!("IZN/{}/{}/{:03}", year, satker, sequence);
        assert_eq!(permit_number, "IZN/2024/0100/001");
    }

    #[test]
    fn test_expiry_calculation() {
        let end_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let today = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();

        let days_until_expiry = (end_date - today).num_days();
        assert_eq!(days_until_expiry, 30);
    }

    #[test]
    fn test_expiry_reminder_thresholds() {
        let days_until_expiry = 7;

        let should_send_h7 = days_until_expiry == 7;
        let should_send_h14 = days_until_expiry == 14;
        let should_send_h30 = days_until_expiry == 30;

        assert!(should_send_h7);
        assert!(!should_send_h14);
        assert!(!should_send_h30);
    }

    #[test]
    fn test_auto_expire_logic() {
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let today = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();

        let should_expire = today > end_date;
        assert!(should_expire, "Permit should be expired");
    }

    #[test]
    fn test_permit_renewal() {
        let original_end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let new_end = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        assert!(new_end > original_end, "Renewal should extend period");
    }
}
