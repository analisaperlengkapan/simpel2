//! `GET /bank-aset/kodefikasi` — the codification a form offers instead of
//! asking an operator to type it.
//!
//! The kebutuhan-BMN form asked for a nama barang (free text, required) and a
//! kode barang (free text, optional). Both already exist in `integrasi`, so
//! typing them invited the two failures the reference rule exists to prevent:
//! a wrong code, and two spellings of one item that stop aggregating together.
//!
//! What this file pins, measured on the staging snapshot 2026-08-30 over
//! 624 533 assets:
//!
//! * the codification is a FUNCTION — 2 033 codes, 2 033 names, no code with
//!   two names;
//! * the reverse is NOT — seven names are shared by two codes each, so a name
//!   does not determine a code and the endpoint must return the code;
//! * the search reads the snapshot, not `siman_aset`: the same query against
//!   the table is a 1 285 ms parallel seq scan.
//!
//! The rows below use REAL (code, name) pairs. A fixture that invents them is
//! not a smaller version of the truth: it makes the codification look
//! many-to-many, which is precisely the property this endpoint relies on.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const SATKER: &str = "SKR001";

fn auth_headers() -> Headers {
    vec![
        (
            reqwest::header::HeaderName::from_static("x-user-id"),
            reqwest::header::HeaderValue::from_static(OPERATOR),
        ),
        (
            reqwest::header::HeaderName::from_static("x-user-role"),
            reqwest::header::HeaderValue::from_static("operator_satker"),
        ),
        (
            reqwest::header::HeaderName::from_static("x-satker-id"),
            reqwest::header::HeaderValue::from_static(SATKER),
        ),
        (
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_static(
                "Bearer mock::operator_satker::00000000-0000-0000-0000-000000000001::SKR001",
            ),
        ),
    ]
}

/// Seed assets, then refresh the snapshot the way the CronJob does.
///
/// The refresh is part of the test rather than hidden in the harness: the
/// snapshot is only as fresh as its last refresh, and a test that never
/// refreshes would pass against a definition that reads nothing.
async fn seed(db: &layanan_perlengkapan::shared::db::Database) {
    let client = db.pool().get().await.unwrap();
    client
        .batch_execute(
            "INSERT INTO integrasi.siman_aset
               (jenis_aset, no_aset, ur_sskel, nama, kd_brg, kdsatker_keu)
             VALUES
               -- Station Wagon at two satkers: the counts must aggregate, and
               -- jumlah_satker must count satkers rather than rows.
               ('Alat Angkutan Bermotor', 'K-1', 'Station Wagon', 'Avanza', '3020101003', 'KEU-A'),
               ('Alat Angkutan Bermotor', 'K-2', 'Station Wagon', 'Innova', '3020101003', 'KEU-A'),
               ('Alat Angkutan Bermotor', 'K-3', 'Station Wagon', 'Xenia',  '3020101003', 'KEU-B'),
               -- Two motorbikes, so ordering by prevalence has something to say.
               ('Alat Angkutan Bermotor', 'K-4', 'Sepeda Motor',  'Vario',  '3020104001', 'KEU-A'),
               ('Alat Angkutan Bermotor', 'K-5', 'Sepeda Motor',  'Beat',   '3020104001', 'KEU-B'),
               -- One notebook.
               ('Peralatan Mesin Khusus TIK', 'K-6', 'Note Book', 'Latitude', '3100102003', 'KEU-A'),
               -- A row with no barang code at all. 22% of the register has a
               -- blank `nama`, and blanks reach this column too; a codification
               -- entry with an empty code is not an entry.
               ('Peralatan Mesin Khusus TIK', 'K-7', 'Tanpa Kode', 'Anonim', '', 'KEU-A')",
        )
        .await
        .unwrap();
    client
        .batch_execute("REFRESH MATERIALIZED VIEW integrasi.mv_kodefikasi_barang")
        .await
        .unwrap();
}

async fn kodefikasi(server: &TestServer, query: &str) -> serde_json::Value {
    let mut req = server.get(&format!("/bank-aset/kodefikasi{query}"));
    for (k, v) in auth_headers() {
        req = req.add_header(k, v);
    }
    let resp = req.await;
    assert_eq!(resp.status_code(), 200, "body: {}", resp.text());
    resp.json::<serde_json::Value>()
}

fn names(body: &serde_json::Value) -> Vec<String> {
    body["data"]
        .as_array()
        .expect("data must be an array")
        .iter()
        .map(|e| e["nama_barang"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn search_returns_identified_codes_ordered_by_prevalence() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    let body = kodefikasi(&server, "?q=motor").await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "only Sepeda Motor matches: {body}");
    assert_eq!(data[0]["kode_barang"], "3020104001");
    assert_eq!(data[0]["nama_barang"], "Sepeda Motor");
    assert_eq!(data[0]["jumlah_aset"], 2);
    assert_eq!(
        data[0]["jumlah_satker"], 2,
        "jumlah_satker counts satkers, not rows"
    );

    // Prevalence first, across the whole codification. Asserted as RELATIVE
    // order rather than as the whole list: the shared harness seeds assets of
    // its own, so a test that pinned the exact list would be claiming to own a
    // register it shares — and would break the next time the harness gains a
    // row, for a reason having nothing to do with this endpoint.
    //
    // The three-character minimum lives in the picker, not here: over 2 038
    // rows a short query costs nothing, and a backend that answered "no match"
    // to a real query would be lying about the codification.
    let body = kodefikasi(&server, "").await;
    let all = names(&body);
    let at = |n: &str| {
        all.iter()
            .position(|x| x == n)
            .unwrap_or_else(|| panic!("{n} missing from the codification: {body}"))
    };
    assert!(
        at("Station Wagon") < at("Sepeda Motor"),
        "3 assets must outrank 2: {all:?}"
    );
    assert!(
        at("Sepeda Motor") < at("Note Book"),
        "2 assets must outrank 1: {all:?}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn one_asset_per_row_is_not_one_entry_per_asset() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    // Three Station Wagon assets, ONE codification entry. The whole point of
    // the snapshot is that a picker offers 2 038 rows and not 624 533.
    let body = kodefikasi(&server, "?q=station").await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1, "three assets, one entry: {body}");
    assert_eq!(data[0]["jumlah_aset"], 3);
    assert_eq!(data[0]["jumlah_satker"], 2);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_code_is_matched_by_prefix_and_survives_the_dotted_spelling() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    // SIMAN stores ten undotted digits; the rest of the system writes the
    // dotted presentation form, and five live rows of
    // pengajuan_kebutuhan_bmn_satker_barang carry it. An operator pasting one
    // must find the code, or the comparison that "can never hold" is back.
    // The full code, and the dotted presentation form of it, name one entry.
    for q in ["3020101003", "3.02.01.01.003"] {
        let body = kodefikasi(&server, &format!("?q={q}")).await;
        assert_eq!(
            names(&body),
            vec!["Station Wagon".to_string()],
            "query {q} must resolve the code: {body}"
        );
    }

    // A partial code is a BRANCH of the taxonomy, not a worse spelling of one
    // code: `30201` is the vehicle sub-group, so it must return every code in
    // it — still in prevalence order. Asserting one result here would have
    // pinned the prefix search to behaviour it does not have, and should not.
    let body = kodefikasi(&server, "?q=30201").await;
    assert_eq!(
        names(&body),
        vec!["Station Wagon".to_string(), "Sepeda Motor".to_string()],
        "a code prefix returns the whole sub-group: {body}"
    );

    // Infix on the code is deliberately NOT supported: "0101" sits in the
    // middle of unrelated codes, and an operator typing digits is typing the
    // start of one.
    let body = kodefikasi(&server, "?q=0101").await;
    assert!(
        names(&body).is_empty(),
        "codes match by prefix, not infix: {body}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_entry_without_a_code_is_not_an_entry() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    // The asset exists, but it cannot be OFFERED: picking it would hand the
    // form an empty kode_barang, which is exactly the free-text state this
    // endpoint replaces.
    let body = kodefikasi(&server, "?q=Tanpa%20Kode").await;
    assert!(
        names(&body).is_empty(),
        "a blank barang code must not reach the picker: {body}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_codification_is_national_not_satker_scoped() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    // The caller sits in SKR001, which maps to no kdsatker_keu in this fixture
    // at all — and must still see the whole codification. A satker states what
    // it NEEDS, which is most often a barang it does not own; scoping this
    // list would make the form unable to express its commonest request, and it
    // would do so silently, as an empty result.
    let body = kodefikasi(&server, "?q=note").await;
    assert_eq!(
        names(&body),
        vec!["Note Book".to_string()],
        "the codification is a national taxonomy: {body}"
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn limit_is_validated_rather_than_silently_clamped() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed(&db).await;

    let mut req = server.get("/bank-aset/kodefikasi?q=station&limit=5000");
    for (k, v) in auth_headers() {
        req = req.add_header(k, v);
    }
    let resp = req.await;
    assert_eq!(
        resp.status_code(),
        400,
        "an out-of-range limit must be refused, not quietly answered: {}",
        resp.text()
    );

    // A caller that omits `limit` gets the default rather than a 400 — the
    // parameter is optional, and #817 was exactly a query field that turned a
    // working request into a 400 before the handler ran.
    let body = kodefikasi(&server, "").await;
    assert!(
        body["data"].as_array().unwrap().len() >= 3,
        "no query returns the codification: {body}"
    );

    teardown_test_db(&db_name).await;
}
