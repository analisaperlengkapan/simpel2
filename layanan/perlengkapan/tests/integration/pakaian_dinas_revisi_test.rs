//! Revisi: setiap pengembalian harus punya jalan pulang.
//!
//! Validator pusat menekan "Kembalikan ke Wilayah" → satker berstatus 1007
//! (Revisi Wilayah). Status itu TIDAK punya langkah keluar: tak ada transisi
//! yang meninggalkannya, `STATUS_DAPAT_DIUBAH` tak memuatnya, dan layar tak
//! menawarkan tombol apa pun kepada siapa pun. Satker yang dikembalikan Pusat
//! karenanya terparkir selamanya, dan kampanyenya tak pernah bisa 100% Selesai.
//!
//! Tes alur yang ada hanya menempuh jalan bahagia (1000 → 1001 → 1004 → 1008),
//! jadi tak satu pun menyentuh sisi revisi. Dua tes di sini menempuh KEDUA jalan
//! keluar dari 1007, lewat HTTP seperti yang dilakukan aplikasi:
//!
//!   - Wilayah meneruskan lagi ke Pusat (yang diminta Pusat tak butuh data baru),
//!   - Wilayah mengembalikan ke operator satker, satu-satunya peran yang boleh
//!     mengubah ukuran, lalu operator memperbaiki dan mengajukan ulang.

use super::pakaian_dinas_pengisian_test::{
    NIP_A, OPERATOR, PUSAT, SATKER_A, buat_kampanye, get, post, put,
};
use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

/// Satu langkah workflow oleh `role`.
async fn langkah(
    server: &TestServer,
    row: &str,
    role: &str,
    aksi: &str,
) -> axum_test::TestResponse {
    // Pelaku apa pun cukup: yang diuji adalah (status, aksi, peran), bukan identitas.
    let user = if role == "operator_satker" {
        OPERATOR
    } else {
        PUSAT
    };
    post(
        server,
        "/pakaian-dinas/validator-action",
        role,
        user,
        SATKER_A,
        json!({"pengajuan_satker_id": row, "aksi": aksi, "komentar": format!("{role}: {aksi}")}),
    )
    .await
}

/// Lakukan langkah yang HARUS berhasil dan pastikan status barunya.
async fn harus(server: &TestServer, row: &str, role: &str, aksi: &str, status_baru: i64) {
    let res = langkah(server, row, role, aksi).await;
    assert_eq!(res.status_code(), 200, "{role} {aksi}: {:?}", res.text());
    assert_eq!(
        res.json::<Value>()["data"]["aktivitas_id"].as_i64(),
        Some(status_baru),
        "{role} {aksi} harus berujung di {status_baru}"
    );
}

/// Lakukan langkah yang HARUS ditolak; statusnya tidak boleh bergeser.
async fn tertolak(server: &TestServer, row: &str, role: &str, aksi: &str) {
    let res = langkah(server, row, role, aksi).await;
    assert_eq!(
        res.status_code(),
        400,
        "{role} {aksi} seharusnya ditolak: {:?}",
        res.text()
    );
}

async fn dapat_diubah(server: &TestServer, pengajuan_id: &str) -> bool {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "roster: {:?}", res.text());
    res.json::<Value>()["data"]["dapat_diubah"]
        .as_bool()
        .expect("roster memuat dapat_diubah")
}

async fn simpan_ukuran(
    server: &TestServer,
    pengajuan_id: &str,
    pakaian_id: &str,
    ukuran: &str,
) -> axum_test::TestResponse {
    put(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai/{NIP_A}"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
        json!({"with_hijab": false, "ukuran": [{"pakaian_id": pakaian_id, "ukuran": ukuran}]}),
    )
    .await
}

/// Id baris satker (`pengajuan_pakaian_dinas_satker`) milik SATKER_A.
async fn baris_satker(server: &TestServer, pengajuan_id: &str) -> String {
    let res = get(
        server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "daftar satker: {:?}", res.text());
    res.json::<Value>()["data"][0]["id"]
        .as_str()
        .expect("kampanye punya baris untuk satker operator")
        .to_string()
}

/// Status yang pernah dilalui baris itu, dari yang terlama.
async fn riwayat(server: &TestServer, row: &str) -> Vec<i64> {
    let res = get(
        server,
        &format!("/pakaian-dinas/satker/{row}/aktivitas"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "riwayat: {:?}", res.text());
    res.json::<Value>()["data"]
        .as_array()
        .expect("riwayat berupa daftar")
        .iter()
        .map(|a| a["aktivitas_id"].as_i64().unwrap())
        .collect()
}

#[tokio::test]
async fn pusat_mengembalikan_wilayah_meneruskan_lagi_sampai_selesai() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let row = baris_satker(&server, &pengajuan_id).await;

    assert_eq!(
        simpan_ukuran(&server, &pengajuan_id, &pakaian_id, "L")
            .await
            .status_code(),
        200
    );
    harus(&server, &row, "operator_satker", "submit", 1001).await;
    harus(&server, &row, "validator_wilayah", "approve", 1004).await;

    // Pusat mengembalikan. Sebelum perbaikan, di sinilah baris itu berhenti.
    harus(&server, &row, "validator_pusat", "reject", 1007).await;

    // Yang boleh menjawab hanya Wilayah — bukan operator (melompati Wilayah)
    // dan bukan Pusat (menjawab pengembaliannya sendiri).
    tertolak(&server, &row, "operator_satker", "submit").await;
    tertolak(&server, &row, "validator_pusat", "approve").await;
    tertolak(&server, &row, "validator_pusat", "reject").await;
    // Terparkir di Wilayah: operator belum boleh mengubah apa pun.
    assert!(!dapat_diubah(&server, &pengajuan_id).await);

    harus(&server, &row, "validator_wilayah", "approve", 1004).await;
    harus(&server, &row, "validator_pusat", "approve", 1008).await;

    assert_eq!(
        riwayat(&server, &row).await,
        [1001, 1004, 1007, 1004, 1008],
        "setiap langkah tercatat, termasuk pengembaliannya"
    );
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pusat_mengembalikan_wilayah_menurunkan_ke_satker_yang_memperbaiki() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (pengajuan_id, pakaian_id) = buat_kampanye(&server).await;
    let row = baris_satker(&server, &pengajuan_id).await;

    assert_eq!(
        simpan_ukuran(&server, &pengajuan_id, &pakaian_id, "L")
            .await
            .status_code(),
        200
    );
    harus(&server, &row, "operator_satker", "submit", 1001).await;

    // Revisi tingkat Wilayah: satker memperbaiki dan mengajukan ulang.
    harus(&server, &row, "validator_wilayah", "reject", 1003).await;
    assert!(
        dapat_diubah(&server, &pengajuan_id).await,
        "dikembalikan = terbuka untuk diperbaiki"
    );
    assert_eq!(
        simpan_ukuran(&server, &pengajuan_id, &pakaian_id, "XL")
            .await
            .status_code(),
        200
    );
    harus(&server, &row, "operator_satker", "submit", 1001).await;
    harus(&server, &row, "validator_wilayah", "approve", 1004).await;

    // Revisi tingkat Pusat. Perubahan DATA hanya bisa dilakukan operator
    // (`require_role("operator_satker")`), jadi Wilayah harus bisa menurunkannya.
    harus(&server, &row, "validator_pusat", "reject", 1007).await;
    assert!(!dapat_diubah(&server, &pengajuan_id).await);
    assert_eq!(
        simpan_ukuran(&server, &pengajuan_id, &pakaian_id, "S")
            .await
            .status_code(),
        400,
        "selama terparkir di Wilayah, isinya terkunci"
    );
    harus(&server, &row, "validator_wilayah", "reject", 1003).await;
    assert!(dapat_diubah(&server, &pengajuan_id).await);
    assert_eq!(
        simpan_ukuran(&server, &pengajuan_id, &pakaian_id, "M")
            .await
            .status_code(),
        200
    );

    harus(&server, &row, "operator_satker", "submit", 1001).await;
    harus(&server, &row, "validator_wilayah", "approve", 1004).await;
    harus(&server, &row, "validator_pusat", "approve", 1008).await;

    // Perbaikan terakhir itulah yang sampai ke Pusat, bukan ukuran pertama.
    let res = get(
        &server,
        &format!("/pakaian-dinas/pengajuan/{pengajuan_id}/satker/{SATKER_A}/pegawai"),
        "operator_satker",
        OPERATOR,
        SATKER_A,
    )
    .await;
    let pegawai = &res.json::<Value>()["data"]["pegawai"];
    let ukuran = pegawai
        .as_array()
        .and_then(|p| p.iter().find(|x| x["nip"] == NIP_A))
        .map(|x| x["ukuran"].to_string())
        .unwrap_or_default();
    assert!(
        ukuran.contains("\"M\""),
        "ukuran terakhir (M) yang tersimpan: {ukuran}"
    );

    assert_eq!(
        riwayat(&server, &row).await,
        [1001, 1003, 1001, 1004, 1007, 1003, 1001, 1004, 1008],
        "dua putaran revisi, semuanya tercatat berurutan"
    );
    teardown_test_db(&db_name).await;
}
