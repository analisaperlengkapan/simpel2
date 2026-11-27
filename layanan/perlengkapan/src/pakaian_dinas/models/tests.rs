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
        progres: None,
        roster_snapshot_at: None,
        is_open: true,
        periode: "terbuka".to_string(),
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

#[test]
fn satker_list_filter_blank_means_no_filter() {
    let f = SatkerListFilter::parse(None, None).unwrap();
    assert_eq!(f, SatkerListFilter::default());
    // Whitespace and stray commas are what an empty input box and a cleared select send.
    let f = SatkerListFilter::parse(Some("   "), Some(" , ,")).unwrap();
    assert_eq!(f, SatkerListFilter::default());
}

#[test]
fn satker_list_filter_escapes_the_users_wildcards() {
    // `%` and `_` typed into the box are text, not "match anything".
    let f = SatkerListFilter::parse(Some("100%_a\\b"), None).unwrap();
    assert_eq!(f.q_pattern.as_deref(), Some("%100\\%\\_a\\\\b%"));
    let f = SatkerListFilter::parse(Some("  Jakarta Pusat "), None).unwrap();
    assert_eq!(f.q_pattern.as_deref(), Some("%Jakarta Pusat%"));
}

#[test]
fn satker_list_filter_parses_status_codes_once_each() {
    let f = SatkerListFilter::parse(None, Some("1001, 1007,1001,1012")).unwrap();
    assert_eq!(f.statuses, vec![1001, 1007, 1012]);
}

#[test]
fn satker_list_filter_refuses_what_is_not_a_filter() {
    assert!(SatkerListFilter::parse(None, Some("1001,abc")).is_err());
    assert!(SatkerListFilter::parse(None, Some("1;DROP")).is_err());
    let long = "a".repeat(SatkerListFilter::MAX_Q + 1);
    assert!(SatkerListFilter::parse(Some(&long), None).is_err());
    let many = (1000..1000 + SatkerListFilter::MAX_STATUSES as i32 + 1)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    assert!(SatkerListFilter::parse(None, Some(&many)).is_err());
    // The boundary itself is fine.
    let ok = "a".repeat(SatkerListFilter::MAX_Q);
    assert!(SatkerListFilter::parse(Some(&ok), None).is_ok());
}
