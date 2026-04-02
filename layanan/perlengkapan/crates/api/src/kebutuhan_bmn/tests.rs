//! # Kebutuhan BMN Unit Tests
//!
//! Comprehensive unit tests for BMN needs analysis module.

use chrono::NaiveDate;
use uuid::Uuid;
use validator::Validate;

use super::models::*;
use super::repository::UserInfo;

// ============================================================================
// Model Tests
// ============================================================================

mod model_tests {
    use super::*;

    #[test]
    fn test_status_from_code_valid() {
        assert_eq!(KebutuhanBmnStatus::from_code(2000), Some(KebutuhanBmnStatus::Draft));
        assert_eq!(KebutuhanBmnStatus::from_code(2001), Some(KebutuhanBmnStatus::InputBarang));
        assert_eq!(KebutuhanBmnStatus::from_code(2002), Some(KebutuhanBmnStatus::SubmitWilayah));
        assert_eq!(KebutuhanBmnStatus::from_code(2003), Some(KebutuhanBmnStatus::RevisiSatker));
        assert_eq!(KebutuhanBmnStatus::from_code(2004), Some(KebutuhanBmnStatus::SubmitPusat));
        assert_eq!(KebutuhanBmnStatus::from_code(2005), Some(KebutuhanBmnStatus::AnalisisKelayakan));
        assert_eq!(KebutuhanBmnStatus::from_code(2006), Some(KebutuhanBmnStatus::Approved));
        assert_eq!(KebutuhanBmnStatus::from_code(2007), Some(KebutuhanBmnStatus::Rejected));
        assert_eq!(KebutuhanBmnStatus::from_code(2008), Some(KebutuhanBmnStatus::Completed));
        assert_eq!(KebutuhanBmnStatus::from_code(2009), Some(KebutuhanBmnStatus::Cancelled));
        assert_eq!(KebutuhanBmnStatus::from_code(2010), Some(KebutuhanBmnStatus::RevisiWilayah));
    }

    #[test]
    fn test_status_labels() {
        assert_eq!(KebutuhanBmnStatus::Draft.label(), "Draft");
        assert_eq!(KebutuhanBmnStatus::AnalisisKelayakan.label(), "Analisis Kelayakan");
    }

    #[test]
    fn test_status_transitions() {
        let draft = KebutuhanBmnStatus::Draft;
        assert!(draft.can_transition_to(KebutuhanBmnStatus::InputBarang));
        assert!(draft.can_transition_to(KebutuhanBmnStatus::Cancelled));

        let sub_wil = KebutuhanBmnStatus::SubmitWilayah;
        assert!(sub_wil.can_transition_to(KebutuhanBmnStatus::SubmitPusat));
        assert!(sub_wil.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

        let sub_pusat = KebutuhanBmnStatus::SubmitPusat;
        assert!(sub_pusat.can_transition_to(KebutuhanBmnStatus::AnalisisKelayakan));

        let analisis = KebutuhanBmnStatus::AnalisisKelayakan;
        assert!(analisis.can_transition_to(KebutuhanBmnStatus::Approved));
        assert!(analisis.can_transition_to(KebutuhanBmnStatus::Rejected));
        assert!(analisis.can_transition_to(KebutuhanBmnStatus::RevisiWilayah));
    }
}

mod workflow_tests {
    use super::*;

    #[test]
    fn test_allowed_transitions_list() {
        let draft = KebutuhanBmnStatus::Draft;
        let allowed = draft.allowed_transitions();
        assert!(allowed.contains(&KebutuhanBmnStatus::InputBarang));
        assert!(allowed.contains(&KebutuhanBmnStatus::Cancelled));
    }

    #[test]
    fn test_submit_satker_transitions() {
        let status = KebutuhanBmnStatus::SubmitWilayah;
        assert!(status.can_transition_to(KebutuhanBmnStatus::SubmitPusat));
        assert!(status.can_transition_to(KebutuhanBmnStatus::RevisiSatker));
    }

    #[test]
    fn test_analisis_kelayakan_transitions() {
        let status = KebutuhanBmnStatus::AnalisisKelayakan;
        assert!(status.can_transition_to(KebutuhanBmnStatus::Approved));
        assert!(status.can_transition_to(KebutuhanBmnStatus::Rejected));
        assert!(status.can_transition_to(KebutuhanBmnStatus::RevisiWilayah));
    }
}

mod validation_tests {
    use super::*;

    fn create_valid_pengajuan_request() -> CreatePengajuanRequest {
        CreatePengajuanRequest {
            nama: "Test Pengajuan".to_string(),
            deskripsi: Some("Test description".to_string()),
            tahun: 2026,
            tgl_mulai: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            tgl_selesai: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
            pilihan_satker: Some("semua".to_string()),
            satker_ids: vec![],
            asset_types: vec![],
        }
    }

    #[test]
    fn test_valid_pengajuan_request() {
        let request = create_valid_pengajuan_request();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_pengajuan_nama_too_short() {
        let mut request = create_valid_pengajuan_request();
        request.nama = "AB".to_string();
        assert!(request.validate().is_err());
    }
}

mod response_tests {
    use super::*;

    #[test]
    fn test_analisis_summary() {
        let summary = AnalisisSummary {
            total_diminta: 100,
            total_existing: 30,
            total_gap: 70,
            kelayakan_persen: 30.0,
        };
        assert_eq!(summary.total_gap, 70);
    }
}
