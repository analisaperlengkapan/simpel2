//! Port 8080 of layanan-integrasi answers health and nothing else.
//!
//! Four monitoring endpoints used to answer here without authentication and with CORS open to any
//! origin: `sync-status`, `sync-history`, `alerts`, `dashboard`. They had no caller in the repo,
//! the ingress did not route to them, and what they served (`integrasi.api_log`) could not show a
//! sync that failed before an HTTP answer. They were removed on 2026-11-12; sync outcomes are in
//! `integrasi.sync_status` and shown to administrators by perlengkapan. See `src/monitoring.rs`.
//!
//! `DATABASE_URL` is REQUIRED (the health answer is about the database handle).

use layanan_integrasi::db_handle::DbHandle;
use layanan_integrasi::monitoring::{MonitoringState, create_monitoring_router};

async fn serve() -> String {
    let url = std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must point at a Postgres for this test \
         (locally: postgres://simpel:simpel@localhost:5456/postgres)",
    );
    let db = DbHandle::connect(&url).await.expect("DbHandle::connect");
    let app = create_monitoring_router(MonitoringState { db_client: db });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind a local port");
    let base = format!("http://{}", listener.local_addr().expect("local addr"));
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    base
}

#[tokio::test]
async fn health_answers_and_the_removed_endpoints_do_not() {
    let base = serve().await;
    let http = reqwest::Client::new();

    let health = http
        .get(format!("{base}/api/monitoring/health"))
        .send()
        .await
        .expect("health reachable");
    assert_eq!(health.status().as_u16(), 200);
    let body: serde_json::Value = health.json().await.expect("json");
    assert_eq!(body["status"], "healthy", "{body}");

    for path in [
        "/api/monitoring/sync-status",
        "/api/monitoring/sync-history",
        "/api/monitoring/alerts",
        "/api/monitoring/dashboard",
    ] {
        let status = http
            .get(format!("{base}{path}"))
            .send()
            .await
            .expect("reachable")
            .status()
            .as_u16();
        assert_eq!(status, 404, "{path} answered {status}");
    }
}
