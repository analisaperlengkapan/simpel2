//! Kebutuhan BMN satker decisions must reach the cross-module audit trail.
//!
//! `transition_satker_status` deliberately bypasses the generic workflow engine
//! (the engine's status table is the PARENT `pengajuan_kebutuhan_bmn`, the
//! satker row is not there), and the engine is what writes `audit_log`. So the
//! busiest workflow in the system wrote its own per-satker activity rows
//! (267 on staging) and nothing at all where auditors look: the admin Audit Log
//! page, even once it read the right table, could not show a single Kebutuhan
//! BMN decision.
//!
//! Driven through HTTP exactly as the app does it: operator submits, wilayah
//! forwards, pusat decides. Then the trail is read back through `GET /audit`.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

const PUSAT: &str = "00000000-0000-0000-0000-000000000003";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const WILAYAH: &str = "00000000-0000-0000-0000-000000000002";
const ADMIN: &str = "00000000-0000-0000-0000-000000000009";
const SATKER: &str = "SKR001";

type Req = axum_test::TestRequest;

fn as_role(req: Req, role: &str, user: &str, satker: &str) -> Req {
    req.add_header(
        reqwest::header::HeaderName::from_static("authorization"),
        reqwest::header::HeaderValue::from_str(&format!("Bearer mock::{role}::{user}::{satker}"))
            .unwrap(),
    )
}

/// The satker's audit rows, oldest first.
async fn trail(server: &TestServer, satker_row: &str) -> Vec<Value> {
    let res = as_role(
        server.get(&format!(
            "/audit?entity=kebutuhan_bmn_satker&resource_id={satker_row}&per_page=200"
        )),
        "admin",
        ADMIN,
        "PUSAT001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let mut rows = res.json::<Value>()["data"].as_array().unwrap().clone();
    rows.reverse(); // the endpoint answers newest first
    rows
}

#[tokio::test]
async fn every_satker_decision_leaves_a_row_in_the_audit_trail() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // Campaign + satker + barang (setup, not the thing under test).
    let res = as_role(
        server.post("/kebutuhan-bmn/pengajuan").json(&json!({
            "nama": "Kebutuhan BMN 2026", "tahun": 2026,
            "tgl_mulai": "2026-01-01", "tgl_selesai": "2026-12-31"
        })),
        "validator_pusat",
        PUSAT,
        "PUSAT001",
    )
    .await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let pengajuan_id = res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/pengajuan/{pengajuan_id}/satker"))
            .json(&json!({"satker_id": SATKER, "satker_name": "Kejaksaan Negeri Jakarta Selatan"})),
        "validator_pusat",
        PUSAT,
        "PUSAT001",
    )
    .await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());
    let created = res.json::<Value>();
    let row = created["data"]["id"].as_str().unwrap().to_string();
    let start_status = created["data"]["status_kode"].as_i64().unwrap();

    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/satker/{row}/barang"))
            .json(&json!({"nama": "Laptop", "jumlah": 5, "satuan": "Unit"})),
        "operator_satker",
        OPERATOR,
        SATKER,
    )
    .await;
    assert_eq!(res.status_code(), 201, "{:?}", res.text());

    // Nothing has been decided yet: the trail for this row is empty.
    assert!(trail(&server, &row).await.is_empty());

    // operator → wilayah → pusat approves.
    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/satker/{row}/submit-wilayah"))
            .json(&json!({
                "catatan_satker": "Mohon persetujuan",
                "lampiran_surat_permohonan": "https://storage.example.com/surat.pdf"
            })),
        "operator_satker",
        OPERATOR,
        SATKER,
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let after_submit = trail(&server, &row).await;
    assert_eq!(
        after_submit.len(),
        1,
        "one decision, one row: {after_submit:?}"
    );

    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/satker/{row}/validator-wilayah"))
            .json(&json!({"aksi": "forward", "catatan": "Diteruskan ke Pusat"})),
        "validator_wilayah",
        WILAYAH,
        "KJT01",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());

    // A refused request changes nothing and therefore records nothing: an
    // operator may not take the pusat decision.
    let before_refused = trail(&server, &row).await.len();
    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/satker/{row}/keputusan-pusat"))
            .json(&json!({"is_approved": true, "alasan": "tidak berwenang"})),
        "operator_satker",
        OPERATOR,
        SATKER,
    )
    .await;
    assert_eq!(res.status_code(), 403, "{:?}", res.text());
    assert_eq!(trail(&server, &row).await.len(), before_refused);

    let res = as_role(
        server
            .post(&format!("/kebutuhan-bmn/satker/{row}/keputusan-pusat"))
            .json(&json!({"is_approved": true, "alasan": "Sesuai standar"})),
        "validator_pusat",
        PUSAT,
        "PUSAT001",
    )
    .await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    assert_eq!(res.json::<Value>()["data"]["status_kode"], 2006);

    // --- the trail itself -------------------------------------------------
    let rows = trail(&server, &row).await;
    // A pusat approval from SubmitPusat passes through AnalisisKelayakan, so the
    // count is not "one per HTTP call"; what must hold is that the rows form an
    // unbroken chain from where the satker started to where it ended.
    assert!(rows.len() >= 3, "submit, forward, decide: {rows:?}");
    let meta = |r: &Value, k: &str| r["metadata"][k].as_i64().unwrap_or(-1);
    assert_eq!(meta(&rows[0], "from_status_kode"), start_status);
    for pair in rows.windows(2) {
        assert_eq!(
            meta(&pair[0], "to_status_kode"),
            meta(&pair[1], "from_status_kode"),
            "a decision is missing between {pair:?}"
        );
    }
    assert_eq!(meta(rows.last().unwrap(), "to_status_kode"), 2006);

    for r in &rows {
        assert_eq!(r["module"], "kebutuhan_bmn");
        assert_eq!(r["resource_type"], "kebutuhan_bmn_satker");
        assert_eq!(r["action"], "custom");
        assert_eq!(r["action_name"], "workflow.transition");
        assert_eq!(r["success"], true);
        assert_eq!(r["resource_id"], row.as_str());
        assert_eq!(r["metadata"]["satker_code"], SATKER);
        // Never a blank name: absent, or a real one.
        assert_ne!(r["actor_name"], "", "blank actor name: {r}");
    }
    // Who did what: the operator submitted, the pusat validator decided.
    assert_eq!(rows[0]["actor_id"], OPERATOR);
    assert_eq!(rows.last().unwrap()["actor_id"], PUSAT);
    assert_eq!(rows[0]["metadata"]["actor_role"], "operator_satker");
    assert_eq!(rows.last().unwrap()["message"], "Sesuai standar");

    // And the module the admin page could never show is now selectable.
    let by_module = as_role(
        server.get("/audit?module=kebutuhan_bmn&per_page=200"),
        "admin",
        ADMIN,
        "PUSAT001",
    )
    .await;
    assert_eq!(by_module.status_code(), 200, "{:?}", by_module.text());
    assert_eq!(
        by_module.json::<Value>()["total"].as_i64().unwrap(),
        rows.len() as i64
    );

    teardown_test_db(&db_name).await;
}
