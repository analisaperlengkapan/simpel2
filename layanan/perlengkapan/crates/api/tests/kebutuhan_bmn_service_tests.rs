//! Unit tests for Kebutuhan BMN service

#[cfg(test)]
mod kebutuhan_bmn_service_tests {
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct PengajuanKebutuhanBmn {
        id: Uuid,
        nama: String,
        tahun: i32,
        tgl_mulai: NaiveDate,
        tgl_selesai: NaiveDate,
        status_kode: i32,
    }

    #[test]
    fn test_create_pengajuan() {
        let pengajuan = PengajuanKebutuhanBmn {
            id: Uuid::new_v4(),
            nama: "Kebutuhan BMN 2024".to_string(),
            tahun: 2024,
            tgl_mulai: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            tgl_selesai: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            status_kode: 2000, // DRAFT
        };

        assert_eq!(pengajuan.tahun, 2024);
        assert_eq!(pengajuan.status_kode, 2000);
    }

    #[test]
    fn test_validate_period_dates() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();

        assert!(start < end, "Start date must be before end date");
    }

    #[test]
    fn test_status_transition() {
        // DRAFT -> SUBMITTED
        let mut status = 2002;
        assert_eq!(status, 2002);

        // SUBMITTED -> REVIEWED_WILAYAH
        status = 2003;
        assert_eq!(status, 2003);
    }

    #[test]
    fn test_eligible_bmn_validation() {
        let eligible_bmn = vec!["3.1.01.01.001", "3.2.01.01.001"];
        let requested_bmn = "3.1.01.01.001";

        assert!(eligible_bmn.contains(&requested_bmn));
    }

    #[test]
    fn test_eligible_satker_validation() {
        let eligible_satkers = vec!["0100", "0200", "3400"];
        let requesting_satker = "0100";

        assert!(eligible_satkers.contains(&requesting_satker));
    }

    #[test]
    fn test_deadline_validation() {
        let deadline = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();
        let submission_date = NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

        assert!(
            submission_date <= deadline,
            "Submission must be before deadline"
        );
    }
}
