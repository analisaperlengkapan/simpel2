//! Function-level authorization (OWASP API5:2023 BFLA) — the tests that pin
//! what happened when the blanket "admin bypasses every check" was removed.
//!
//! Roles ride in the mock bearer token `mock::<role>::<user_id>::<satker>`
//! decoded by `AuthencClient::dummy()`.
//!
//! What is asserted here, and why each is a regression test rather than a
//! nicety:
//!
//! * **An administrator cannot make a business decision.** Every application
//!   admin role used to pass every policy in every state.
//! * **The dedicated endpoints cannot be side-stepped.** The generic permit
//!   transition used to accept any move the engine's role table allowed, skipping
//!   the version lock and the validator/approver stamps.
//! * **Maker-checker.** The proposer cannot validate, and the approver cannot be
//!   the proposer or the validator.
//! * **Break-glass is the only override**, needs a reason, and leaves a row.
//! * **Authentication is deny-by-default** at the router layer.

use crate::common::{setup_test_app, teardown_test_db};
use axum_test::TestServer;
use serde_json::{Value, json};

type Headers = Vec<(reqwest::header::HeaderName, reqwest::header::HeaderValue)>;

fn auth_headers(role: &str, user_id: &str, satker: &str) -> Headers {
    vec![(
        reqwest::header::HeaderName::from_static("authorization"),
        reqwest::header::HeaderValue::from_str(&format!(
            "Bearer mock::{role}::{user_id}::{satker}"
        ))
        .unwrap(),
    )]
}

const SATKER: &str = "SKR001";
const OPERATOR: &str = "00000000-0000-0000-0000-000000000001";
const VALIDATOR: &str = "00000000-0000-0000-0000-000000000002";
const APPROVER: &str = "00000000-0000-0000-0000-000000000003";
const PUSAT: &str = "00000000-0000-0000-0000-000000000004";
const ADMIN: &str = "00000000-0000-0000-0000-000000000009";

async fn call(
    server: &TestServer,
    method: &str,
    path: &str,
    role: &str,
    user: &str,
    body: Option<Value>,
) -> axum_test::TestResponse {
    let mut req = match method {
        "GET" => server.get(path),
        "POST" => server.post(path),
        "PUT" => server.put(path),
        other => panic!("unsupported method {other}"),
    };
    if let Some(b) = body {
        req = req.json(&b);
    }
    for (k, v) in auth_headers(role, user, SATKER) {
        req = req.add_header(k, v);
    }
    req.await
}

fn permit_body() -> Value {
    json!({
        "pegawai_nip": "198501012010011001",
        "pegawai_nama": "Budi",
        "pegawai_satker_id": "00000000-0000-0000-0000-000000000001",
        "pegawai_satker_nama": "Kejari Jaksel",
        "jenis_bmn": "LAPTOP",
        "bmn_nup": "77",
        "bmn_kode_barang": "3060201003",
        "bmn_nama_barang": "Laptop Dell",
        "bmn_merk": "Dell",
        "serial_number": "SN-1",
        "tanggal_mulai": "2026-01-01",
        "tanggal_selesai": "2026-12-31",
        "keperluan": "Tugas kedinasan sehari-hari operator"
    })
}

async fn create_permit(server: &TestServer) -> String {
    let res = call(
        server,
        "POST",
        "/pemakaian-bmn",
        "operator_satker",
        OPERATOR,
        Some(permit_body()),
    )
    .await;
    assert_eq!(res.status_code(), 201, "setup create: {:?}", res.text());
    res.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn permit_version(server: &TestServer, id: &str) -> i64 {
    let res = call(
        server,
        "GET",
        &format!("/pemakaian-bmn/{id}"),
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(res.status_code(), 200, "detail: {:?}", res.text());
    // `izin` is `#[serde(flatten)]`ed into the detail response.
    res.json::<Value>()["data"]["version"].as_i64().unwrap()
}

async fn submit(server: &TestServer, id: &str) {
    let res = call(
        server,
        "POST",
        &format!("/pemakaian-bmn/{id}/transition"),
        "operator_satker",
        OPERATOR,
        Some(json!({"target_status": "SUBMITTED"})),
    )
    .await;
    assert_eq!(res.status_code(), 200, "operator submit: {:?}", res.text());
}

// ---------------------------------------------------------------------------
// An administrator cannot make business decisions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_administrator_cannot_take_a_pemakaian_decision() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    submit(&server, &id).await;
    let version = permit_version(&server, &id).await;

    for role in ["admin", "superadmin", "admin_pusat"] {
        for (path, body) in [
            (
                "validator-satker-action",
                json!({"action": "forward", "expected_version": version}),
            ),
            (
                "approver-satker-action",
                json!({"action": "approve", "expected_version": version}),
            ),
            ("revoke", json!({"alasan": "tidak diperlukan lagi"})),
        ] {
            let r = call(
                &server,
                "POST",
                &format!("/pemakaian-bmn/{id}/{path}"),
                role,
                ADMIN,
                Some(body),
            )
            .await;
            assert_eq!(r.status_code(), 403, "{role} on {path}: {:?}", r.text());
        }
    }
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_administrator_cannot_decide_penghapusan_kebutuhan_or_pakaian() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let some_id = uuid::Uuid::new_v4();

    for (path, body) in [
        (
            format!("/penghapusan-bmn/{some_id}/verifikasi-pusat"),
            json!({}),
        ),
        (
            format!("/penghapusan-bmn/{some_id}/forward-pusat"),
            json!({"aksi": "forward"}),
        ),
        (
            format!("/penghapusan-bmn/{some_id}/generate-sk-wilayah"),
            json!({}),
        ),
        (
            format!("/kebutuhan-bmn/satker/{some_id}/keputusan-pusat"),
            json!({"is_approved": true}),
        ),
        (
            format!("/kebutuhan-bmn/satker/{some_id}/validator-wilayah"),
            json!({"aksi": "forward"}),
        ),
        (
            format!("/pakaian-dinas/pengajuan/{some_id}/approve"),
            json!({}),
        ),
        (
            format!("/pakaian-dinas/pengajuan/{some_id}/reject"),
            json!({}),
        ),
    ] {
        let r = call(&server, "POST", &path, "admin", ADMIN, Some(body)).await;
        assert_eq!(r.status_code(), 403, "admin on {path}: {:?}", r.text());
    }
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn opening_a_request_is_an_operators_act() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for role in [
        "validator_pusat",
        "validator_wilayah",
        "approver_satker",
        "admin",
    ] {
        let r = call(
            &server,
            "POST",
            "/pemakaian-bmn",
            role,
            PUSAT,
            Some(permit_body()),
        )
        .await;
        assert_eq!(
            r.status_code(),
            403,
            "{role} must not open a permit: {:?}",
            r.text()
        );
    }
    let r = call(
        &server,
        "POST",
        "/penghapusan-bmn",
        "validator_pusat",
        PUSAT,
        Some(json!({
            "kode_barang": "3060201003", "nama_barang": "Laptop", "nup": "77",
            "tanggal_penghapusan": "2026-01-01",
            "alasan": "Rusak berat dan sudah usang, tidak ekonomis diperbaiki",
            "metode_penghapusan": "Pemusnahan",
            "lampiran_persyaratan": "https://storage.example.com/l.pdf"
        })),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "penghapusan create as pusat: {:?}",
        r.text()
    );
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// The dedicated endpoints cannot be side-stepped
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_generic_transition_serves_only_submit_and_cancel() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    submit(&server, &id).await;

    // SUBMITTED -> SUBMITTED_APPROVER_SATKER is the validator's forward: it
    // belongs to `validator-satker-action` (version lock + stamps), even for the
    // role that is allowed to make the move.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/transition"),
        "validator_satker",
        VALIDATOR,
        Some(json!({"target_status": "SUBMITTED_APPROVER_SATKER"})),
    )
    .await;
    assert_eq!(r.status_code(), 400, "generic forward: {:?}", r.text());

    // A move no action performs is refused outright, for anyone.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/transition"),
        "operator_satker",
        OPERATOR,
        Some(json!({"target_status": "ACTIVE"})),
    )
    .await;
    assert!(
        r.status_code() == 400 || r.status_code() == 403,
        "generic SUBMITTED->ACTIVE: {} {:?}",
        r.status_code(),
        r.text()
    );
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_operator_may_cancel_their_draft_but_a_validator_may_not() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;

    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/transition"),
        "validator_satker",
        VALIDATOR,
        Some(json!({"target_status": "CANCELLED"})),
    )
    .await;
    assert_eq!(r.status_code(), 403, "validator cancel: {:?}", r.text());

    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/transition"),
        "operator_satker",
        OPERATOR,
        Some(json!({"target_status": "CANCELLED", "catatan": "salah input"})),
    )
    .await;
    assert_eq!(r.status_code(), 200, "operator cancel: {:?}", r.text());
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn activation_document_and_maintenance_endpoints_have_owners() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;

    // Manual activation: approver only, and only when APPROVED.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/activate"),
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "operator activate: {:?}", r.text());
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/activate"),
        "admin",
        ADMIN,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "admin activate: {:?}", r.text());

    // Documents: not for the read-only validators.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/generate-konsep-surat"),
        "validator_pusat",
        PUSAT,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "pusat generate: {:?}", r.text());
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/upload-signed-pdf"),
        "validator_wilayah",
        PUSAT,
        Some(json!({"signed_pdf_url": "https://storage.example.com/s.pdf"})),
    )
    .await;
    assert_eq!(r.status_code(), 403, "wilayah upload: {:?}", r.text());

    // Auto-expire sweeps every satker: administrators only.
    let r = call(
        &server,
        "POST",
        "/pemakaian-bmn/auto-expire",
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "operator auto-expire: {:?}", r.text());
    let r = call(
        &server,
        "POST",
        "/pemakaian-bmn/auto-expire",
        "validator_pusat",
        PUSAT,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "pusat auto-expire: {:?}", r.text());
    let r = call(
        &server,
        "POST",
        "/pemakaian-bmn/auto-expire",
        "admin",
        ADMIN,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 200, "admin auto-expire: {:?}", r.text());
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn admin_surfaces_are_not_open_to_validator_pusat() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // `require_admin` here used to be "may READ across satkers", which admitted
    // validator_pusat / pusat / analis_pusat to master-data writes.
    for role in ["validator_pusat", "pusat", "analis_pusat"] {
        let r = call(&server, "GET", "/admin/master", role, PUSAT, None).await;
        assert_eq!(r.status_code(), 403, "{role} master list: {:?}", r.text());
        let r = call(&server, "GET", "/admin/templates", role, PUSAT, None).await;
        assert_eq!(r.status_code(), 403, "{role} templates: {:?}", r.text());
        let r = call(&server, "GET", "/admin/audit", role, PUSAT, None).await;
        assert_eq!(r.status_code(), 403, "{role} admin audit: {:?}", r.text());
    }
    let r = call(&server, "GET", "/admin/master", "admin", ADMIN, None).await;
    assert_eq!(r.status_code(), 200, "admin master list: {:?}", r.text());
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Maker-checker
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_proposer_cannot_validate_their_own_permit() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    submit(&server, &id).await;
    let version = permit_version(&server, &id).await;

    // Same account, holding the validator role too: allowed by role, refused by
    // segregation of duties.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/validator-satker-action"),
        "validator_satker",
        OPERATOR,
        Some(json!({"action": "forward", "expected_version": version})),
    )
    .await;
    assert_eq!(r.status_code(), 403, "self-validation: {:?}", r.text());
    assert!(r.text().contains("Pemisahan tugas"), "{:?}", r.text());

    // A different validator may.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/validator-satker-action"),
        "validator_satker",
        VALIDATOR,
        Some(json!({"action": "forward", "expected_version": version})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        200,
        "independent validator: {:?}",
        r.text()
    );
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn the_approver_cannot_be_the_proposer_or_the_validator() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    submit(&server, &id).await;
    let version = permit_version(&server, &id).await;
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/validator-satker-action"),
        "validator_satker",
        VALIDATOR,
        Some(json!({"action": "forward", "expected_version": version})),
    )
    .await;
    assert_eq!(r.status_code(), 200, "validator forward: {:?}", r.text());
    let version = permit_version(&server, &id).await;

    for (who, why) in [(OPERATOR, "the proposer"), (VALIDATOR, "the validator")] {
        let r = call(
            &server,
            "POST",
            &format!("/pemakaian-bmn/{id}/approver-satker-action"),
            "approver_satker",
            who,
            Some(json!({"action": "approve", "expected_version": version})),
        )
        .await;
        assert_eq!(r.status_code(), 403, "approval by {why}: {:?}", r.text());
    }
    // The permit is untouched by the refusals.
    assert_eq!(permit_version(&server, &id).await, version);
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Break-glass
// ---------------------------------------------------------------------------

#[tokio::test]
async fn break_glass_needs_an_admin_a_reason_and_leaves_a_row() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    let path = format!("/admin/break-glass/pemakaian_bmn/{id}/transition");
    let reason = "Pengusul mutasi dan usulan tidak dapat lagi diproses";

    // Not for the business roles — including the one whose decisions it replaces.
    for role in ["operator_satker", "validator_pusat", "approver_satker"] {
        let r = call(
            &server,
            "POST",
            &path,
            role,
            PUSAT,
            Some(json!({"target_status": "CANCELLED", "alasan": reason})),
        )
        .await;
        assert_eq!(r.status_code(), 403, "{role} break-glass: {:?}", r.text());
    }

    // A one-word justification is not a justification.
    let r = call(
        &server,
        "POST",
        &path,
        "admin",
        ADMIN,
        Some(json!({"target_status": "CANCELLED", "alasan": "macet"})),
    )
    .await;
    assert_eq!(r.status_code(), 400, "short reason: {:?}", r.text());

    // Nothing has been logged for refusals that never got as far as acting.
    let client = db.pool().get().await.unwrap();
    let entity: uuid::Uuid = id.parse().unwrap();
    let n: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM perlengkapan.break_glass_log WHERE entity_id = $1",
            &[&entity],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(n, 0, "validation failures must not create log rows");

    // An unreachable state is refused by the workflow itself (break-glass moves
    // along the process's edges; it does not invent new ones) — and the attempt
    // is on record as a failure.
    let r = call(
        &server,
        "POST",
        &path,
        "admin",
        ADMIN,
        Some(json!({"target_status": "ACTIVE", "alasan": reason})),
    )
    .await;
    assert!(
        r.status_code().is_client_error() || r.status_code().is_server_error(),
        "{:?}",
        r.text()
    );
    let row = client
        .query_one(
            "SELECT outcome FROM perlengkapan.break_glass_log WHERE entity_id = $1 ORDER BY created_at DESC LIMIT 1",
            &[&entity],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>(0), "failure");

    // A valid override succeeds, is attributed to the admin, and is settled.
    let r = call(
        &server,
        "POST",
        &path,
        "admin",
        ADMIN,
        Some(json!({"target_status": "CANCELLED", "alasan": reason, "referensi": "ND-12/2026"})),
    )
    .await;
    assert_eq!(r.status_code(), 200, "admin break-glass: {:?}", r.text());
    let row = client
        .query_one(
            "SELECT outcome, actor_user_id::text, reason, reference, from_state, to_state
               FROM perlengkapan.break_glass_log
              WHERE entity_id = $1 AND outcome = 'success'",
            &[&entity],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, String>(1), ADMIN);
    assert_eq!(row.get::<_, String>(2), reason);
    assert_eq!(row.get::<_, String>(3), "ND-12/2026");
    assert_eq!(row.get::<_, String>(4), "DRAFT");
    assert_eq!(row.get::<_, String>(5), "CANCELLED");

    // The overridden side can read the log; an operator cannot.
    let r = call(
        &server,
        "GET",
        "/admin/break-glass",
        "validator_pusat",
        PUSAT,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 200, "pusat reads log: {:?}", r.text());
    assert!(r.json::<Value>()["data"].as_array().unwrap().len() >= 2);
    let r = call(
        &server,
        "GET",
        "/admin/break-glass",
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 403, "operator reads log: {:?}", r.text());

    // A module that has no engine-backed state is not addressable.
    let r = call(
        &server,
        "POST",
        &format!("/admin/break-glass/pakaian_dinas/{id}/transition"),
        "admin",
        ADMIN,
        Some(json!({"target_status": "SELESAI", "alasan": reason})),
    )
    .await;
    assert_eq!(r.status_code(), 400, "unsupported module: {:?}", r.text());

    drop(client);
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Delegation is a validated record, not a grant
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_delegation_is_validated_against_what_the_delegator_holds() {
    let (app, db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // The delegate must be a real account (authenc.users is a stub table here).
    let delegate = uuid::Uuid::new_v4();
    let client = db.pool().get().await.unwrap();
    client
        .execute("INSERT INTO authenc.users (id) VALUES ($1)", &[&delegate])
        .await
        .unwrap();

    let until = (chrono::Utc::now() + chrono::Duration::days(5)).to_rfc3339();
    let body = |role: &str, delegate: uuid::Uuid, reason: &str| json!({"delegate_user_id": delegate, "role": role, "valid_until": until, "reason": reason});

    // Not a role the caller holds.
    let r = call(
        &server,
        "POST",
        "/workflow/delegations",
        "operator_satker",
        OPERATOR,
        Some(body("validator_pusat", delegate, "Cuti tahunan lima hari")),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "delegating a role not held: {:?}",
        r.text()
    );

    // An administrator role is never delegable.
    let r = call(
        &server,
        "POST",
        "/workflow/delegations",
        "admin",
        ADMIN,
        Some(body("admin", delegate, "Cuti tahunan lima hari")),
    )
    .await;
    assert_eq!(r.status_code(), 400, "delegating admin: {:?}", r.text());

    // A delegate that does not exist.
    let r = call(
        &server,
        "POST",
        "/workflow/delegations",
        "operator_satker",
        OPERATOR,
        Some(body(
            "operator_satker",
            uuid::Uuid::new_v4(),
            "Cuti tahunan lima hari",
        )),
    )
    .await;
    assert_eq!(r.status_code(), 400, "unknown delegate: {:?}", r.text());

    // No written reason.
    let r = call(
        &server,
        "POST",
        "/workflow/delegations",
        "operator_satker",
        OPERATOR,
        Some(body("operator_satker", delegate, "cuti")),
    )
    .await;
    assert_eq!(r.status_code(), 400, "short reason: {:?}", r.text());

    // A valid one is recorded — and says plainly that it grants nothing yet.
    let r = call(
        &server,
        "POST",
        "/workflow/delegations",
        "operator_satker",
        OPERATOR,
        Some(body("operator_satker", delegate, "Cuti tahunan lima hari")),
    )
    .await;
    assert_eq!(r.status_code(), 201, "valid delegation: {:?}", r.text());
    assert_eq!(r.json::<Value>()["data"]["berlaku"], json!(false));

    drop(client);
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Authentication is deny-by-default
// ---------------------------------------------------------------------------

#[tokio::test]
async fn the_router_layer_refuses_every_unauthenticated_request() {
    use axum::middleware::from_fn_with_state;
    use layanan_perlengkapan::shared::grpc::clients::AuthencClient;
    use layanan_perlengkapan::shared::middleware::require_authentication;

    let (app, _db, db_name) = setup_test_app().await;
    let app = app.layer(from_fn_with_state(
        AuthencClient::dummy(),
        require_authentication,
    ));
    let server = TestServer::new(app);

    // No token: 401 on a real route, on a route that does not exist (no
    // 404-vs-401 oracle), and on a state-changing one.
    for (method, path) in [
        ("GET", "/pemakaian-bmn"),
        ("GET", "/no-such-route"),
        ("POST", "/pemakaian-bmn/auto-expire"),
        ("GET", "/admin/break-glass"),
    ] {
        let mut req = match method {
            "GET" => server.get(path),
            _ => server.post(path),
        };
        req = req.add_header(
            reqwest::header::HeaderName::from_static("x-forwarded-for"),
            reqwest::header::HeaderValue::from_static("1.2.3.4"),
        );
        let r = req.await;
        assert_eq!(
            r.status_code(),
            401,
            "{method} {path} without a token: {:?}",
            r.text()
        );
    }

    // A token that is not a Bearer credential is not one either.
    let r = server
        .get("/pemakaian-bmn")
        .add_header(
            reqwest::header::HeaderName::from_static("authorization"),
            reqwest::header::HeaderValue::from_static("Basic dXNlcjpwYXNz"),
        )
        .await;
    assert_eq!(
        r.status_code(),
        401,
        "non-bearer credential: {:?}",
        r.text()
    );

    // With a token the request reaches the handler, which reads the claims the
    // layer already validated.
    let mut req = server.get("/pemakaian-bmn");
    for (k, v) in auth_headers("operator_satker", OPERATOR, SATKER) {
        req = req.add_header(k, v);
    }
    let r = req.await;
    assert_eq!(r.status_code(), 200, "authenticated list: {:?}", r.text());

    // The WebSocket handshake cannot carry a header, so it is not turned away
    // here (it validates its own query token).
    let r = server.get("/dashboard/ws").await;
    assert_ne!(
        r.status_code(),
        401,
        "ws handshake is self-authenticating: {:?}",
        r.text()
    );

    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn a_token_cannot_assert_the_internal_system_role() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;
    submit(&server, &id).await;
    let version = permit_version(&server, &id).await;

    // `system` bypasses the engine's role table. If a token could carry it, the
    // reserved role would be a master key.
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/approver-satker-action"),
        "system",
        APPROVER,
        Some(json!({"action": "approve", "expected_version": version})),
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "system role in a token: {:?}",
        r.text()
    );
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Export: rows are confined to the caller's satker; jobs belong to their owner
// ---------------------------------------------------------------------------

#[tokio::test]
async fn an_export_contains_only_the_callers_own_satker() {
    use layanan_perlengkapan::export::models::ExportQuery;
    use layanan_perlengkapan::shared::satker_scope::SatkerScope;

    let (app, db, db_name) = setup_test_app().await;
    let _server = TestServer::new(app);
    let client = db.pool().get().await.unwrap();
    for (nip, nama, satker) in [
        ("198501012010011001", "Pegawai A", "SKR001"),
        ("198501012010011002", "Pegawai B", "SKR002"),
    ] {
        client
            .execute(
                "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama, ukuran_baju, kode_satker, status)
                 VALUES ($1, $2, 'L', $3, 'ACTIVE')",
                &[&nip, &nama, &satker],
            )
            .await
            .unwrap();
    }

    let query = |scope: SatkerScope| ExportQuery {
        entity_type: "pakaian_dinas".to_string(),
        filters: None,
        limit: Some(100),
        tahun_anggaran: None,
        satker_id: None,
        status: None,
        scope,
    };
    let nips = |rows: Vec<Value>| -> Vec<String> {
        rows.iter()
            .map(|r| r["nip"].as_str().unwrap().to_string())
            .collect()
    };

    // A satker-tier caller gets their own employees, and nobody else's.
    let own = nips(
        db.export_rows(&query(SatkerScope::Satker("SKR001".into())))
            .await
            .unwrap(),
    );
    assert_eq!(own, vec!["198501012010011001"]);

    // No identity → nothing (the fail-closed default), not everything.
    let none = db.export_rows(&query(SatkerScope::Denied)).await.unwrap();
    assert!(
        none.is_empty(),
        "a denied scope must export nothing: {none:?}"
    );

    // A cross-satker role sees both.
    let all = nips(db.export_rows(&query(SatkerScope::All)).await.unwrap());
    assert_eq!(all.len(), 2);

    drop(client);
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_export_job_is_only_visible_to_its_owner_or_an_administrator() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    // The unscopable legacy entities are cross-satker only.
    let r = call(
        &server,
        "GET",
        "/export/excel?entity_type=roadmap_sarpras",
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(
        r.status_code(),
        403,
        "operator roadmap export: {:?}",
        r.text()
    );
    let r = call(
        &server,
        "GET",
        "/export/excel?entity_type=riwayat_pemenuhan&limit=5000",
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert!(
        r.status_code() == 403 || r.status_code() == 202,
        "{}: {:?}",
        r.status_code(),
        r.text()
    );

    // Queue a large job as the operator...
    let r = call(
        &server,
        "GET",
        "/export/excel?entity_type=kebutuhan_bmn&limit=5000",
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 202, "queue: {:?}", r.text());
    let job = r.json::<Value>()["job_id"].as_str().unwrap().to_string();

    // ...someone else cannot see it, download it, or tell it exists...
    for path in [
        format!("/export/jobs/{job}/status"),
        format!("/export/jobs/{job}/download"),
    ] {
        let r = call(&server, "GET", &path, "operator_satker", VALIDATOR, None).await;
        assert_eq!(r.status_code(), 404, "stranger on {path}: {:?}", r.text());
    }
    // ...the owner and an administrator can.
    let r = call(
        &server,
        "GET",
        &format!("/export/jobs/{job}/status"),
        "operator_satker",
        OPERATOR,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 200, "owner status: {:?}", r.text());
    let r = call(
        &server,
        "GET",
        &format!("/export/jobs/{job}/status"),
        "admin",
        ADMIN,
        None,
    )
    .await;
    assert_eq!(r.status_code(), 200, "admin status: {:?}", r.text());

    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Kebutuhan BMN: who may write what
// ---------------------------------------------------------------------------

#[tokio::test]
async fn kebutuhan_writes_have_owners() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = uuid::Uuid::new_v4();

    // The campaign (periode RKBMN) is validator_pusat's to author, change,
    // delete and populate. update/delete/add-satker had no check at all.
    for role in ["operator_satker", "validator_wilayah", "admin"] {
        for (method, path, body) in [
            (
                "PUT",
                format!("/kebutuhan-bmn/pengajuan/{id}"),
                Some(json!({"nama": "Nama baru", "version": 1})),
            ),
            (
                "POST",
                format!("/kebutuhan-bmn/pengajuan/{id}/satker"),
                Some(json!({"satker_id": "SKR001"})),
            ),
        ] {
            let r = call(&server, method, &path, role, VALIDATOR, body).await;
            assert_eq!(
                r.status_code(),
                403,
                "{role} {method} {path}: {:?}",
                r.text()
            );
        }
        let mut req = server.delete(&format!("/kebutuhan-bmn/pengajuan/{id}"));
        for (k, v) in auth_headers(role, VALIDATOR, SATKER) {
            req = req.add_header(k, v);
        }
        let r = req.await;
        assert_eq!(
            r.status_code(),
            403,
            "{role} delete campaign: {:?}",
            r.text()
        );
    }

    // The satker's line items are entered by that satker's operator.
    for role in ["validator_pusat", "validator_wilayah", "admin"] {
        let r = call(
            &server,
            "POST",
            &format!("/kebutuhan-bmn/satker/{id}/barang"),
            role,
            PUSAT,
            Some(json!({"nama": "Meja", "jumlah": 1})),
        )
        .await;
        assert_eq!(r.status_code(), 403, "{role} adds barang: {:?}", r.text());
    }

    // The verdict on them (`jml_setuju`) and the ranking are the validators' — and
    // nobody else's, an operator least of all.
    for role in ["operator_satker", "admin", "approver_satker"] {
        let r = call(
            &server,
            "PUT",
            &format!("/kebutuhan-bmn/barang/{id}/approval"),
            role,
            OPERATOR,
            Some(json!({"jml_setuju": 1})),
        )
        .await;
        assert_eq!(
            r.status_code(),
            403,
            "{role} sets jml_setuju: {:?}",
            r.text()
        );
        let r = call(
            &server,
            "POST",
            "/kebutuhan-bmn/prioritas",
            role,
            OPERATOR,
            Some(json!({"items": []})),
        )
        .await;
        assert_eq!(
            r.status_code(),
            403,
            "{role} sets prioritas: {:?}",
            r.text()
        );
    }

    // The bulk endpoints reached the engine with no handler-side check.
    for role in ["approver_satker", "admin", "validator_satker"] {
        for path in ["approve", "reject", "update-status"] {
            let r = call(&server, "POST", &format!("/kebutuhan-bmn/batch/{path}"), role, OPERATOR,
                Some(json!({"kebutuhan_ids": [id], "target_status": 2003, "komentar": "Alasan yang cukup panjang"}))).await;
            assert_eq!(r.status_code(), 403, "{role} batch {path}: {:?}", r.text());
        }
    }
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn pakaian_roster_writes_are_the_operators() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = uuid::Uuid::new_v4();

    for role in ["validator_pusat", "validator_wilayah", "admin"] {
        let r = call(
            &server,
            "POST",
            "/pakaian-dinas/pegawai-profile",
            role,
            PUSAT,
            Some(json!({"nip": "198501012010011001", "kode_satker": "SKR001"})),
        )
        .await;
        assert_eq!(
            r.status_code(),
            403,
            "{role} profile upsert: {:?}",
            r.text()
        );
        let r = call(
            &server,
            "POST",
            "/pakaian-dinas/pegawai-profile/bulk",
            role,
            PUSAT,
            Some(json!([])),
        )
        .await;
        assert_eq!(r.status_code(), 403, "{role} bulk upsert: {:?}", r.text());
        let r = call(
            &server,
            "PUT",
            &format!("/pakaian-dinas/pengajuan/{id}/satker/SKR001/pegawai/198501012010011001"),
            role,
            PUSAT,
            Some(json!({"with_hijab": false, "ukuran": []})),
        )
        .await;
        assert_eq!(r.status_code(), 403, "{role} sizes: {:?}", r.text());
    }
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Data that cannot be confined to a satker is for the roles that see all of them
// ---------------------------------------------------------------------------

#[tokio::test]
async fn national_analytics_and_monitoring_are_not_for_everyone() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);

    for path in [
        "/forecast",
        "/forecast/summary",
        "/forecast/compare",
        "/forecast/export",
    ] {
        let r = call(&server, "GET", path, "operator_satker", OPERATOR, None).await;
        assert_eq!(r.status_code(), 403, "operator {path}: {:?}", r.text());
    }
    for path in [
        "/workflow/monitoring/metrics",
        "/workflow/monitoring/active",
        "/workflow/monitoring/sla-breaches",
        "/workflow/monitoring/bottlenecks",
    ] {
        for role in ["operator_satker", "validator_wilayah"] {
            let r = call(&server, "GET", path, role, OPERATOR, None).await;
            assert_eq!(r.status_code(), 403, "{role} {path}: {:?}", r.text());
        }
        let r = call(&server, "GET", path, "admin", ADMIN, None).await;
        assert_eq!(r.status_code(), 200, "admin {path}: {:?}", r.text());
    }
    teardown_test_db(&db_name).await;
}

#[tokio::test]
async fn an_analysis_belongs_to_the_satker_that_wrote_it() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let body = json!({
        "judul": "Analisis milik SKR001", "kategori": "TIK", "prioritas": "tinggi",
        "estimasi_biaya": 1000000, "justifikasi": "Perangkat sudah usang"
    });

    // Written by an operator of SKR001...
    let mut req = server.post("/analisis").json(&body);
    for (k, v) in auth_headers("operator_satker", OPERATOR, "SKR001") {
        req = req.add_header(k, v);
    }
    let created = req.await;
    assert_eq!(created.status_code(), 201, "create: {:?}", created.text());
    let id = created.json::<Value>()["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // ...not by anyone else.
    let r = call(
        &server,
        "POST",
        "/analisis",
        "validator_pusat",
        PUSAT,
        Some(body.clone()),
    )
    .await;
    assert_eq!(r.status_code(), 403, "pusat create: {:?}", r.text());

    let get_as = |role: &'static str, user: &'static str, satker: &'static str, path: String| {
        let server = &server;
        async move {
            let mut req = server.get(&path);
            for (k, v) in auth_headers(role, user, satker) {
                req = req.add_header(k, v);
            }
            req.await
        }
    };

    // An operator of another satker neither lists it nor reads it by id.
    let r = get_as(
        "operator_satker",
        VALIDATOR,
        "SKR002",
        "/analisis?page=1&per_page=50".into(),
    )
    .await;
    assert_eq!(r.status_code(), 200);
    assert!(
        r.json::<Value>()["data"].as_array().unwrap().is_empty(),
        "leaked into SKR002's list"
    );
    let r = get_as(
        "operator_satker",
        VALIDATOR,
        "SKR002",
        format!("/analisis/{id}"),
    )
    .await;
    assert_eq!(r.status_code(), 404, "other satker by id: {:?}", r.text());

    // The owner and a cross-satker role do.
    let r = get_as(
        "operator_satker",
        OPERATOR,
        "SKR001",
        "/analisis?page=1&per_page=50".into(),
    )
    .await;
    assert_eq!(r.json::<Value>()["data"].as_array().unwrap().len(), 1);
    let r = get_as(
        "validator_pusat",
        PUSAT,
        "PUSAT001",
        format!("/analisis/{id}"),
    )
    .await;
    assert_eq!(r.status_code(), 200, "pusat by id: {:?}", r.text());
    teardown_test_db(&db_name).await;
}

// ---------------------------------------------------------------------------
// Stored links and uploaded files
// ---------------------------------------------------------------------------

#[tokio::test]
async fn a_stored_document_link_must_be_a_web_url() {
    let (app, _db, db_name) = setup_test_app().await;
    let server = TestServer::new(app);
    let id = create_permit(&server).await;

    // `javascript:` stored as a link runs when the reviewing validator clicks it.
    for bad in [
        "javascript:alert(document.cookie)",
        "data:text/html;base64,PHNjcmlwdD4=",
        "file:///etc/passwd",
        "//evil.example/x.pdf",
    ] {
        let r = call(
            &server,
            "POST",
            &format!("/pemakaian-bmn/{id}/upload-signed-pdf"),
            "operator_satker",
            OPERATOR,
            Some(json!({"signed_pdf_url": bad})),
        )
        .await;
        assert_eq!(r.status_code(), 400, "{bad}: {:?}", r.text());
    }
    // (A well-formed URL passes validation; whether the permit is in a state to
    // receive it is a separate, later check.)
    let r = call(
        &server,
        "POST",
        &format!("/pemakaian-bmn/{id}/upload-signed-pdf"),
        "operator_satker",
        OPERATOR,
        Some(json!({"signed_pdf_url": "https://storage.example.com/s.pdf"})),
    )
    .await;
    assert_ne!(r.status_code(), 403, "{:?}", r.text());
    assert_ne!(r.status_code(), 401);
    teardown_test_db(&db_name).await;
}
