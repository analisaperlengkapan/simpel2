//! A penghapusan usulan names its assets by kode barang + NUP; SIMAN says the rest.
//!
//! `create` verified ONE asset — the "item utama" in the request's top-level
//! fields — and wrote the `items` array exactly as the client sent it. So the
//! other assets on a multi-item usulan could be another satker's, could not
//! exist, could carry any name, and could carry any value, including a small one
//! that kept the usulan under the Wilayah's authority ceiling. `PUT` let the
//! operator overwrite SIMAN's value on the record afterwards.
//!
//! Every test asserts what the database holds. A `201` proves nothing here: the
//! unverified create answered 201 too.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use layanan_perlengkapan::shared::db::Database;
use serde_json::{Value, json};

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";

/// The fixture's SIMAN rows for SKR001 (`tests/common.rs`):
/// `3060201003 / 15` (Pesawat Telephone, Rp 12 500 000),
/// `3060201003 / 99` (Pesawat Telephone, Rp 9 750 000),
/// `3060201004 / 16` (Telephone Mobile,  Rp 4 300 000).
const KODE_A: &str = "3060201003";
const KODE_B: &str = "3060201004";

fn auth_headers(
    role: &str,
    user_id: &str,
    satker: &str,
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
            reqwest::header::HeaderValue::from_str(satker).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user_id}::{satker}"
            ))
            .unwrap(),
        ),
    ]
}

/// The request as the form now sends it: identities only.
fn body(assets: &[(&str, &str)]) -> Value {
    let (kode, nup) = assets[0];
    json!({
        "kode_barang": kode,
        "nup": nup,
        "tanggal_penghapusan": "2026-01-01",
        "alasan": "Rusak berat dan sudah usang, tidak ekonomis untuk diperbaiki",
        "metode_penghapusan": "Pemusnahan",
        "lampiran_persyaratan": "https://storage.example.com/lampiran-persyaratan.pdf",
        "items": assets
            .iter()
            .map(|(k, n)| json!({"kode_barang": k, "nup": n}))
            .collect::<Vec<_>>(),
    })
}

async fn create(server: &TestServer, body: &Value) -> axum_test::TestResponse {
    let mut req = server.post("/penghapusan-bmn").json(body);
    for (k, v) in auth_headers("operator_satker", OPERATOR, "SKR001") {
        req = req.add_header(k, v);
    }
    req.await
}

async fn put(server: &TestServer, id: &str, body: &Value) -> axum_test::TestResponse {
    let mut req = server.put(&format!("/penghapusan-bmn/{id}")).json(body);
    for (k, v) in auth_headers("operator_satker", OPERATOR, "SKR001") {
        req = req.add_header(k, v);
    }
    req.await
}

async fn usulan_count(db: &Database) -> i64 {
    db.pool()
        .get()
        .await
        .unwrap()
        .query_one("SELECT count(*) FROM perlengkapan.penghapusan_bmn", &[])
        .await
        .unwrap()
        .get(0)
}

/// `(nup, nama_barang, nilai)` of each stored item, in order.
async fn stored_items(db: &Database, id: &str) -> Vec<(String, String, Option<f64>)> {
    let id: uuid::Uuid = id.parse().unwrap();
    db.pool()
        .get()
        .await
        .unwrap()
        .query(
            "SELECT nup, nama_barang, nilai_perolehan::float8 \
               FROM perlengkapan.penghapusan_bmn_item \
              WHERE penghapusan_id = $1 ORDER BY urutan",
            &[&id],
        )
        .await
        .unwrap()
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2)))
        .collect()
}

/// `(nama_barang, nilai, dari_backfill)` of the usulan row itself.
async fn stored_parent(db: &Database, id: &str) -> (String, Option<f64>, bool) {
    let id: uuid::Uuid = id.parse().unwrap();
    let r = db
        .pool()
        .get()
        .await
        .unwrap()
        .query_one(
            "SELECT nama_barang, nilai_perolehan::float8, nilai_perolehan_dari_backfill \
               FROM perlengkapan.penghapusan_bmn WHERE id = $1",
            &[&id],
        )
        .await
        .unwrap();
    (r.get(0), r.get(1), r.get(2))
}

/// An asset that belongs to SKR002, whose operator is not the caller.
async fn seed_other_satkers_asset(db: &Database) {
    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "INSERT INTO integrasi.siman_aset \
                (jenis_aset, nama, ur_sskel, ur_kondisi, kd_brg, no_aset, rph_aset, \
                 tgl_perlh, nama_satker, kdsatker_keu, merk) \
             VALUES ('Peralatan Mesin Non TIK', 'Milik B', 'Pesawat Telephone', 'Baik', \
                     '3060201003', '77', '50000000', '2021-03-11', \
                     'KEJAKSAAN NEGERI UJI B', '006010199666405000KP', 'Merk B')",
            &[],
        )
        .await
        .unwrap();
}

/// Naming the assets is enough; the name and the value are SIMAN's.
#[tokio::test]
async fn every_asset_on_the_usulan_carries_the_name_and_value_siman_holds() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(&server, &body(&[(KODE_A, "15"), (KODE_B, "16")])).await;
    assert_eq!(res.status_code(), 201, "create: {}", res.text());
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(
        stored_items(&db, &id).await,
        vec![
            (
                "15".to_string(),
                "Pesawat Telephone".to_string(),
                Some(12_500_000.0)
            ),
            (
                "16".to_string(),
                "Telephone Mobile".to_string(),
                Some(4_300_000.0)
            ),
        ]
    );
    // The usulan row describes the first asset, as it always has.
    assert_eq!(
        stored_parent(&db, &id).await,
        ("Pesawat Telephone".to_string(), Some(12_500_000.0), false)
    );

    teardown_test_db(&db_name).await;
}

/// A client that still sends names and values — the old form always did — is
/// not rejected. It is overruled, for the item utama and for every other item.
#[tokio::test]
async fn a_name_or_value_in_the_body_is_overruled_not_stored() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut b = body(&[(KODE_A, "15"), (KODE_B, "16")]);
    let o = b.as_object_mut().unwrap();
    o.insert("nama_barang".into(), json!("Kapal Selam"));
    o.insert("nilai_perolehan".into(), json!(1));
    let items = o.get_mut("items").unwrap().as_array_mut().unwrap();
    items[0]["nama_barang"] = json!("Kapal Selam");
    items[0]["nilai_perolehan"] = json!(1);
    items[1]["nama_barang"] = json!("Pesawat Tempur");
    items[1]["nilai_perolehan"] = json!(2);

    let res = create(&server, &b).await;
    assert_eq!(res.status_code(), 201, "create: {}", res.text());
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let stored = stored_items(&db, &id).await;
    assert_eq!(stored[0].1, "Pesawat Telephone");
    assert_eq!(stored[0].2, Some(12_500_000.0));
    // The second item is the one the old create wrote as sent.
    assert_eq!(stored[1].1, "Telephone Mobile");
    assert_eq!(stored[1].2, Some(4_300_000.0));
    assert_eq!(
        stored_parent(&db, &id).await.0,
        "Pesawat Telephone",
        "the usulan row's name is SIMAN's too"
    );

    teardown_test_db(&db_name).await;
}

/// The scope applied to the item utama never reached the rest of the list.
#[tokio::test]
async fn another_satkers_asset_anywhere_in_the_list_refuses_the_whole_usulan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_other_satkers_asset(&db).await;

    // Second position — the case the unverified list let through.
    let res = create(&server, &body(&[(KODE_A, "15"), (KODE_A, "77")])).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    let text = res.text();
    assert!(text.contains("ke-2"), "the message names the entry: {text}");
    assert_eq!(usulan_count(&db).await, 0, "nothing is stored on a refusal");

    // First position — already refused before; kept as an anchor.
    let res = create(&server, &body(&[(KODE_A, "77"), (KODE_A, "15")])).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert_eq!(usulan_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

/// An asset SIMAN has never heard of cannot be put on an SK by listing it second.
#[tokio::test]
async fn an_asset_siman_does_not_hold_refuses_the_whole_usulan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(&server, &body(&[(KODE_A, "15"), (KODE_A, "NOPE-404")])).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert!(res.text().contains("ke-2"), "{}", res.text());
    assert_eq!(usulan_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_list_that_disagrees_with_its_own_item_utama_is_refused() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut b = body(&[(KODE_A, "15"), (KODE_B, "16")]);
    // The top level names one asset; the list starts with another.
    b["nup"] = json!("99");
    let res = create(&server, &b).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert_eq!(usulan_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_same_asset_listed_twice_is_refused() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(
        &server,
        &body(&[(KODE_A, "15"), (KODE_B, "16"), (KODE_A, "15")]),
    )
    .await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert_eq!(usulan_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

/// Restores the ceiling variable even when an assertion panics, so a failure
/// here cannot leave every later test running under a ceiling.
struct CeilingGuard;
impl CeilingGuard {
    fn set(rupiah: &str) -> Self {
        // SAFETY: no other test in this binary reads this variable, and none
        // files a WILAYAH usulan, so no concurrent reader exists.
        unsafe { std::env::set_var("PENGHAPUSAN_WILAYAH_MAX_NILAI", rupiah) };
        Self
    }
}
impl Drop for CeilingGuard {
    fn drop(&mut self) {
        // SAFETY: see `set`.
        unsafe { std::env::remove_var("PENGHAPUSAN_WILAYAH_MAX_NILAI") };
    }
}

/// The ceiling saw the item utama's value alone, so a small first asset carried
/// any number of large ones past the Wilayah's authority.
#[tokio::test]
async fn the_wilayah_ceiling_is_judged_on_the_whole_usulan() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let _ceiling = CeilingGuard::set("15000000");

    let wilayah = |mut b: Value| {
        b["kewenangan_penetap_sk"] = json!("WILAYAH");
        b
    };

    // 12.5M alone is under the 15M line.
    let res = create(&server, &wilayah(body(&[(KODE_A, "15")]))).await;
    assert_eq!(
        res.status_code(),
        201,
        "one asset under the line: {}",
        res.text()
    );

    // 4.3M first (under the line), then 12.5M: each alone is allowed, the
    // package is 16.8M. The old check looked at the first value only.
    let res = create(&server, &wilayah(body(&[(KODE_B, "16"), (KODE_A, "15")]))).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert!(res.text().contains("melebihi"), "{}", res.text());
    assert_eq!(usulan_count(&db).await, 1, "only the first usulan exists");

    // The PUSAT route is never the violation.
    let res = create(&server, &body(&[(KODE_B, "16"), (KODE_A, "15")])).await;
    assert_eq!(res.status_code(), 201, "{}", res.text());

    teardown_test_db(&db_name).await;
}

/// A PUT could overwrite SIMAN's figure with anything, after the create-time
/// ceiling had been applied to the real one.
#[tokio::test]
async fn an_edit_cannot_overwrite_the_siman_value() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(&server, &body(&[(KODE_A, "15")])).await;
    assert_eq!(res.status_code(), 201, "{}", res.text());
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = put(
        &server,
        &id,
        &json!({
            "alasan": "Alasan yang diperbarui oleh operator satker",
            "nilai_perolehan": 1
        }),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let (_, nilai, _) = stored_parent(&db, &id).await;
    assert_eq!(nilai, Some(12_500_000.0), "the PUT's value was believed");

    teardown_test_db(&db_name).await;
}

/// The one legitimate reason to touch the value: a back-filled row flagged for
/// verification. SIMAN is re-read; the operator's typing is not involved.
#[tokio::test]
async fn an_edit_re_reads_a_backfilled_value_from_siman_and_clears_the_flag() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(&server, &body(&[(KODE_A, "15")])).await;
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let uid: uuid::Uuid = id.parse().unwrap();

    // What the legacy back-fill left: a `nilai_residu`-derived figure and the flag.
    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "UPDATE perlengkapan.penghapusan_bmn \
                SET nilai_perolehan = 7, nilai_perolehan_dari_backfill = TRUE WHERE id = $1",
            &[&uid],
        )
        .await
        .unwrap();

    let res = put(
        &server,
        &id,
        &json!({"alasan": "Alasan yang diperbarui oleh operator satker"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let (_, nilai, backfill) = stored_parent(&db, &id).await;
    assert_eq!(nilai, Some(12_500_000.0));
    assert!(!backfill, "the flag clears once SIMAN has answered");

    teardown_test_db(&db_name).await;
}

/// If SIMAN cannot answer, the flagged row stays flagged — nothing untrusted
/// is stored in its place.
#[tokio::test]
async fn a_backfilled_row_stays_flagged_when_siman_has_no_figure() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create(&server, &body(&[(KODE_A, "15")])).await;
    let id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let uid: uuid::Uuid = id.parse().unwrap();

    let client = db.pool().get().await.unwrap();
    client
        .execute(
            "UPDATE perlengkapan.penghapusan_bmn \
                SET nilai_perolehan = 7, nilai_perolehan_dari_backfill = TRUE WHERE id = $1",
            &[&uid],
        )
        .await
        .unwrap();
    client
        .execute(
            "UPDATE integrasi.siman_aset SET rph_aset = NULL \
              WHERE kd_brg = '3060201003' AND no_aset = '15'",
            &[],
        )
        .await
        .unwrap();
    drop(client);

    let res = put(
        &server,
        &id,
        &json!({"alasan": "Alasan yang diperbarui oleh operator satker"}),
    )
    .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());

    let (_, nilai, backfill) = stored_parent(&db, &id).await;
    assert_eq!(nilai, Some(7.0), "the figure is left as it was");
    assert!(backfill, "still waiting for a real verification");

    teardown_test_db(&db_name).await;
}

/// An asset with no value in SIMAN cannot be placed on an authority, so the
/// usulan is refused with a message that says so — it used to read "not found".
#[tokio::test]
async fn an_asset_with_no_siman_value_is_refused_for_what_it_is() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "UPDATE integrasi.siman_aset SET rph_aset = NULL \
              WHERE kd_brg = '3060201003' AND no_aset = '15'",
            &[],
        )
        .await
        .unwrap();

    let res = create(&server, &body(&[(KODE_A, "15")])).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert!(
        res.text().contains("nilai perolehan"),
        "the refusal names the missing figure: {}",
        res.text()
    );
    assert_eq!(usulan_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}
