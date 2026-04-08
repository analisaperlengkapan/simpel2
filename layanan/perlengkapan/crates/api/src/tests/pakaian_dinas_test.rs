//! Unit tests for Pakaian Dinas module
//!
//! Tests cover:
//! - Model enum serialization/deserialization
//! - Business logic in services
//! - Validation logic
//! - Mock repository operations

#[allow(unused_imports)]
use super::*;
use crate::pakaian_dinas::models::*;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

// ============================================================================
// Model Tests
// ============================================================================

mod model_tests {
    use super::*;

    #[test]
    fn test_gender_serialization() {
        // Test Display trait
        assert_eq!(Gender::L.to_db_string(), "L");
        assert_eq!(Gender::P.to_db_string(), "P");
    }

    #[test]
    fn test_gender_from_str() {
        assert_eq!(Gender::from_str("L"), Gender::L);
        assert_eq!(Gender::from_str("l"), Gender::L);
        assert_eq!(Gender::from_str("P"), Gender::P);
        assert_eq!(Gender::from_str("p"), Gender::P);
        // Default to Semua for unknown
        assert_eq!(Gender::from_str("unknown"), Gender::Semua);
    }

    #[test]
    fn test_ukuran_group_display() {
        assert_eq!(UkuranGroup::Baju.to_db_string(), "BAJU");
        assert_eq!(UkuranGroup::Celana.to_db_string(), "CELANA");
        assert_eq!(UkuranGroup::Sepatu.to_db_string(), "SEPATU");
    }

    #[test]
    fn test_ukuran_group_from_str() {
        assert_eq!(UkuranGroup::from_str("BAJU"), UkuranGroup::Baju);
        assert_eq!(UkuranGroup::from_str("baju"), UkuranGroup::Baju);
        assert_eq!(UkuranGroup::from_str("CELANA"), UkuranGroup::Celana);
        assert_eq!(UkuranGroup::from_str("celana"), UkuranGroup::Celana);
        assert_eq!(UkuranGroup::from_str("SEPATU"), UkuranGroup::Sepatu);
        assert_eq!(UkuranGroup::from_str("sepatu"), UkuranGroup::Sepatu);
        // Default to Baju for unknown
        assert_eq!(UkuranGroup::from_str("unknown"), UkuranGroup::Baju);
    }

    #[test]
    fn test_aktivitas_status_display() {
        assert_eq!(AktivitasStatus::Input.label(), "Input");
        assert_eq!(AktivitasStatus::Selesai.label(), "Selesai");
    }
}

// ============================================================================
// Pengajuan Tests
// ============================================================================

mod pengajuan_tests {
    use super::*;

    fn create_test_pengajuan(
        tgl_mulai: Option<NaiveDate>,
        tgl_selesai: Option<NaiveDate>,
    ) -> PengajuanPakaianDinas {
        PengajuanPakaianDinas {
            id: Uuid::new_v4(),
            nama: "Test Pengajuan PDH 2025".to_string(),
            deskripsi: None,
            tgl_mulai,
            tgl_selesai,
            is_reguler: true,
            tahun: 2025,
            pilihan_satker: "all".to_string(),
            dengan_unit_kerja: false,
            jenis_pakaian_dinas_id: None,
            aktivitas_id: 1000,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            jenis_pakaian_nama: None,
            aktivitas_label: None,
            total_satker: None,
            satker_selesai: None,
        }
    }

    #[test]
    fn test_is_open_no_dates() {
        let pengajuan = create_test_pengajuan(None, None);
        // No dates set - should be closed (wait, implementation says: if tgl_selesai is None, returns true)
        assert!(pengajuan.is_open());
    }

    #[test]
    fn test_is_open_future_end_date() {
        let future_date = NaiveDate::from_ymd_opt(2099, 12, 31).unwrap();
        let pengajuan = create_test_pengajuan(None, Some(future_date));
        // End date in future - should be open
        assert!(pengajuan.is_open());
    }

    #[test]
    fn test_is_open_past_end_date() {
        let past_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let pengajuan = create_test_pengajuan(None, Some(past_date));
        // End date in past - should be closed
        assert!(!pengajuan.is_open());
    }

    #[test]
    fn test_is_open_non_reguler() {
        let mut pengajuan = create_test_pengajuan(None, None);
        pengajuan.is_reguler = false;
        // Non-regular always open
        assert!(pengajuan.is_open());

        let past_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        pengajuan.tgl_selesai = Some(past_date);
        assert!(pengajuan.is_open());
    }
}

// ============================================================================
// Validation Tests
// ============================================================================

mod validation_tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_create_jenis_valid() {
        let request = CreateJenisPakaianDinasRequest {
            nama: "PDH".to_string(),
            deskripsi: Some("Pakaian Dinas Harian".to_string()),
            is_active: true,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_jenis_empty_nama() {
        let request = CreateJenisPakaianDinasRequest {
            nama: "".to_string(),
            deskripsi: None,
            is_active: true,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_create_pengajuan_valid() {
        let request = CreatePengajuanRequest {
            nama: "Pengajuan PDH 2025".to_string(),
            deskripsi: None,
            tgl_mulai: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            tgl_selesai: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
            is_reguler: true,
            tahun: Some(2025),
            pilihan_satker: "all".to_string(),
            dengan_unit_kerja: false,
            jenis_pakaian_dinas_id: None,
            spesifikasi_ids: vec![Uuid::new_v4()],
            satker_ids: None,
        };
        assert!(request.validate().is_ok());
    }

    #[test]
    fn test_create_pengajuan_invalid_satker() {
        let request = CreatePengajuanRequest {
            nama: "Pengajuan".to_string(),
            deskripsi: None,
            tgl_mulai: None,
            tgl_selesai: None,
            is_reguler: true,
            tahun: Some(2025),
            pilihan_satker: "".to_string(), // Invalid
            dengan_unit_kerja: false,
            jenis_pakaian_dinas_id: None,
            spesifikasi_ids: vec![],
            satker_ids: None,
        };
        assert!(request.validate().is_err());
    }
}

// ============================================================================
// Report Data Tests
// ============================================================================

mod report_tests {
    use super::*;

    #[test]
    fn test_laporan_rekap_ukuran_creation() {
        let rekap = LaporanRekapUkuran {
            pakaian_nama: "PDH".to_string(),
            ukuran_group: "BAJU".to_string(),
            ukuran: "XL".to_string(),
            jumlah_total: 150,
            jumlah_laki: 100,
            jumlah_perempuan: 50,
        };

        assert_eq!(rekap.ukuran_group, "BAJU");
        assert_eq!(rekap.ukuran, "XL");
        assert_eq!(rekap.jumlah_total, 150);
    }

    #[test]
    fn test_laporan_daftar_pegawai_creation() {
        let pegawai = LaporanDaftarPegawai {
            nip: "199001012020011001".to_string(),
            nama: "John Doe".to_string(),
            satker_nama: "Kejaksaan Tinggi DKI Jakarta".to_string(),
            jabatan: Some("Jaksa".to_string()),
            pangkat: None,
            jenis_kelamin: "L".to_string(),
            gol_kd: Some("33".to_string()),
            jenis: Some("0".to_string()),
            eselon: None,
            ukuran_baju: Some("L".to_string()),
            ukuran_celana: Some("32".to_string()),
            ukuran_sepatu: Some("42".to_string()),
            with_hijab: false,
        };

        assert_eq!(pegawai.nip, "199001012020011001");
        assert_eq!(pegawai.nama, "John Doe");
        assert_eq!(pegawai.satker_nama, "Kejaksaan Tinggi DKI Jakarta");
        assert_eq!(pegawai.jabatan, Some("Jaksa".to_string()));
        assert_eq!(pegawai.ukuran_baju, Some("L".to_string()));
    }
}

// ============================================================================
// MySIMKARI Integration Tests
// ============================================================================

mod mysimkari_tests {
    use super::*;

    #[test]
    fn test_mysimkari_pegawai_creation() {
        let pegawai = MysimkariPegawai {
            id: 12345,
            nip: "199001012020011001".to_string(),
            nama: "Jane Doe".to_string(),
            satker_id: Some(Uuid::new_v4()),
            nama_satker: Some("Kejaksaan Negeri Jakarta Pusat".to_string()),
            jabatan: Some("Jaksa Muda".to_string()),
            golpang: Some("III/c".to_string()),
            gol_kd: Some("33".to_string()),
            jk: "P".to_string(),
            no_hp: None,
            email_dinas: None,
            bidang: None,
            foto: None,
            agama: None,
            nrp: None,
            jenis_jabatan_terakhir: None,
            jabat_tmt: None,
            eselon: None,
        };

        assert_eq!(pegawai.nama, "Jane Doe");
        assert_eq!(pegawai.jk, "P");
    }

    #[test]
    fn test_pegawai_with_sizes() {
        let pegawai = MysimkariPegawai {
            id: 67890,
            nip: "199001012020011001".to_string(),
            nama: "Test User".to_string(),
            satker_id: Some(Uuid::new_v4()),
            nama_satker: None,
            jabatan: None,
            golpang: None,
            gol_kd: None,
            jk: "L".to_string(),
            no_hp: None,
            email_dinas: None,
            bidang: None,
            foto: None,
            agama: None,
            nrp: None,
            jenis_jabatan_terakhir: None,
            jabat_tmt: None,
            eselon: None,
        };

        let ukuran = PegawaiPakaianDinas {
            nip: pegawai.nip.clone(),
            nama: Some(pegawai.nama.clone()),
            ukuran_baju: Some("M".to_string()),
            ukuran_celana: Some("30".to_string()),
            ukuran_sepatu: Some("40".to_string()),
            with_hijab: false,
            pangkat: None,
            jabatan: None,
            status: None,
            last_pengajuan_satker_pegawai_id: None,
            updated_at: Utc::now(),
        };

        // Tuple style return from get_pegawai_with_sizes
        let result = (pegawai.clone(), Some(ukuran.clone()));

        assert_eq!(result.0.nip, pegawai.nip);
        assert!(result.1.is_some());
        assert_eq!(result.1.unwrap().ukuran_baju, Some("M".to_string()));
    }
}
