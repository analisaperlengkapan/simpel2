//! A request value must never become part of a SQL statement.
//!
//! Found by reading the code before the first production install (2026-11-12).
//! `/api/monitoring/sync-status` and `/api/monitoring/sync-history` built their statement with
//! `format!("... WHERE module = '{}' ...", source)`, where `source` is the query string of a
//! request nobody has to authenticate for. `?source=x' OR '1'='1` answered with every module; a
//! `UNION SELECT` answered with the contents of `integrasi.monsakti_tokens` — the table that holds
//! the bearer tokens of the Ministry of Finance API.
//!
//! These tests serve the real router over HTTP against a real Postgres that was migrated by the
//! real `integrasi-migrate` binary, and send the payloads. They also pin the numbers the endpoints
//! are supposed to answer with: the same file read `COUNT(*)` (a `bigint`) into an `i32` behind
//! `try_get(..).unwrap_or(0)`, so `consecutive_failures` was 0 in every alert it ever produced.
//!
//! `DATABASE_URL` is REQUIRED and a missing or unreachable database FAILS the test (see
//! `db_reconnect.rs` for why a test that skips itself is worth nothing here).

use layanan_integrasi::db_handle::DbHandle;
use layanan_integrasi::monitoring::{
    MAX_HISTORY_DAYS, MAX_HISTORY_LIMIT, MonitoringState, create_monitoring_router,
};
use serde_json::Value;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio_postgres::NoTls;

/// What an attacker would be after. It sits in `monsakti_tokens.token_value`, a column no
/// monitoring endpoint has any reason to read.
const CANARY: &str = "CANARY-kR7vQm2ZxLp9TfWc4HsYb8Nd";

fn admin_url() -> String {
    std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must point at a Postgres for this test \
         (locally: postgres://simpel:simpel@localhost:5456/postgres)",
    )
}

/// The same server, another database: swap the path of the URL, keep its query string.
fn url_for(admin: &str, database: &str) -> String {
    let (base, tail) = admin
        .rsplit_once('/')
        .expect("DATABASE_URL must look like postgres://user:pass@host:port/database");
    let query = tail
        .split_once('?')
        .map(|(_, q)| format!("?{q}"))
        .unwrap_or_default();
    format!("{base}/{database}{query}")
}

/// A database of its own, migrated like production's, with the router listening on a local port.
struct Fixture {
    admin: String,
    database: String,
    base: String,
    http: reqwest::Client,
}

impl Drop for Fixture {
    /// Runs on a failed assertion too. A thread of its own, because a runtime cannot be blocked on
    /// from inside the test's runtime.
    fn drop(&mut self) {
        let (admin, database) = (self.admin.clone(), self.database.clone());
        let _ = std::thread::spawn(move || {
            let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            rt.block_on(async {
                if let Ok((client, connection)) = tokio_postgres::connect(&admin, NoTls).await {
                    tokio::spawn(connection);
                    let _ = client
                        .batch_execute(&format!("DROP DATABASE IF EXISTS {database} WITH (FORCE)"))
                        .await;
                }
            });
        })
        .join();
    }
}

impl Fixture {
    async fn new() -> Self {
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let admin = admin_url();
        // Lower-case letters, digits and underscores only: it is spliced into CREATE DATABASE.
        let database = format!(
            "integrasi_mon_{}_{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::SeqCst)
        );

        let (client, connection) = tokio_postgres::connect(&admin, NoTls)
            .await
            .expect("connect to DATABASE_URL");
        tokio::spawn(connection);
        client
            .batch_execute(&format!("CREATE DATABASE {database}"))
            .await
            .expect("CREATE DATABASE");
        let url = url_for(&admin, &database);

        // The schema is whatever the real migration binary produces — not a table typed out here.
        let migrate = Command::new(env!("CARGO_BIN_EXE_integrasi-migrate"))
            .env_clear()
            .env("DATABASE_URL", &url)
            .output()
            .expect("integrasi-migrate must start");
        assert!(
            migrate.status.success(),
            "integrasi-migrate failed on an empty database:\n{}{}",
            String::from_utf8_lossy(&migrate.stdout),
            String::from_utf8_lossy(&migrate.stderr)
        );

        let (seed, connection) = tokio_postgres::connect(&url, NoTls)
            .await
            .expect("connect to the test database");
        tokio::spawn(connection);
        seed.batch_execute(SEED)
            .await
            .expect("seed integrasi.api_log");
        seed.execute(
            "INSERT INTO integrasi.monsakti_tokens (module, token_value) VALUES ('ADM', $1)",
            &[&CANARY],
        )
        .await
        .expect("seed the canary");

        let db = DbHandle::connect(&url).await.expect("DbHandle::connect");
        let app = create_monitoring_router(MonitoringState { db_client: db });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind a local port");
        let base = format!("http://{}", listener.local_addr().expect("local addr"));
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        Fixture {
            admin,
            database,
            base,
            http: reqwest::Client::new(),
        }
    }

    /// GET `path` with `params` percent-encoded by reqwest; the status and the body, whatever it is.
    async fn get(&self, path: &str, params: &[(&str, &str)]) -> (u16, String) {
        let response = self
            .http
            .get(format!("{}{}", self.base, path))
            .query(params)
            .send()
            .await
            .expect("the request must reach the router");
        let status = response.status().as_u16();
        (status, response.text().await.expect("a body"))
    }

    /// GET that must answer 200 with JSON.
    async fn json(&self, path: &str, params: &[(&str, &str)]) -> Value {
        let (status, body) = self.get(path, params).await;
        assert_eq!(
            status, 200,
            "GET {path} {params:?} answered {status}: {body}"
        );
        serde_json::from_str(&body).unwrap_or_else(|e| panic!("GET {path}: not JSON ({e}): {body}"))
    }
}

/// `siman`: three calls in the last three hours — two succeeded (100 and 50 records, 30 s and 45 s),
/// one failed. `mysimkari`: one success ten days ago, outside the default 7-day window.
/// `monsakti`: four failures in the last hour and nothing else.
const SEED: &str = r#"
INSERT INTO integrasi.api_log (module, endpoint, record_count, success, error_message, started_at, completed_at) VALUES
  ('siman',     'aset/tanah',  100,  true,  NULL,               now() - interval '3 hours',    now() - interval '3 hours' + interval '30 seconds'),
  ('siman',     'aset/gedung', 50,   true,  NULL,               now() - interval '2 hours',    now() - interval '2 hours' + interval '45 seconds'),
  ('siman',     'aset/kdp',    NULL, false, 'upstream timeout', now() - interval '1 hour',     NULL),
  ('mysimkari', 'satker',      7,    true,  NULL,               now() - interval '10 days',    now() - interval '10 days' + interval '2 seconds'),
  ('monsakti',  'adm/ref',     NULL, false, 'token expired',    now() - interval '40 minutes', NULL),
  ('monsakti',  'adm/ref',     NULL, false, 'token expired',    now() - interval '30 minutes', NULL),
  ('monsakti',  'adm/ref',     NULL, false, 'token expired',    now() - interval '20 minutes', NULL),
  ('monsakti',  'adm/ref',     NULL, false, 'token expired',    now() - interval '10 minutes', NULL);
"#;

fn sources(rows: &Value) -> Vec<String> {
    rows.as_array()
        .unwrap_or_else(|| panic!("expected a JSON array, got {rows}"))
        .iter()
        .map(|r| r["source"].as_str().unwrap_or("<no source>").to_string())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// What the endpoints are FOR. If these fail, the payload tests below prove nothing: an endpoint
// that answers `[]` to everything also "resists" every injection.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn sync_status_reports_each_module_with_its_own_numbers() {
    let fx = Fixture::new().await;

    let all = fx.json("/api/monitoring/sync-status", &[]).await;
    assert_eq!(sources(&all), ["monsakti", "mysimkari", "siman"]);

    let siman = fx
        .json("/api/monitoring/sync-status", &[("source", "siman")])
        .await;
    assert_eq!(sources(&siman), ["siman"]);
    let row = &siman[0];
    assert_eq!(row["records_synced"], 150, "{row}");
    let rate = row["success_rate"].as_f64().expect("success_rate");
    assert!((rate - 200.0 / 3.0).abs() < 1e-9, "2 of 3 succeeded: {row}");
    assert_eq!(row["state"], "degraded", "{row}");
    assert!(
        row["last_sync_at"].as_str().is_some_and(|s| s.len() >= 20),
        "last_sync_at must be a timestamp: {row}"
    );

    let none = fx
        .json("/api/monitoring/sync-status", &[("source", "tidak-ada")])
        .await;
    assert_eq!(none, serde_json::json!([]));
}

#[tokio::test]
async fn sync_history_filters_by_module_window_and_limit() {
    let fx = Fixture::new().await;

    // Default window (7 days): the mysimkari call of ten days ago is out.
    let all = fx.json("/api/monitoring/sync-history", &[]).await;
    assert_eq!(all.as_array().map(Vec::len), Some(7), "{all}");
    assert!(!sources(&all).contains(&"mysimkari".to_string()));

    // Newest first, with the values of the row and not their defaults.
    let siman = fx
        .json("/api/monitoring/sync-history", &[("source", "siman")])
        .await;
    let endpoints: Vec<&str> = siman
        .as_array()
        .expect("array")
        .iter()
        .map(|r| r["endpoint"].as_str().unwrap_or(""))
        .collect();
    assert_eq!(endpoints, ["aset/kdp", "aset/gedung", "aset/tanah"]);
    let failed = &siman[0];
    assert_eq!(failed["success"], false, "{failed}");
    assert_eq!(failed["error_message"], "upstream timeout", "{failed}");
    assert_eq!(failed["records_fetched"], 0, "{failed}");
    assert_eq!(failed["duration_seconds"], Value::Null, "{failed}");
    let gedung = &siman[1];
    assert_eq!(gedung["success"], true, "{gedung}");
    assert_eq!(gedung["records_fetched"], 50, "{gedung}");
    assert_eq!(gedung["duration_seconds"], 45, "{gedung}");
    assert_eq!(gedung["error_message"], Value::Null, "{gedung}");
    assert!(gedung["id"].as_i64().is_some_and(|id| id > 0), "{gedung}");
    assert!(
        gedung["sync_started_at"]
            .as_str()
            .is_some_and(|s| s.len() >= 20),
        "{gedung}"
    );

    // A wider window lets the old call in; a limit cuts the list.
    let wide = fx
        .json("/api/monitoring/sync-history", &[("days", "30")])
        .await;
    assert_eq!(wide.as_array().map(Vec::len), Some(8), "{wide}");
    let two = fx
        .json("/api/monitoring/sync-history", &[("limit", "2")])
        .await;
    assert_eq!(two.as_array().map(Vec::len), Some(2), "{two}");
    assert_eq!(sources(&two), ["monsakti", "monsakti"], "the two newest");
}

#[tokio::test]
async fn alerts_count_the_failures_they_announce() {
    let fx = Fixture::new().await;
    let alerts = fx.json("/api/monitoring/alerts", &[]).await;

    // monsakti: 4 failures in 24 h, all 4 among its 5 latest calls. siman: 1 failure — no alert.
    assert_eq!(sources(&alerts), ["monsakti"], "{alerts}");
    let alert = &alerts[0];
    assert_eq!(
        alert["consecutive_failures"], 4,
        "a bigint read as i32 behind unwrap_or(0) said 0 here: {alert}"
    );
    assert_eq!(alert["severity"], "high", "{alert}");
    assert_eq!(
        alert["message"], "4 consecutive sync failures for monsakti",
        "{alert}"
    );
    assert_eq!(alert["error_details"], "token expired", "{alert}");
}

#[tokio::test]
async fn dashboard_agrees_with_the_endpoints_it_summarises() {
    let fx = Fixture::new().await;
    let dash = fx.json("/api/monitoring/dashboard", &[]).await;

    // Every seeded call but the ten-day-old one is younger than three hours. Run within three
    // hours after local midnight some of them fall on "yesterday", so today's totals are bounded,
    // not pinned; what must always hold is that they add up.
    let total = dash["total_syncs_today"].as_i64().expect("total");
    let ok = dash["successful_syncs_today"].as_i64().expect("successful");
    let failed = dash["failed_syncs_today"].as_i64().expect("failed");
    assert_eq!(total, ok + failed, "{dash}");
    assert!((0..=7).contains(&total), "{dash}");

    let by_source = dash["sync_status_by_source"]
        .as_object()
        .expect("sync_status_by_source");
    let mut modules: Vec<&str> = by_source.keys().map(String::as_str).collect();
    modules.sort_unstable();
    assert_eq!(modules, ["monsakti", "mysimkari", "siman"]);
    assert_eq!(by_source["siman"]["records_synced"], 150);

    // "The 10 latest calls, whatever their age": all eight, the old one included.
    assert_eq!(
        dash["recent_sync_history"].as_array().map(Vec::len),
        Some(8),
        "{dash}"
    );
    assert_eq!(
        dash["active_alerts"][0]["consecutive_failures"], 4,
        "{dash}"
    );
}

// ---------------------------------------------------------------------------------------------
// The payloads. Each one is a `source` that names no module, so the only right answer is `[]`.
// ---------------------------------------------------------------------------------------------

#[tokio::test]
async fn a_tautology_in_source_matches_no_module() {
    let fx = Fixture::new().await;
    for path in [
        "/api/monitoring/sync-status",
        "/api/monitoring/sync-history",
    ] {
        let (status, body) = fx.get(path, &[("source", "x' OR '1'='1")]).await;
        assert_eq!(
            (status, body.as_str()),
            (200, "[]"),
            "{path}: `x' OR '1'='1` is not the name of a module"
        );
    }
}

#[tokio::test]
async fn a_union_in_source_does_not_read_the_token_table() {
    let fx = Fixture::new().await;

    // sync-status: five columns of the right types. The payload closes the first SELECT with a
    // GROUP BY of its own, because the statement's GROUP BY — on the line after the comment —
    // lands on the injected SELECT. The token came back in the `source` field.
    let payload = "x' GROUP BY module UNION SELECT max(token_value), now(), 0, 0, 0 \
                   FROM integrasi.monsakti_tokens --";
    let (status, body) = fx
        .get("/api/monitoring/sync-status", &[("source", payload)])
        .await;
    assert!(
        !body.contains("CANARY"),
        "sync-status answered with a row of integrasi.monsakti_tokens: {body}"
    );
    assert_eq!((status, body.as_str()), (200, "[]"));
}

#[tokio::test]
async fn source_cannot_ask_yes_or_no_questions_about_the_token_table() {
    let fx = Fixture::new().await;

    // sync-history ends in ORDER BY started_at, which rules a UNION out. The token is read one
    // question at a time instead: rows came back if and only if the guess about it was right.
    for (guess, what) in [("CANARY%", "a right guess"), ("WRONG%", "a wrong guess")] {
        let oracle = format!(
            "x' OR EXISTS (SELECT 1 FROM integrasi.monsakti_tokens \
             WHERE token_value LIKE '{guess}') OR '1'='2"
        );
        let (status, body) = fx
            .get(
                "/api/monitoring/sync-history",
                &[("source", oracle.as_str())],
            )
            .await;
        assert_eq!(
            (status, body.as_str()),
            (200, "[]"),
            "sync-history told {what} about integrasi.monsakti_tokens apart from no answer"
        );
    }
}

#[tokio::test]
async fn a_quote_in_source_is_data_not_a_syntax_error() {
    let fx = Fixture::new().await;
    for path in [
        "/api/monitoring/sync-status",
        "/api/monitoring/sync-history",
    ] {
        for source in ["o'brien", "'", "\\", "a\"b", "siman'--", ";", "%", ""] {
            let (status, body) = fx.get(path, &[("source", source)]).await;
            assert_eq!(
                (status, body.as_str()),
                (200, "[]"),
                "{path} with source={source:?}"
            );
        }
    }
}

#[tokio::test]
async fn days_and_limit_outside_their_bounds_are_refused_not_executed() {
    let fx = Fixture::new().await;
    let too_many_days = (i64::from(MAX_HISTORY_DAYS) + 1).to_string();
    let too_many_rows = (MAX_HISTORY_LIMIT + 1).to_string();
    for (name, value) in [
        ("limit", "-1"),
        ("limit", "0"),
        ("limit", too_many_rows.as_str()),
        ("days", "-1"),
        ("days", "0"),
        ("days", too_many_days.as_str()),
        ("days", "2147483647"),
    ] {
        let (status, body) = fx
            .get("/api/monitoring/sync-history", &[(name, value)])
            .await;
        assert_eq!(status, 400, "{name}={value} answered {status}: {body}");
        assert!(
            body.contains(&format!("{name} must be between 1 and")),
            "{name}={value}: the refusal must say which bound: {body}"
        );
    }

    // The bounds themselves are served.
    let max_days = MAX_HISTORY_DAYS.to_string();
    let max_rows = MAX_HISTORY_LIMIT.to_string();
    let widest = fx
        .json(
            "/api/monitoring/sync-history",
            &[("days", max_days.as_str()), ("limit", max_rows.as_str())],
        )
        .await;
    assert_eq!(widest.as_array().map(Vec::len), Some(8), "{widest}");
}

#[tokio::test]
async fn a_failed_query_does_not_recite_the_database_error() {
    let fx = Fixture::new().await;
    // Take the table away from under the running server: every query now fails for a reason that
    // names a relation.
    let (client, connection) = tokio_postgres::connect(&url_for(&fx.admin, &fx.database), NoTls)
        .await
        .expect("connect");
    tokio::spawn(connection);
    client
        .batch_execute("ALTER TABLE integrasi.api_log RENAME TO api_log_gone")
        .await
        .expect("rename");

    for path in [
        "/api/monitoring/sync-status",
        "/api/monitoring/sync-history",
        "/api/monitoring/alerts",
    ] {
        let (status, body) = fx.get(path, &[]).await;
        assert_eq!(status, 500, "{path}: {body}");
        let json: Value = serde_json::from_str(&body).expect("a JSON error body");
        assert!(json["error"].is_string(), "{path}: {body}");
        // `details` used to carry `e.to_string()`. With this tokio-postgres that is the two words
        // "db error"; what it says after an upgrade of the driver is not ours to decide, and the
        // caller is unauthenticated. The reason goes to the log.
        assert!(
            json.get("details").is_none() && !body.contains("api_log"),
            "{path} passed the database error on to the caller: {body}"
        );
    }
}
