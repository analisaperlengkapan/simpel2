//! The audit trail the admin page shows must be the one the system writes.
//!
//! The admin "Audit Log" page used to read `workflow_transitions` JOIN
//! `workflow_instances` — two tables no code path ever INSERTs into — so it
//! answered "0 entri" in every environment while `perlengkapan.audit_log` held
//! every real event. Nothing failed: the endpoint returned 200 and the access
//! tests only asked who may OPEN the page, never what it contained.
//!
//! These tests write through the real [`PgAuditSink`] (the writer production
//! uses, not a hand-written INSERT that could drift from it) and read back
//! through the HTTP endpoints the page calls.
//!
//! Roles ride in the mock bearer token `mock::<role>::<user_id>::<satker>`.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use layanan_perlengkapan::contracts::AuditSink;
use layanan_perlengkapan::shared::audit::PgAuditSink;
use lib_perlengkapan::audit::{AuditAction, AuditEvent};
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

const ADMIN: &str = "00000000-0000-0000-0000-000000000009";
const PUSAT: &str = "00000000-0000-0000-0000-000000000004";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const SATKER: &str = "SKR001";

async fn get(server: &TestServer, path: &str, role: &str, user: &str) -> axum_test::TestResponse {
    server
        .get(path)
        .add_header(
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_str(&format!(
                "Bearer mock::{role}::{user}::{SATKER}"
            ))
            .unwrap(),
        )
        .await
}

/// One event per writer the system really has, with the module / resource_type /
/// action_name strings those writers really use (`bantuan/ticket.rs`,
/// `pakaian_dinas/services.rs`, `workflow/engine/transition.rs`).
async fn seed_like_the_real_writers(sink: &PgAuditSink) {
    let who = Uuid::parse_str(OPERATOR).unwrap();
    let events = [
        AuditEvent::new("bantuan", AuditAction::Create, "tiket")
            .action_name("tiket.create")
            .actor(who, "operator.satu")
            .resource_id(Uuid::new_v4().to_string())
            .message("subject=Printer rusak"),
        AuditEvent::new("bantuan", AuditAction::Update, "tiket")
            .action_name("tiket.status_change")
            .actor(who, "operator.satu")
            .resource_id(Uuid::new_v4().to_string())
            .message("open -> in_progress"),
        AuditEvent::new("pakaian_dinas", AuditAction::Approve, "pengajuan_satker")
            .action_name("pakaian_dinas.validator_action")
            .actor(who, "operator.satu")
            .resource_id(Uuid::new_v4().to_string()),
        AuditEvent::new("penghapusan_bmn", AuditAction::Custom, "penghapusan_bmn")
            .action_name("workflow.transition")
            .actor(who, "")
            .resource_id(Uuid::new_v4().to_string())
            .message("4000 → 4001"),
        AuditEvent::new("pemakaian_bmn", AuditAction::Custom, "pemakaian_bmn")
            .action_name("workflow.transition")
            .actor(who, "")
            .resource_id(Uuid::new_v4().to_string()),
    ];
    for e in events {
        sink.log(e).await.expect("audit sink write");
    }
}

fn strings(v: &Value) -> BTreeSet<String> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_str().expect("string").to_string())
        .collect()
}

#[tokio::test]
async fn what_the_system_audits_is_what_the_page_can_list() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_like_the_real_writers(&PgAuditSink::new(db.pool().clone())).await;

    // The page's own call: no filter, default page. Before the fix the page read
    // a different table and this total was 0 regardless of what was audited.
    let all = get(&server, "/audit?page=1&per_page=25", "admin", ADMIN).await;
    assert_eq!(all.status_code(), 200, "{:?}", all.text());
    assert_eq!(
        all.json::<Value>()["total"],
        5,
        "every audited event listed"
    );
    let first = &all.json::<Value>()["data"][0];
    // The fields the page renders must be present on the wire.
    for key in [
        "id",
        "occurred_at",
        "action",
        "resource_type",
        "module",
        "success",
    ] {
        assert!(
            first.get(key).is_some(),
            "wire entry lacks `{key}`: {first}"
        );
    }

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn every_dropdown_option_matches_a_row_and_no_row_is_unreachable() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_like_the_real_writers(&PgAuditSink::new(db.pool().clone())).await;

    let res = get(&server, "/audit/facets", "admin", ADMIN).await;
    assert_eq!(res.status_code(), 200, "{:?}", res.text());
    let body = res.json::<Value>();
    let f = &body["data"];

    // Exactly the vocabulary that was written — nothing invented (`bank_aset`,
    // `login`, ...), nothing missing (`tiket`, `pengajuan_satker`).
    assert_eq!(
        strings(&f["modules"]),
        [
            "bantuan",
            "pakaian_dinas",
            "pemakaian_bmn",
            "penghapusan_bmn"
        ]
        .map(String::from)
        .into()
    );
    assert_eq!(
        strings(&f["entities"]),
        [
            "pemakaian_bmn",
            "penghapusan_bmn",
            "pengajuan_satker",
            "tiket"
        ]
        .map(String::from)
        .into()
    );
    assert_eq!(
        strings(&f["actions"]),
        ["approve", "create", "custom", "update"]
            .map(String::from)
            .into()
    );

    // The property that closes the class: choosing any offered option shows at
    // least one row. A dropdown that offers a value nothing was logged under is
    // exactly the dead filter this replaces.
    for (facet, query_key) in [
        ("modules", "module"),
        ("entities", "entity"),
        ("actions", "action"),
    ] {
        for value in strings(&f[facet]) {
            let r = get(
                &server,
                &format!("/audit?{query_key}={value}&per_page=200"),
                "admin",
                ADMIN,
            )
            .await;
            assert_eq!(r.status_code(), 200, "{:?}", r.text());
            let total = r.json::<Value>()["total"].as_i64().unwrap();
            assert!(
                total >= 1,
                "{query_key}={value} is offered but matches nothing"
            );
        }
    }

    // And the filter really filters: a value that does not occur narrows to zero
    // rather than being ignored (serde drops unknown keys silently; an ignored
    // filter would still say 200 with every row).
    let none = get(&server, "/audit?module=tidak_ada", "admin", ADMIN).await;
    assert_eq!(none.json::<Value>()["total"], 0);
    let one = get(&server, "/audit?module=pakaian_dinas", "admin", ADMIN).await;
    assert_eq!(one.json::<Value>()["total"], 1);

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_date_filter_needs_rfc3339_and_a_future_bound_hides_everything() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_like_the_real_writers(&PgAuditSink::new(db.pool().clone())).await;

    // `from` in the future: nothing happened yet.
    let r = get(
        &server,
        "/audit?from=2999-01-01T00:00:00%2B07:00",
        "admin",
        ADMIN,
    )
    .await;
    assert_eq!(r.json::<Value>()["total"], 0, "{:?}", r.text());
    // `to` in the past: nothing happened before.
    let r = get(
        &server,
        "/audit?to=2000-01-01T23:59:59%2B07:00",
        "admin",
        ADMIN,
    )
    .await;
    assert_eq!(r.json::<Value>()["total"], 0, "{:?}", r.text());
    // A window around now holds all of it.
    let r = get(
        &server,
        "/audit?from=2000-01-01T00:00:00%2B07:00&to=2999-01-01T23:59:59%2B07:00",
        "admin",
        ADMIN,
    )
    .await;
    assert_eq!(r.json::<Value>()["total"], 5, "{:?}", r.text());

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_trail_and_its_facets_are_for_admin_and_pusat_only() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    seed_like_the_real_writers(&PgAuditSink::new(db.pool().clone())).await;

    for path in ["/audit", "/audit/facets"] {
        for (role, user) in [
            ("admin", ADMIN),
            ("validator_pusat", PUSAT),
            ("pusat", PUSAT),
        ] {
            let r = get(&server, path, role, user).await;
            assert_eq!(r.status_code(), 200, "{role} {path}: {:?}", r.text());
        }
        // The trail names people and IPs across every satker: a satker operator
        // must not read it.
        let r = get(&server, path, "operator_satker", OPERATOR).await;
        assert_eq!(r.status_code(), 403, "operator {path}: {:?}", r.text());
    }

    teardown_test_db(&db_name).await;
}
