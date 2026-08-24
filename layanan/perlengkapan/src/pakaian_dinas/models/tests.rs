use super::*;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

#[test]
fn test_gender_conversion() {
    assert_eq!(Gender::from_str("L"), Gender::L);
    assert_eq!(Gender::from_str("P"), Gender::P);
    assert_eq!(Gender::from_str("SEMUA"), Gender::Semua);
    assert_eq!(Gender::from_str("semua"), Gender::Semua);
    assert_eq!(Gender::from_str("unknown"), Gender::Semua);
}

#[test]
fn test_ukuran_group_conversion() {
    assert_eq!(UkuranGroup::from_str("BAJU"), UkuranGroup::Baju);
    assert_eq!(UkuranGroup::from_str("CELANA"), UkuranGroup::Celana);
    assert_eq!(UkuranGroup::from_str("SEPATU"), UkuranGroup::Sepatu);
    assert_eq!(UkuranGroup::from_str("baju"), UkuranGroup::Baju);
}

#[test]
fn test_aktivitas_status_conversion() {
    assert_eq!(
        AktivitasStatus::from_i32(1000),
        Some(AktivitasStatus::Input)
    );
    assert_eq!(
        AktivitasStatus::from_i32(1008),
        Some(AktivitasStatus::Selesai)
    );
    assert_eq!(AktivitasStatus::from_i32(9999), None);
    assert_eq!(AktivitasStatus::Input.to_i32(), 1000);
}

#[test]
fn test_aktivitas_label() {
    assert_eq!(AktivitasStatus::Input.label(), "Input");
    assert_eq!(AktivitasStatus::Selesai.label(), "Selesai");
}

#[test]
fn test_pengajuan_is_open() {
    let mut pengajuan = PengajuanPakaianDinas {
        id: Uuid::new_v4(),
        nama: "Test".to_string(),
        deskripsi: None,
        tgl_mulai: None,
        tgl_selesai: None,
        is_reguler: true,
        tahun: 2026,
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
        is_open: true,
    };

    // No end date = open
    assert!(pengajuan.is_open());

    // Non-regular = always open
    pengajuan.is_reguler = false;
    pengajuan.tgl_selesai = Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
    assert!(pengajuan.is_open());

    // Regular with past end date = closed
    pengajuan.is_reguler = true;
    assert!(!pengajuan.is_open());

    // Regular with future end date = open
    pengajuan.tgl_selesai = Some(NaiveDate::from_ymd_opt(2030, 12, 31).unwrap());
    assert!(pengajuan.is_open());
}
