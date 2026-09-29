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
