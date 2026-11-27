//! Whoever's move is next is told that work is waiting, in every module, and wherever they happened
//! to open the app.
//!
//! The pakaian dinas flow already told the next role (`pakaian_dinas_notifikasi_test`). The generic
//! workflow engine did not: its approver lookup was a placeholder that returned nobody and it matched
//! on state names no workflow has, so a penghapusan, a pemakaian or a kebutuhan sat in a list until
//! the next role happened to open it; Kebutuhan BMN's per-satker moves never reached the engine's
//! notifications at all. Recipients are the users the service has seen (`pengguna_dikenal`), and until
//! now only the pakaian dinas screens recorded anybody.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::json;

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const WILAYAH: &str = "00000000-0000-0000-0000-000000000002";
const PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const WILAYAH_LAIN: &str = "00000000-0000-0000-0000-000000000004";
/// A SECOND pusat validator: the actor of a pusat move must not be told about their own move, but
/// the other one must.
const PUSAT_LAIN: &str = "00000000-0000-0000-0000-000000000005";

const SATKER: &str = "SKR001";
/// The Kejati the fixture puts above SKR001 (KJT02 sits above SKR003).
const KEJATI: &str = "KJT01";
const KEJATI_LAIN: &str = "KJT02";

fn auth_headers(
    role: &str,
    user_id: &str,
    satker_id: &str,
) -> Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)> {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_str(user_id).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_str(role).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_str(satker_id).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user_id}::{satker_id}"
            ))
            .unwrap(),
        ),
    ]
}

/// `user` is known to the service with `role` in `satker`, exactly as the registry records them.
async fn dikenal(
    db: &layanan_perlengkapan::shared::db::Database,
    user: &str,
    role: &str,
    satker: &str,
) {
    layanan_perlengkapan::shared::pengguna::catat(
        db.pool(),
        user.parse().unwrap(),
        None,
        &[role.to_string()],
        Some(satker),
    )
    .await
    .unwrap();
}

/// (type, title, message, link) of everything `user` was told, oldest first.
async fn kotak_masuk(
    db: &layanan_perlengkapan::shared::db::Database,
    user: &str,
) -> Vec<(String, String, String, Option<String>)> {
    let client = db.pool().get().await.unwrap();
    client
        .query(
            "SELECT notification_type, title, message, action_url \
               FROM notifikasi.in_app_notifications \
              WHERE user_id = $1::text::uuid ORDER BY created_at, id",
            &[&user],
        )
        .await
        .unwrap()
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3)))
        .collect()
}

fn menunggu(
    kotak: &[(String, String, String, Option<String>)],
) -> Vec<&(String, String, String, Option<String>)> {
    kotak
        .iter()
        .filter(|n| n.0 == "workflow.approval_required")
        .collect()
}

async fn buat_usulan(server: &TestServer) -> String {
    let mut req = server.post("/penghapusan-bmn").json(&json!({
        "kode_barang": "3060201003",
        "nama_barang": "Laptop Dell Latitude 5520",
        "nup": "15",
        "tanggal_penghapusan": "2026-01-01",
        "alasan": "Rusak berat dan sudah usang, tidak ekonomis untuk diperbaiki",
        "metode_penghapusan": "Pemusnahan",
        "nilai_perolehan": 0,
        "lampiran_persyaratan": "https://storage.example.com/lampiran-persyaratan.pdf"
    }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn aksi(server: &TestServer, role: &str, user: &str, satker: &str, jalur: &str) {
    let mut req = server
        .post(jalur)
        .json(&json!({"aksi": "forward", "catatan": "uji"}));
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{jalur}: {:?}", res.text());
}

#[tokio::test]
async fn penghapusan_diajukan_memberi_tahu_validator_wilayah_di_wilayahnya_saja() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    dikenal(&db, OPERATOR, "operator_satker", SATKER).await;
    dikenal(&db, WILAYAH, "validator_wilayah", KEJATI).await;
    dikenal(&db, WILAYAH_LAIN, "validator_wilayah", KEJATI_LAIN).await;
    dikenal(&db, PUSAT, "validator_pusat", "PUSAT001").await;
    dikenal(&db, PUSAT_LAIN, "validator_pusat", "PUSAT001").await;

    let id = buat_usulan(&server).await;
    aksi(
        &server,
        "operator_satker",
        OPERATOR,
        SATKER,
        &format!("/penghapusan-bmn/{id}/submit-wilayah"),
    )
    .await;

    // The wilayah validator above the satker is told, with a way to the usulan.
    let w = kotak_masuk(&db, WILAYAH).await;
    let m = menunggu(&w);
    assert_eq!(m.len(), 1, "wilayah told that work is waiting: {w:?}");
    assert_eq!(m[0].1, "Perlu tindakan Anda: Penghapusan BMN");
    assert!(
        m[0].2.contains("Menunggu tindakan dari Validator Wilayah"),
        "{}",
        m[0].2
    );
    assert_eq!(
        m[0].3.as_deref(),
        Some(format!("/pengelolaan/penghapusan/detail/{id}").as_str()),
        "the notification leads to the usulan"
    );

    // Not the one of another wilayah, not pusat (not its turn yet), and the sender is not told
    // to act on what they just sent: they get the outcome notice only.
    assert!(menunggu(&kotak_masuk(&db, WILAYAH_LAIN).await).is_empty());
    assert!(
        kotak_masuk(&db, PUSAT).await.is_empty(),
        "pusat's turn is not yet"
    );
    let o = kotak_masuk(&db, OPERATOR).await;
    assert!(
        menunggu(&o).is_empty(),
        "the sender is not asked to act: {o:?}"
    );
    assert_eq!(
        o.len(),
        1,
        "the requester still learns the usulan moved: {o:?}"
    );

    // Wilayah forwards: now it is pusat's turn, and only the OTHER pusat validator is told
    // (the actor is a pusat-side user in no case here; both known pusat validators are told).
    aksi(
        &server,
        "validator_wilayah",
        WILAYAH,
        KEJATI,
        &format!("/penghapusan-bmn/{id}/forward-pusat"),
    )
    .await;
    for p in [PUSAT, PUSAT_LAIN] {
        let k = kotak_masuk(&db, p).await;
        assert_eq!(menunggu(&k).len(), 1, "pusat validator {p} told: {k:?}");
    }
    // The wilayah validator who forwarded is not told to act on their own move.
    assert_eq!(
        menunggu(&kotak_masuk(&db, WILAYAH).await).len(),
        1,
        "still only the first: the forward is not announced to the one who made it"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn kebutuhan_per_satker_memberi_tahu_peran_berikutnya() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    dikenal(&db, WILAYAH, "validator_wilayah", KEJATI).await;
    dikenal(&db, WILAYAH_LAIN, "validator_wilayah", KEJATI_LAIN).await;
    dikenal(&db, PUSAT, "validator_pusat", "PUSAT001").await;

    // Pusat composes the campaign and adds the satker.
    let mut req = server.post("/kebutuhan-bmn/pengajuan").json(&json!({
        "nama": "Kebutuhan BMN 2026", "tahun": 2026,
        "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"
    }));
    for (k, v) in auth_headers("validator_pusat", PUSAT, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let body: serde_json::Value = res.json();
    let pengajuan_id = body["data"]["pengajuan"]["id"]
        .as_str()
        .or(body["data"]["id"].as_str())
        .unwrap()
        .to_string();
    let mut req = server
        .post(&format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"))
        .json(&json!({"satker_id": SATKER, "satker_name": "Kejaksaan Negeri Jakarta Selatan"}));
    for (k, v) in auth_headers("validator_pusat", PUSAT, "PUSAT001") {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let satker_row = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // The operator fills in and submits: the wilayah validator above the satker is told, naming
    // the satker; the other wilayah's validator and pusat are not.
    let mut req = server
        .post(&format!("/kebutuhan-bmn/satker/{satker_row}/barang"))
        .json(&json!({"nama": "Laptop", "jumlah": 5, "satuan": "Unit"}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    assert_eq!(req.await.status_code(), 201);
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{satker_row}/submit-wilayah"
        ))
        .json(&json!({
            "catatan_satker": "Mohon persetujuan",
            "lampiran_surat_permohonan": "https://storage.example.com/surat-permohonan.pdf"
        }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    let w = kotak_masuk(&db, WILAYAH).await;
    let m = menunggu(&w);
    assert_eq!(m.len(), 1, "kebutuhan satker move told to wilayah: {w:?}");
    assert_eq!(m[0].1, "Perlu tindakan Anda: Kebutuhan BMN");
    assert!(
        m[0].2.contains("Menunggu tindakan dari Validator Wilayah"),
        "{}",
        m[0].2
    );
    assert_eq!(
        m[0].3.as_deref(),
        Some(format!("/kebutuhan-bmn/detail/{pengajuan_id}").as_str())
    );
    assert!(menunggu(&kotak_masuk(&db, WILAYAH_LAIN).await).is_empty());
    assert!(
        menunggu(&kotak_masuk(&db, PUSAT).await).is_empty(),
        "not pusat's turn yet"
    );

    // The wilayah validator forwards: pusat's turn.
    let mut req = server
        .post(&format!(
            "/kebutuhan-bmn/satker/{satker_row}/validator-wilayah"
        ))
        .json(&json!({"aksi": "forward", "catatan": "lengkap"}));
    for (k, v) in auth_headers("validator_wilayah", WILAYAH, KEJATI) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    assert_eq!(
        menunggu(&kotak_masuk(&db, PUSAT).await).len(),
        1,
        "forwarded to pusat, pusat told"
    );

    teardown_test_db(&db_name).await;
}

/// Every authenticated caller becomes somebody a handover can reach, from whatever screen they
/// landed on. Only four pakaian dinas handlers recorded anybody, so a validator who had opened the
/// dashboard and nothing else could never be told that work was waiting for them.
#[tokio::test]
async fn setiap_pemanggil_terautentikasi_menjadi_dikenal_dari_layar_mana_pun() {
    use axum::{
        Router,
        extract::Request,
        middleware::{Next, from_fn, from_fn_with_state},
        routing::get,
    };
    use layanan_perlengkapan::shared::middleware::Claims;
    use layanan_perlengkapan::shared::pengguna::{PenggunaRegistry, catat_pengguna_middleware};

    let (_app, db, db_name) = setup_test_app().await;
    let registry = PenggunaRegistry::new(db.pool().clone());
    let user: uuid::Uuid = WILAYAH.parse().unwrap();
    let mut claims = Claims::with_roles(user, "budi", ["validator_wilayah"]);
    claims.satker_code = Some(KEJATI.to_string());

    // `require_authentication` puts the claims in the request; this stands in for it, around a
    // route that has nothing to do with pakaian dinas.
    let app = Router::new()
        .route("/apa-saja", get(|| async { "ok" }))
        .layer(from_fn_with_state(registry, catat_pengguna_middleware))
        .layer(from_fn(move |mut req: Request, next: Next| {
            let c = claims.clone();
            async move {
                req.extensions_mut().insert(c);
                next.run(req).await
            }
        }));
    let server = TestServer::new(app);
    assert_eq!(server.get("/apa-saja").await.status_code(), 200);

    // The write is detached from the request, so give it a moment.
    let mut baris = Vec::new();
    for _ in 0..40 {
        let client = db.pool().get().await.unwrap();
        baris = client
            .query(
                "SELECT roles, satker_code FROM perlengkapan.pengguna_dikenal WHERE user_id = $1",
                &[&user],
            )
            .await
            .unwrap();
        if !baris.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert_eq!(
        baris.len(),
        1,
        "the caller was recorded without opening pakaian dinas"
    );
    assert_eq!(
        baris[0].get::<_, Vec<String>>(0),
        vec!["validator_wilayah".to_string()]
    );
    assert_eq!(
        baris[0].get::<_, Option<String>>(1).as_deref(),
        Some(KEJATI)
    );

    // And from that moment a handover reaches them.
    let kode = layanan_perlengkapan::shared::pengguna::penerima_peran(
        db.pool(),
        "validator_wilayah",
        SATKER,
    )
    .await
    .unwrap();
    assert_eq!(kode, vec![user]);

    teardown_test_db(&db_name).await;
}

/// Pemakaian BMN stays inside the satker: its validator and then its approver are told, in turn,
/// and the same roles of another satker are not. (A satker-bound role used to fall into the lookup's
/// "every holder in the country" arm.)
#[tokio::test]
async fn pemakaian_memberi_tahu_validator_lalu_approver_satkernya_saja() {
    const VALIDATOR_SATKER: &str = "00000000-0000-0000-0000-0000000000b1";
    const APPROVER_SATKER: &str = "00000000-0000-0000-0000-0000000000b2";
    const VALIDATOR_LAIN: &str = "00000000-0000-0000-0000-0000000000b3";
    const APPROVER_LAIN: &str = "00000000-0000-0000-0000-0000000000b4";
    const SATKER_LAIN: &str = "SKR003";

    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    dikenal(&db, VALIDATOR_SATKER, "validator_satker", SATKER).await;
    dikenal(&db, APPROVER_SATKER, "approver_satker", SATKER).await;
    dikenal(&db, VALIDATOR_LAIN, "validator_satker", SATKER_LAIN).await;
    dikenal(&db, APPROVER_LAIN, "approver_satker", SATKER_LAIN).await;

    let mut req = server.post("/pemakaian-bmn").json(&json!({
        "pegawai_nip": "19800101000000001",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": [{
            "jenis_bmn": "LAPTOP", "bmn_nup": "9001",
            "bmn_kode_barang": "3060201003", "bmn_nama_barang": "Laptop Dell Latitude 5520",
            "bmn_merk": "Dell", "serial_number": "SN-12345678"
        }]
    }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Submitted: the satker's validator is told, not the approver yet, not another satker's.
    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/transition"))
        .json(&json!({"target_status": "SUBMITTED", "catatan": "Mohon validasi"}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    assert_eq!(req.await.status_code(), 200);
    let v = kotak_masuk(&db, VALIDATOR_SATKER).await;
    let m = menunggu(&v);
    assert_eq!(m.len(), 1, "the satker's validator is told: {v:?}");
    assert!(m[0].2.contains("Validator Satker"), "{}", m[0].2);
    assert_eq!(
        m[0].3.as_deref(),
        Some(format!("/pengelolaan/pemakaian/detail/{id}").as_str())
    );
    assert!(
        menunggu(&kotak_masuk(&db, APPROVER_SATKER).await).is_empty(),
        "not the approver yet"
    );
    assert!(
        menunggu(&kotak_masuk(&db, VALIDATOR_LAIN).await).is_empty(),
        "not another satker's"
    );
    assert!(menunggu(&kotak_masuk(&db, APPROVER_LAIN).await).is_empty());

    // Forwarded: now the approver of that satker.
    let mut req = server
        .post(&format!("/pemakaian-bmn/{id}/validator-satker-action"))
        .json(&json!({"action": "forward", "expected_version": 1, "catatan": "Diteruskan"}));
    for (k, v) in auth_headers("validator_satker", VALIDATOR_SATKER, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    assert_eq!(
        menunggu(&kotak_masuk(&db, APPROVER_SATKER).await).len(),
        1,
        "the approver of the satker is told"
    );
    assert!(menunggu(&kotak_masuk(&db, APPROVER_LAIN).await).is_empty());
    assert_eq!(
        menunggu(&kotak_masuk(&db, VALIDATOR_SATKER).await).len(),
        1,
        "the validator who forwarded is not told to act on their own move"
    );

    teardown_test_db(&db_name).await;
}

/// A ticket reaches the helpdesk, not only the person who filed it, and the link in the
/// notification opens the page that exists.
#[tokio::test]
async fn tiket_bantuan_baru_dan_balasan_pelapor_memberi_tahu_petugas_helpdesk() {
    const ADMIN: &str = "00000000-0000-0000-0000-0000000000c1";
    const ADMIN_LAIN: &str = "00000000-0000-0000-0000-0000000000c2";

    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    dikenal(&db, ADMIN, "admin", "PUSAT001").await;
    dikenal(&db, ADMIN_LAIN, "admin", "PUSAT001").await;
    dikenal(&db, OPERATOR, "operator_satker", SATKER).await;

    let mut req = server.post("/bantuan/tiket").json(&json!({
        "subject": "Tidak bisa mengisi ukuran", "description": "Tombol simpan tidak merespons"
    }));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let id = res.json::<serde_json::Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // Notifications to the helpdesk are sent from a detached task: wait for them.
    let tunggu = |user: &'static str, event: &'static str| {
        let db = db.clone();
        async move {
            for _ in 0..40 {
                let k = kotak_masuk(&db, user).await;
                if k.iter().any(|n| n.0 == event) {
                    return k;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            kotak_masuk(&db, user).await
        }
    };
    for staff in [ADMIN, ADMIN_LAIN] {
        let k = tunggu(staff, "tiket.baru").await;
        let baru: Vec<_> = k.iter().filter(|n| n.0 == "tiket.baru").collect();
        assert_eq!(baru.len(), 1, "every helpdesk staff is told: {k:?}");
        assert_eq!(
            baru[0].3.as_deref(),
            Some("/bantuan/helpdesk"),
            "a page that exists"
        );
    }
    let o = tunggu(OPERATOR, "tiket.dibuat").await;
    assert!(
        o.iter().any(|n| n.0 == "tiket.dibuat"),
        "the reporter is told it was received"
    );
    assert!(
        !o.iter().any(|n| n.0 == "tiket.baru"),
        "the reporter is not told to handle their own ticket"
    );

    // The reporter answers: it is the helpdesk's move again.
    let mut req = server
        .post(&format!("/bantuan/tiket/{id}/komentar"))
        .json(&json!({"content": "Sudah saya coba ulang, masih sama"}));
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    assert!(res.status_code().is_success(), "{:?}", res.text());
    let k = tunggu(ADMIN, "tiket.balasan_pelapor").await;
    assert!(
        k.iter().any(|n| n.0 == "tiket.balasan_pelapor"),
        "the reporter's reply reaches the helpdesk: {k:?}"
    );

    teardown_test_db(&db_name).await;
}
