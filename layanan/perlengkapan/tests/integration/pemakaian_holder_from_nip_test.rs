//! A permit names its holder by NIP alone; everything else comes from MySIMKARI.
//!
//! The create form used to type the employee's name, rank, posting and satker
//! into the request. Two things were wrong with that, and each is a test below:
//!
//! 1. **It could not be submitted at all.** The satker was read from a JWT claim
//!    (`satker_id`) that authenc never issues, so the form stopped itself with
//!    "satker_id kosong" for every user. No spec pressed the button, so nothing
//!    noticed.
//! 2. **Whatever it sent was believed.** `pegawai_satker_id` was a client UUID
//!    with no foreign key, `pegawai_nama` was whatever the body said, and an
//!    operator could file a permit for any NIP in the country.
//!
//! These tests assert what the database holds, not merely `201`: a create that
//! stored the body's claims would answer 201 too.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const VALIDATOR_SATKER: &str = "00000000-0000-0000-0000-000000000002";
const APPROVER_SATKER: &str = "00000000-0000-0000-0000-000000000003";

/// The fixture's employees (`tests/common.rs`): one per satker.
const NIP_SKR001: &str = "19800101000000001";
const NIP_SKR002: &str = "19800101000000002";
const NIP_SKR003: &str = "19800101000000003";

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

fn item(nup: &str) -> Value {
    json!({
        "jenis_bmn": "LAPTOP",
        "bmn_nup": nup,
        "bmn_kode_barang": "3060201003",
        "bmn_nama_barang": "Laptop Dell Latitude 5520",
        "bmn_merk": "Dell",
        "serial_number": format!("SN-{nup}")
    })
}

/// The request as the form now sends it: the NIP, the period, the reason, the
/// assets. No name, no rank, no satker.
fn body_for(nip: &str, nup: &str) -> Value {
    json!({
        "pegawai_nip": nip,
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Digunakan untuk pelaksanaan tugas kedinasan sehari-hari",
        "items": [item(nup)]
    })
}

async fn create_as(server: &TestServer, satker: &str, body: &Value) -> axum_test::TestResponse {
    let mut req = server.post("/pemakaian-bmn").json(body);
    for (k, v) in auth_headers("operator_satker", OPERATOR, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

async fn post_as(
    server: &TestServer,
    path: &str,
    role: &str,
    user: &str,
    satker: &str,
    body: Value,
) -> Value {
    let mut req = server.post(path).json(&body);
    for (k, v) in auth_headers(role, user, satker) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    let (status, teks) = (res.status_code(), res.text());
    assert!(status.is_success(), "POST {path} -> {status}: {teks}");
    serde_json::from_str(&teks).unwrap()
}

/// The satker surrogate the SERVER derives, asked of Postgres itself so the
/// test does not restate the formula it is checking.
async fn satker_uuid_of(db: &layanan_perlengkapan::shared::db::Database, kode: &str) -> String {
    let client = db.pool().get().await.unwrap();
    client
        .query_one(
            "SELECT md5('simpel:satker:' || $1::text)::uuid::text",
            &[&kode],
        )
        .await
        .unwrap()
        .get::<_, String>(0)
}

async fn permit_count(db: &layanan_perlengkapan::shared::db::Database) -> i64 {
    let client = db.pool().get().await.unwrap();
    client
        .query_one("SELECT count(*) FROM perlengkapan.izin_pemakaian_bmn", &[])
        .await
        .unwrap()
        .get(0)
}

/// Naming a NIP is enough, and the permit then says what MySIMKARI says.
#[tokio::test]
async fn a_permit_filed_with_only_a_nip_carries_the_employee_from_the_database() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Give the fixture employee the columns it is seeded without, so rank and
    // photo are measured rather than vacuously `None == None`.
    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "UPDATE integrasi.mysimkari_pegawai \
                SET golpang = 'Jaksa Madya / (IV/a)', foto = 'foto-uji-a.jpg' \
              WHERE nip = $1",
            &[&NIP_SKR001],
        )
        .await
        .unwrap();

    let res = create_as(&server, "SKR001", &body_for(NIP_SKR001, "701")).await;
    assert_eq!(res.status_code(), 201, "create: {}", res.text());
    let data = res.json::<Value>()["data"].clone();

    assert_eq!(data["pegawai_nip"], NIP_SKR001);
    assert_eq!(data["pegawai_nama"], "Pegawai Uji A");
    assert_eq!(data["pegawai_jabatan"], "Staf");
    assert_eq!(data["pegawai_satker_nama"], "KEJAKSAAN NEGERI UJI A");
    assert_eq!(data["pegawai_unit_kerja"], "KEJAKSAAN NEGERI UJI A");
    // The whole `golpang` value goes in as pangkat; splitting it is a guess.
    assert_eq!(data["pegawai_pangkat"], "Jaksa Madya / (IV/a)");
    assert!(data["pegawai_golongan"].is_null());
    assert_eq!(data["foto_pegawai"], "foto-uji-a.jpg");
    assert_eq!(data["satker_code"], "SKR001");
    assert_eq!(
        data["pegawai_satker_id"].as_str().unwrap(),
        satker_uuid_of(&db, "SKR001").await,
        "pegawai_satker_id is derived from the employee's satker, not supplied"
    );

    teardown_test_db(&db_name).await;
}

/// The body can no longer tell the server who the employee is. A client that
/// still sends the old fields is not rejected — it is overruled.
#[tokio::test]
async fn what_the_body_claims_about_the_employee_is_ignored() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let mut body = body_for(NIP_SKR001, "702");
    let obj = body.as_object_mut().unwrap();
    obj.insert("pegawai_nama".into(), json!("Tokoh Karangan"));
    obj.insert("pegawai_jabatan".into(), json!("Jaksa Agung"));
    obj.insert("pegawai_pangkat".into(), json!("Jenderal"));
    obj.insert("pegawai_satker_nama".into(), json!("Satker Karangan"));
    obj.insert(
        "pegawai_satker_id".into(),
        json!("99999999-9999-4999-8999-999999999999"),
    );

    let res = create_as(&server, "SKR001", &body).await;
    assert_eq!(
        res.status_code(),
        201,
        "an old client's extra keys must not turn into a 4xx: {}",
        res.text()
    );
    let data = res.json::<Value>()["data"].clone();
    for (field, spoofed) in [
        ("pegawai_nama", "Tokoh Karangan"),
        ("pegawai_jabatan", "Jaksa Agung"),
        ("pegawai_pangkat", "Jenderal"),
        ("pegawai_satker_nama", "Satker Karangan"),
        ("pegawai_satker_id", "99999999-9999-4999-8999-999999999999"),
    ] {
        assert_ne!(
            data[field], spoofed,
            "{field} took the value from the request body"
        );
    }
    assert_eq!(data["pegawai_nama"], "Pegawai Uji A");
    assert_eq!(
        data["pegawai_satker_id"].as_str().unwrap(),
        satker_uuid_of(&db, "SKR001").await
    );

    teardown_test_db(&db_name).await;
}

/// The other direction. Without it the tests above pass with no scope at all:
/// an operator could file a permit in their own satker's name for anyone.
#[tokio::test]
async fn another_satkers_employee_cannot_be_named_and_nothing_is_written() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let res = create_as(&server, "SKR001", &body_for(NIP_SKR003, "703")).await;
    assert_eq!(
        res.status_code(),
        400,
        "a permit was filed for another satker's employee: {}",
        res.text()
    );
    assert!(
        !res.text().contains("Pegawai Uji C"),
        "the refusal leaked the other satker's employee: {}",
        res.text()
    );
    assert_eq!(permit_count(&db).await, 0, "a refused create left a row");

    teardown_test_db(&db_name).await;
}

/// "Not in MySIMKARI" and "in another satker" are ONE answer, so the refusal
/// cannot be used to find out whether a NIP exists elsewhere.
#[tokio::test]
async fn an_unknown_nip_gets_the_same_refusal_as_another_satkers_nip() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let ada_di_satker_lain = create_as(&server, "SKR001", &body_for(NIP_SKR003, "704")).await;
    let tidak_ada = create_as(&server, "SKR001", &body_for("19000101000000000", "704")).await;

    assert_eq!(tidak_ada.status_code(), 400, "{}", tidak_ada.text());
    assert_eq!(
        tidak_ada.text(),
        ada_di_satker_lain.text(),
        "the two refusals differ, so the response says which NIPs exist"
    );
    assert_eq!(permit_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

/// Each satker's operator files for their own staff, and the surrogate tells
/// satkers apart while keeping a satker's people together.
#[tokio::test]
async fn the_satker_id_is_one_per_satker_not_one_per_permit() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // A second SKR001 employee, so "same satker" is measured between people.
    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "INSERT INTO integrasi.mysimkari_pegawai (nip, nama, satker_id, nama_satker, jabatan, jk) \
             VALUES ('19800101000000011', 'Pegawai Uji A2', 'api-skr001', \
                     'KEJAKSAAN NEGERI UJI A', 'Staf', 'P')",
            &[],
        )
        .await
        .unwrap();

    let satker_id = |res: axum_test::TestResponse| {
        assert_eq!(res.status_code(), 201, "{}", res.text());
        res.json::<Value>()["data"]["pegawai_satker_id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let a1 = satker_id(create_as(&server, "SKR001", &body_for(NIP_SKR001, "711")).await);
    let a2 = satker_id(create_as(&server, "SKR001", &body_for("19800101000000011", "712")).await);
    let b = satker_id(create_as(&server, "SKR002", &body_for(NIP_SKR002, "713")).await);

    assert_eq!(
        a1, a2,
        "two employees of one satker got different satker ids"
    );
    assert_ne!(a1, b, "two satkers got the same satker id");

    teardown_test_db(&db_name).await;
}

/// An employee whose satker does not resolve cannot be given a permit: it would
/// have no satker identity to be seen under.
#[tokio::test]
async fn an_employee_with_an_unresolvable_satker_cannot_be_given_a_permit() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "UPDATE integrasi.mysimkari_pegawai SET satker_id = 'api-tidak-ada' WHERE nip = $1",
            &[&NIP_SKR001],
        )
        .await
        .unwrap();

    let res = create_as(&server, "SKR001", &body_for(NIP_SKR001, "721")).await;
    assert_eq!(res.status_code(), 400, "{}", res.text());
    assert_eq!(permit_count(&db).await, 0);

    teardown_test_db(&db_name).await;
}

/// A renewal is the same person under the first letter's terms. It must not
/// ask MySIMKARI again: an employee who has since left the satker would
/// otherwise make a legitimate renewal unwritable.
#[tokio::test]
async fn a_renewal_keeps_the_first_permits_holder_even_after_the_employee_is_gone() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    let lama = post_as(
        &server,
        "/pemakaian-bmn",
        "operator_satker",
        OPERATOR,
        "SKR001",
        body_for(NIP_SKR001, "731"),
    )
    .await["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    post_as(
        &server,
        &format!("/pemakaian-bmn/{lama}/transition"),
        "operator_satker",
        OPERATOR,
        "SKR001",
        json!({"target_status": "SUBMITTED", "catatan": "Mohon validasi"}),
    )
    .await;
    post_as(
        &server,
        &format!("/pemakaian-bmn/{lama}/validator-satker-action"),
        "validator_satker",
        VALIDATOR_SATKER,
        "SKR001",
        json!({"action": "forward", "expected_version": 1, "catatan": "Diteruskan"}),
    )
    .await;
    post_as(
        &server,
        &format!("/pemakaian-bmn/{lama}/approver-satker-action"),
        "approver_satker",
        APPROVER_SATKER,
        "SKR001",
        json!({"action": "approve", "expected_version": 2, "catatan": "Disetujui"}),
    )
    .await;

    // The employee leaves MySIMKARI.
    db.pool()
        .get()
        .await
        .unwrap()
        .execute(
            "DELETE FROM integrasi.mysimkari_pegawai WHERE nip = $1",
            &[&NIP_SKR001],
        )
        .await
        .unwrap();

    let baru = post_as(
        &server,
        &format!("/pemakaian-bmn/{lama}/renew"),
        "operator_satker",
        OPERATOR,
        "SKR001",
        json!({
            "tanggal_mulai": "2027-01-01",
            "tanggal_selesai": "2027-12-31",
            "keperluan": "Perpanjangan untuk tugas kedinasan tahun berikutnya"
        }),
    )
    .await;
    let data = &baru["data"];
    assert_eq!(data["pegawai_nip"], NIP_SKR001);
    assert_eq!(data["pegawai_nama"], "Pegawai Uji A");
    assert_eq!(data["pegawai_jabatan"], "Staf");
    assert_eq!(
        data["pegawai_satker_id"].as_str().unwrap(),
        satker_uuid_of(&db, "SKR001").await
    );

    teardown_test_db(&db_name).await;
}
