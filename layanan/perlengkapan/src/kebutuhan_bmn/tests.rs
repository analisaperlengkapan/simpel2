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
        assert_eq!(
            KebutuhanBmnStatus::from_code(2000),
            Some(KebutuhanBmnStatus::Draft)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2001),
            Some(KebutuhanBmnStatus::InputBarang)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2002),
            Some(KebutuhanBmnStatus::SubmitWilayah)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2003),
            Some(KebutuhanBmnStatus::RevisiSatker)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2004),
            Some(KebutuhanBmnStatus::SubmitPusat)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2005),
            Some(KebutuhanBmnStatus::AnalisisKelayakan)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2006),
            Some(KebutuhanBmnStatus::Approved)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2007),
            Some(KebutuhanBmnStatus::Rejected)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2008),
            Some(KebutuhanBmnStatus::Completed)
        );
        assert_eq!(
            KebutuhanBmnStatus::from_code(2009),
            Some(KebutuhanBmnStatus::Cancelled)
        );
    }

    #[test]
    fn test_status_from_code_invalid() {
        assert_eq!(KebutuhanBmnStatus::from_code(0), None);
        assert_eq!(KebutuhanBmnStatus::from_code(1000), None);
        assert_eq!(KebutuhanBmnStatus::from_code(9999), None);
        assert_eq!(KebutuhanBmnStatus::from_code(-1), None);
    }

    #[test]
    fn test_status_to_code() {
        assert_eq!(KebutuhanBmnStatus::Draft.to_code(), 2000);
        assert_eq!(KebutuhanBmnStatus::Completed.to_code(), 2008);
        assert_eq!(KebutuhanBmnStatus::Cancelled.to_code(), 2009);
    }

    #[test]
    fn test_status_labels() {
        assert_eq!(KebutuhanBmnStatus::Draft.label(), "Draft");
        assert_eq!(KebutuhanBmnStatus::InputBarang.label(), "Input Barang");
        assert_eq!(
            KebutuhanBmnStatus::AnalisisKelayakan.label(),
            "Analisis Kelayakan"
        );
    }

    #[test]
    fn test_status_default() {
        let status: KebutuhanBmnStatus = Default::default();
        assert_eq!(status, KebutuhanBmnStatus::Draft);
    }

    #[test]
    fn test_pilihan_satker_from_str() {
        assert_eq!(PilihanSatker::from_str("semua"), PilihanSatker::Semua);
        assert_eq!(PilihanSatker::from_str("SEMUA"), PilihanSatker::Semua);
        assert_eq!(PilihanSatker::from_str("sebagian"), PilihanSatker::Sebagian);
        assert_eq!(PilihanSatker::from_str("SEBAGIAN"), PilihanSatker::Sebagian);
        assert_eq!(PilihanSatker::from_str("unknown"), PilihanSatker::Semua);
        assert_eq!(PilihanSatker::from_str(""), PilihanSatker::Semua);
    }

    #[test]
    fn test_pilihan_satker_as_str() {
        assert_eq!(PilihanSatker::Semua.as_str(), "semua");
        assert_eq!(PilihanSatker::Sebagian.as_str(), "sebagian");
    }
}

// ============================================================================
// Workflow Transition Tests
// ============================================================================

mod workflow_tests {
    use super::*;

    #[test]
    fn test_draft_transitions() {
        let draft = KebutuhanBmnStatus::Draft;

        // Valid transitions
        assert!(draft.can_transition_to(KebutuhanBmnStatus::InputBarang));
        assert!(draft.can_transition_to(KebutuhanBmnStatus::Cancelled));

        // Invalid transitions
        assert!(!draft.can_transition_to(KebutuhanBmnStatus::Approved));
        assert!(!draft.can_transition_to(KebutuhanBmnStatus::Completed));
        assert!(!draft.can_transition_to(KebutuhanBmnStatus::SubmitWilayah));
    }

    #[test]
    fn test_input_barang_transitions() {
        let status = KebutuhanBmnStatus::InputBarang;

        assert!(status.can_transition_to(KebutuhanBmnStatus::SubmitWilayah));
        assert!(status.can_transition_to(KebutuhanBmnStatus::Cancelled));

        assert!(!status.can_transition_to(KebutuhanBmnStatus::Draft));
        assert!(!status.can_transition_to(KebutuhanBmnStatus::Approved));
    }

    #[test]
    fn test_submit_satker_transitions() {
        let status = KebutuhanBmnStatus::SubmitWilayah;

        assert!(status.can_transition_to(KebutuhanBmnStatus::SubmitPusat));
        assert!(status.can_transition_to(KebutuhanBmnStatus::RevisiSatker));

        assert!(!status.can_transition_to(KebutuhanBmnStatus::Draft));
        assert!(!status.can_transition_to(KebutuhanBmnStatus::Approved));
    }

    #[test]
    fn test_analisis_kelayakan_transitions() {
        let status = KebutuhanBmnStatus::AnalisisKelayakan;

        assert!(status.can_transition_to(KebutuhanBmnStatus::Approved));
        assert!(status.can_transition_to(KebutuhanBmnStatus::Rejected));
        assert!(status.can_transition_to(KebutuhanBmnStatus::RevisiWilayah));

        assert!(!status.can_transition_to(KebutuhanBmnStatus::Draft));
    }

    #[test]
    fn test_approved_transitions() {
        let status = KebutuhanBmnStatus::Approved;

        assert!(status.can_transition_to(KebutuhanBmnStatus::Completed));

        assert!(!status.can_transition_to(KebutuhanBmnStatus::Draft));
        assert!(!status.can_transition_to(KebutuhanBmnStatus::Rejected));
    }

    #[test]
    fn test_terminal_states_no_transitions() {
        // Terminal states should not allow any transitions
        let terminal_states = [
            KebutuhanBmnStatus::Rejected,
            KebutuhanBmnStatus::Completed,
            KebutuhanBmnStatus::Cancelled,
        ];

        for status in &terminal_states {
            assert!(
                status.allowed_transitions().is_empty(),
                "{:?} should have no allowed transitions",
                status
            );

            // Verify it can't transition to anything
            for target in &[
                KebutuhanBmnStatus::Draft,
                KebutuhanBmnStatus::InputBarang,
                KebutuhanBmnStatus::Approved,
            ] {
                assert!(
                    !status.can_transition_to(*target),
                    "{:?} should not be able to transition to {:?}",
                    status,
                    target
                );
            }
        }
    }

    #[test]
    fn test_allowed_transitions_list() {
        let draft = KebutuhanBmnStatus::Draft;
        let allowed = draft.allowed_transitions();

        assert_eq!(allowed.len(), 2);
        assert!(allowed.contains(&KebutuhanBmnStatus::InputBarang));
        assert!(allowed.contains(&KebutuhanBmnStatus::Cancelled));
    }
}

// ============================================================================
// Request Validation Tests
// ============================================================================

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
            wilayah_id: None,
            satker_ids: vec![],
            asset_types: vec![],
            bmn_referensi_diizinkan: vec![],
        }
    }

    fn create_valid_barang_request() -> CreateBarangRequest {
        CreateBarangRequest {
            nama: "Test Barang".to_string(),
            kode_barang: Some("123".to_string()),
            jumlah: 10,
            satuan: "Unit".to_string(),
            alasan: Some("Test reason".to_string()),
            keterangan: None,
            file_pendukung: vec![],
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

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pengajuan_nama_too_long() {
        let mut request = create_valid_pengajuan_request();
        request.nama = "A".repeat(256);

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pengajuan_tahun_too_early() {
        let mut request = create_valid_pengajuan_request();
        request.tahun = 2019;

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_pengajuan_tahun_too_late() {
        let mut request = create_valid_pengajuan_request();
        request.tahun = 2101;

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_barang_request() {
        let request = create_valid_barang_request();
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_barang_empty_name() {
        let mut request = create_valid_barang_request();
        request.nama = "".to_string();

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_barang_zero_jumlah() {
        let mut request = create_valid_barang_request();
        request.jumlah = 0;

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_barang_negative_jumlah() {
        let mut request = create_valid_barang_request();
        request.jumlah = -1;

        let result = request.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_barang_with_files() {
        let mut request = create_valid_barang_request();
        request.file_pendukung = vec!["file1.pdf".to_string(), "file2.jpg".to_string()];

        assert!(request.validate().is_ok());
    }
}

// ============================================================================
// User Info Tests
// ============================================================================

mod user_info_tests {
    use super::*;

    #[test]
    fn test_user_info_creation() {
        let info = UserInfo {
            nip: Some("198501012010011001".to_string()),
            nama: Some("Test User".to_string()),
            pangkat: Some("Penata Tk. I".to_string()),
            jabatan: Some("Analis".to_string()),
            role: Some("validator_pusat".to_string()),
        };

        assert_eq!(info.nip, Some("198501012010011001".to_string()));
        assert_eq!(info.role, Some("validator_pusat".to_string()));
    }

    #[test]
    fn test_user_info_partial() {
        let info = UserInfo {
            nip: Some("123".to_string()),
            nama: None,
            pangkat: None,
            jabatan: None,
            role: Some("user".to_string()),
        };

        assert!(info.nip.is_some());
        assert!(info.nama.is_none());
    }
}

// ============================================================================
// Priority and Scoring Tests
// ============================================================================

mod priority_tests {
    use super::*;

    #[test]
    fn test_prioritas_item() {
        let item = PrioritasItem {
            barang_id: Uuid::new_v4(),
            prioritas: 1,
            skor: Some(85.5),
        };

        assert_eq!(item.prioritas, 1);
        assert_eq!(item.skor, Some(85.5));
    }

    #[test]
    fn test_set_prioritas_request() {
        let request = SetPrioritasRequest {
            items: vec![
                PrioritasItem {
                    barang_id: Uuid::new_v4(),
                    prioritas: 1,
                    skor: Some(90.0),
                },
                PrioritasItem {
                    barang_id: Uuid::new_v4(),
                    prioritas: 2,
                    skor: Some(80.0),
                },
            ],
        };

        assert_eq!(request.items.len(), 2);
        assert_eq!(request.items[0].prioritas, 1);
        assert_eq!(request.items[1].prioritas, 2);
    }
}

// ============================================================================
// Workflow Transition Request Tests
// ============================================================================

mod transition_request_tests {
    use super::*;

    #[test]
    fn test_workflow_transition_request() {
        let request = WorkflowTransitionRequest {
            target_status: 2001,
            komentar: Some("Transitioning to input".to_string()),
        };

        assert_eq!(request.target_status, 2001);
        assert!(request.komentar.is_some());
    }

    #[test]
    fn test_workflow_transition_without_comment() {
        let request = WorkflowTransitionRequest {
            target_status: 2002,
            komentar: None,
        };

        assert_eq!(request.target_status, 2002);
        assert!(request.komentar.is_none());
    }
}

// ============================================================================
// Response Structure Tests
// ============================================================================

mod response_tests {
    use super::*;

    #[test]
    fn test_workflow_transition_info() {
        let info = WorkflowTransitionInfo {
            status_kode: 2001,
            status_nama: "Input Barang".to_string(),
            requires_comment: false,
        };

        assert_eq!(info.status_kode, 2001);
        assert!(!info.requires_comment);
    }

    #[test]
    fn test_analisis_summary() {
        let summary = AnalisisSummary {
            total_diminta: 100,
            total_existing: 30,
            total_gap: 70,
            kelayakan_persen: 30.0,
        };

        assert_eq!(summary.total_gap, 70);
        assert_eq!(summary.kelayakan_persen, 30.0);
    }
}

// ============================================================================
// Filter Tests
// ============================================================================

mod filter_tests {
    use super::*;

    #[test]
    fn test_pengajuan_filter_all_fields() {
        let filter = PengajuanFilter {
            tahun: Some(2026),
            status_kode: Some(2000),
            satker_id: Some("001".to_string()),
            search: Some("test".to_string()),
        };

        assert_eq!(filter.tahun, Some(2026));
        assert_eq!(filter.status_kode, Some(2000));
        assert!(filter.satker_id.is_some());
        assert!(filter.search.is_some());
    }

    #[test]
    fn test_pengajuan_filter_partial() {
        let filter = PengajuanFilter {
            tahun: Some(2026),
            status_kode: None,
            satker_id: None,
            search: None,
        };

        assert!(filter.tahun.is_some());
        assert!(filter.status_kode.is_none());
    }

    #[test]
    fn test_barang_filter() {
        let filter = BarangFilter {
            kode_barang: Some("ABC123".to_string()),
            prioritas_min: Some(5),
            search: Some("laptop".to_string()),
        };

        assert_eq!(filter.kode_barang, Some("ABC123".to_string()));
        assert_eq!(filter.prioritas_min, Some(5));
    }
}

// ============================================================================
// Asset Type Request Tests
// ============================================================================

mod asset_type_tests {
    use super::*;

    #[test]
    fn test_create_asset_type_request() {
        let request = CreateAssetTypeRequest {
            kode_barang: Some("3010101".to_string()),
            nm_barang: Some("Komputer".to_string()),
            ms_jenis_asset_id: Some(1),
            keterangan: Some("Komputer desktop".to_string()),
        };

        assert_eq!(request.kode_barang, Some("3010101".to_string()));
        assert_eq!(request.ms_jenis_asset_id, Some(1));
    }

    #[test]
    fn test_create_asset_type_minimal() {
        let request = CreateAssetTypeRequest {
            kode_barang: None,
            nm_barang: Some("Barang Test".to_string()),
            ms_jenis_asset_id: None,
            keterangan: None,
        };

        assert!(request.kode_barang.is_none());
        assert!(request.nm_barang.is_some());
    }
}
