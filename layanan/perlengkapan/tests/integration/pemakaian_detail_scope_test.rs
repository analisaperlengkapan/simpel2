//! Object-level scoping for the Pemakaian BMN surfaces reached BY PERMIT ID.
//!
//! The LIST has been scoped since #66, and `pemakaian_monitoring_scope_test`
//! pins it. The detail behind that list was not, and neither were the eleven
//! other places that reach a permit by id. Measured against deployed staging
//! (rc29), operator_a (0200010) against operator_b's permit (0200020):
//!
//! ```text
//! GET /pemakaian-bmn/{foreign}             -> 200, the whole record
//! GET /pemakaian-bmn/{foreign}/sk-izin.pdf -> 200, application/pdf, 5 057 bytes
//! ```
//!
//! A permit names a person, an asset and a period — the exact fields the
//! stakeholder asked to be limited per role. `get_permit_by_id` did read the
//! caller's claims, but only to trim `allowed_transitions`: the record itself
//! came back in full. A handler that uses `claims` is not thereby a handler
//! that authorizes, which is why the unused-`_claims` census could not see it.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use uuid::Uuid;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, satker_code: &str) -> Headers {
    let user_id = "00000000-0000-0000-0000-000000000001";
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
            reqwest::header::HeaderValue::from_str(satker_code).unwrap(),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user_id}::{satker_code}"
            ))
            .unwrap(),
        ),
    ]
}

/// SKR001 and SKR002 both sit under KJT01; SKR003 sits under KJT02.
const SATKER_A: &str = "SKR001";
const SATKER_B: &str = "SKR002";
const SATKER_C: &str = "SKR003";

async fn get(server: &TestServer, path: &str, role: &str, satker: &str) -> axum_test::TestResponse {
    let mut req = server.get(path);
    for (k, v) in auth_headers(role, satker) {
        req = req.add_header(k, v);
    }
    req.await
}

/// One ACTIVE permit per satker, each carrying a name only that satker should
/// ever see. Asserting on the name rather than a row count is deliberate: a
/// count can match by coincidence, a person's name cannot.
async fn seed_permits(db: &layanan_perlengkapan::shared::db::Database) -> (Uuid, Uuid, Uuid) {
    let client = db.pool().get().await.unwrap();
    let ids = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    for (id, satker, pegawai, nip, nup) in [
        (
            ids.0,
            SATKER_A,
            "Andi Pemakai A",
            "19800101000000101",
            "PD-A-1",
        ),
        (
            ids.1,
            SATKER_B,
            "Budi Pemakai B",
            "19800101000000102",
            "PD-B-1",
        ),
        (
            ids.2,
            SATKER_C,
            "Cici Pemakai C",
            "19800101000000103",
            "PD-C-1",
        ),
    ] {
        client
            .execute(
                "INSERT INTO perlengkapan.izin_pemakaian_bmn
                    (id, nomor_izin, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang,
                     bmn_merk, bmn_tipe, pegawai_nip, pegawai_nama, pegawai_jabatan,
                     pegawai_satker_id, pegawai_satker_nama, satker_code,
                     tanggal_mulai, tanggal_selesai, status, approved_at)
                 VALUES ($1, $2, 'LAINNYA', $3, '3060201003', 'Meja Kerja',
                         'Merk Uji', 'Tipe Uji', $4, $5, 'Staf',
                         gen_random_uuid(), $6, $7,
                         CURRENT_DATE - 10, CURRENT_DATE + 20, 'ACTIVE', NOW())",
                &[
                    &id,
                    &format!("IZIN/{satker}/{nup}"),
                    &nup,
                    &nip,
                    &pegawai,
                    &format!("KEJAKSAAN NEGERI UJI {}", &satker[5..]),
                    &satker,
                ],
            )
            .await
            .expect("seed permit");
    }
    ids
}

#[tokio::test]
async fn an_operator_reads_its_own_permit_detail() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (own, _, _) = seed_permits(&db).await;

    let res = get(
        &server,
        &format!("/pemakaian-bmn/{own}"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(res.status_code(), 200, "own permit: {:?}", res.text());
    let body = res.json::<serde_json::Value>();
    // The detail response flattens the permit onto `data` rather than nesting
    // it under `izin`.
    assert_eq!(body["data"]["pegawai_nama"], "Andi Pemakai A", "{body:?}");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_operator_cannot_read_another_satkers_permit_detail() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (_, foreign, _) = seed_permits(&db).await;

    let res = get(
        &server,
        &format!("/pemakaian-bmn/{foreign}"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    // 404, not 403: 403 would confirm the permit exists under another satker.
    assert_eq!(
        res.status_code(),
        404,
        "cross-satker permit detail: {:?}",
        res.text()
    );
    assert!(
        !res.text().contains("Budi Pemakai B"),
        "the holder's name must not leak in the refusal: {}",
        res.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_decision_letter_is_closed_across_satkers() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (own, foreign, _) = seed_permits(&db).await;

    let mine = get(
        &server,
        &format!("/pemakaian-bmn/{own}/sk-izin.pdf"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(mine.status_code(), 200, "own SK: {:?}", mine.text());

    let theirs = get(
        &server,
        &format!("/pemakaian-bmn/{foreign}/sk-izin.pdf"),
        "operator_satker",
        SATKER_A,
    )
    .await;
    assert_eq!(
        theirs.status_code(),
        404,
        "another satker's decision letter was served"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_wilayah_validator_reads_its_region_but_not_the_next_one() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (a, b, c) = seed_permits(&db).await;

    for (id, label) in [(a, "SKR001"), (b, "SKR002")] {
        let res = get(
            &server,
            &format!("/pemakaian-bmn/{id}"),
            "validator_wilayah",
            "KJT01",
        )
        .await;
        assert_eq!(res.status_code(), 200, "{label} is inside KJT01");
    }

    let res = get(
        &server,
        &format!("/pemakaian-bmn/{c}"),
        "validator_wilayah",
        "KJT01",
    )
    .await;
    assert_eq!(res.status_code(), 404, "SKR003 belongs to KJT02");

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_cross_satker_role_reads_every_permit() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (a, b, c) = seed_permits(&db).await;

    for id in [a, b, c] {
        let res = get(
            &server,
            &format!("/pemakaian-bmn/{id}"),
            "validator_pusat",
            "PUSAT001",
        )
        .await;
        assert_eq!(res.status_code(), 200, "pusat sees every permit");
    }

    teardown_test_db(&db_name).await;
}

/// The revoke endpoint is role-gated by policy. That answers "may a role ever
/// do this"; this asserts the other half — may THIS caller do it to THIS
/// permit.
///
/// The role must be `approver_satker`: it is the only one the policy admits
/// for Revoke on an ACTIVE permit. A first version of this test used
/// `validator_satker` and passed — but for the wrong reason, the role gate,
/// and it stayed green when the scope was disabled. The canary caught it. A
/// negative test that a role gate can satisfy is not testing the object gate.
#[tokio::test]
async fn a_workflow_action_cannot_reach_another_satkers_permit() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let (_, foreign, _) = seed_permits(&db).await;

    let mut req = server
        .post(&format!("/pemakaian-bmn/{foreign}/revoke"))
        .json(&serde_json::json!({"alasan": "uji lintas satker"}));
    for (k, v) in auth_headers("approver_satker", SATKER_A) {
        req = req.add_header(k, v);
    }
    let res = req.await;
    // 404, and specifically not 403: the scope decides before the policy does,
    // so the answer must not confirm the permit exists elsewhere.
    assert_eq!(
        res.status_code(),
        404,
        "a revoke must not reach another satker's permit: {:?}",
        res.text()
    );

    // And the permit is untouched: still ACTIVE when its owner looks.
    let after = get(
        &server,
        &format!("/pemakaian-bmn/{foreign}"),
        "operator_satker",
        SATKER_B,
    )
    .await;
    assert_eq!(after.status_code(), 200);
    assert_eq!(
        after.json::<serde_json::Value>()["data"]["status"],
        "ACTIVE"
    );

    teardown_test_db(&db_name).await;
}
