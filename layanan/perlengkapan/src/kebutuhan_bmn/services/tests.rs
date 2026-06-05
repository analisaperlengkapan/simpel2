use super::*;
use crate::kebutuhan_bmn::models::*;
use chrono::NaiveDate;
use uuid::Uuid;
use validator::Validate;

fn create_valid_request() -> CreatePengajuanRequest {
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

#[test]
fn test_validate_request() {
    let request = create_valid_request();
    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_request_short_name() {
    let mut request = create_valid_request();
    request.nama = "AB".to_string(); // Too short
    assert!(request.validate().is_err());
}

#[test]
fn test_validate_request_invalid_tahun() {
    let mut request = create_valid_request();
    request.tahun = 2019; // Before 2020
    assert!(request.validate().is_err());
}

#[test]
fn test_validate_barang_request() {
    let request = CreateBarangRequest {
        nama: "Test Barang".to_string(),
        kode_barang: Some("123".to_string()),
        jumlah: 10,
        satuan: "Unit".to_string(),
        alasan: Some("Test".to_string()),
        keterangan: None,
        file_pendukung: vec![],
    };
    assert!(request.validate().is_ok());
}

#[test]
fn test_validate_barang_empty_name() {
    let request = CreateBarangRequest {
        nama: "".to_string(),
        kode_barang: None,
        jumlah: 10,
        satuan: "Unit".to_string(),
        alasan: None,
        keterangan: None,
        file_pendukung: vec![],
    };
    assert!(request.validate().is_err());
}

#[test]
fn test_validate_barang_zero_jumlah() {
    let request = CreateBarangRequest {
        nama: "Test".to_string(),
        kode_barang: None,
        jumlah: 0, // Invalid
        satuan: "Unit".to_string(),
        alasan: None,
        keterangan: None,
        file_pendukung: vec![],
    };
    assert!(request.validate().is_err());
}

// ========================================================================
// V029 (#24) — Snapshot analisis kelayakan
// ========================================================================

fn fixture_barang(
    id: Uuid,
    nama: &str,
    jumlah: i32,
    existing_count: i32,
) -> PengajuanKebutuhanBmnBarang {
    let now = chrono::Utc::now();
    PengajuanKebutuhanBmnBarang {
        id,
        pengajuan_satker_id: Uuid::new_v4(),
        nama: nama.to_string(),
        kode_barang: None,
        jumlah,
        satuan: "Unit".to_string(),
        jml_setuju: 0,
        alasan: None,
        keterangan: None,
        prioritas: 0,
        skor: 0.0,
        file_pendukung: serde_json::json!([]),
        existing_count,
        existing_condition: None,
        created_by: None,
        updated_by: None,
        created_at: now,
        updated_at: now,
    }
}

/// Snapshot harus round-trip lewat JSON (kolom JSONB) tanpa kehilangan
/// data — properti inti agar Wilayah/Pusat membaca angka yg sama.
#[test]
fn analisis_snapshot_round_trips_through_json() {
    let snap = AnalisisSnapshot {
        snapshot_at: "2026-06-02T10:00:00Z".to_string(),
        summary: AnalisisSnapshotSummary {
            total_diminta: 10,
            total_existing: 4,
            total_gap: 6,
            kelayakan_persen: 40.0,
        },
        barang: vec![AnalisisSnapshotBarang {
            barang_id: Uuid::new_v4(),
            existing_count: 4,
            gap: 6,
            recommendation: "Pengadaan disarankan".to_string(),
            existing_assets: vec![AnalisisSnapshotAsset {
                no_aset: "A1".to_string(),
                nama_aset: "Kendaraan".to_string(),
                kondisi: "Baik".to_string(),
                lokasi: Some("Gudang".to_string()),
            }],
        }],
    };
    let v = serde_json::to_value(&snap).unwrap();
    let back: AnalisisSnapshot = serde_json::from_value(v).unwrap();
    assert_eq!(back.snapshot_at, snap.snapshot_at);
    assert_eq!(back.summary.total_gap, 6);
    assert_eq!(back.barang.len(), 1);
    assert_eq!(back.barang[0].existing_assets[0].kondisi, "Baik");
}

/// Rebuild dari snapshot harus memakai angka SNAPSHOT (existing/gap),
/// BUKAN `existing_count` live di baris barang — itulah inti konsistensi.
#[test]
fn build_from_snapshot_prefers_snapshot_over_live_existing() {
    let id = Uuid::new_v4();
    // Baris barang DB punya existing_count=99 (mis. SIMAN berubah setelah submit).
    let barang = vec![fixture_barang(id, "Laptop", 10, 99)];
    let snap = AnalisisSnapshot {
        snapshot_at: "2026-06-02T10:00:00Z".to_string(),
        summary: AnalisisSnapshotSummary {
            total_diminta: 10,
            total_existing: 3,
            total_gap: 7,
            kelayakan_persen: 30.0,
        },
        barang: vec![AnalisisSnapshotBarang {
            barang_id: id,
            existing_count: 3,
            gap: 7,
            recommendation: "Beku".to_string(),
            existing_assets: vec![],
        }],
    };
    let (out, summary) = KebutuhanBmnService::build_analisis_from_snapshot(barang, &snap);
    assert_eq!(out.len(), 1);
    // gap dari snapshot (7), bukan jumlah - live existing_count (10-99).
    assert_eq!(out[0].gap, 7);
    assert_eq!(out[0].recommendation, "Beku");
    assert_eq!(summary.total_existing, 3);
    assert_eq!(summary.total_gap, 7);
}

/// Barang yg tak ada di snapshot (ditambah saat revisi) jatuh ke fallback
/// stored existing_count, bukan panik / hilang.
#[test]
fn build_from_snapshot_falls_back_for_unknown_barang() {
    let snapshot_id = Uuid::new_v4();
    let extra_id = Uuid::new_v4();
    let barang = vec![
        fixture_barang(snapshot_id, "A", 5, 2),
        fixture_barang(extra_id, "B", 8, 3), // tidak ada di snapshot
    ];
    let snap = AnalisisSnapshot {
        snapshot_at: "2026-06-02T10:00:00Z".to_string(),
        summary: AnalisisSnapshotSummary {
            total_diminta: 5,
            total_existing: 2,
            total_gap: 3,
            kelayakan_persen: 40.0,
        },
        barang: vec![AnalisisSnapshotBarang {
            barang_id: snapshot_id,
            existing_count: 2,
            gap: 3,
            recommendation: "Snap".to_string(),
            existing_assets: vec![],
        }],
    };
    let (out, _) = KebutuhanBmnService::build_analisis_from_snapshot(barang, &snap);
    assert_eq!(out.len(), 2);
    let b = out.iter().find(|x| x.barang.id == extra_id).unwrap();
    // fallback: jumlah(8) - stored existing_count(3) = 5
    assert_eq!(b.gap, 5);
}

/// Gate snapshot: hanya state PASCA submit operator yg memakai snapshot.
#[test]
fn post_submit_gate_matches_expected_states() {
    use KebutuhanBmnStatus::*;
    // Masih edit → live.
    assert!(!Draft.is_post_operator_submit());
    assert!(!InputBarang.is_post_operator_submit());
    assert!(!RevisiSatker.is_post_operator_submit());
    // Sudah submit → snapshot.
    assert!(SubmitWilayah.is_post_operator_submit());
    assert!(SubmitPusat.is_post_operator_submit());
    assert!(AnalisisKelayakan.is_post_operator_submit());
    assert!(Approved.is_post_operator_submit());
    assert!(Completed.is_post_operator_submit());
}
