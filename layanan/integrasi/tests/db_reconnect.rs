//! A server must outlive its database connection.
//!
//! Found on staging (rc46, 2026-11-11): the integrasi server held ONE `tokio_postgres::Client`; one
//! hour after start its connection closed (no traffic, no restart of Postgres) and from then on
//! every query answered `connection closed` and `/api/monitoring/health` answered 503, until the
//! pod was restarted. The pod stayed `Ready` and 218 e2e tests passed.
//!
//! These need a real Postgres: `DATABASE_URL` is REQUIRED, and a missing or unreachable database
//! FAILS the test. A test that skips itself when the database is absent passes in exactly the
//! environment that cannot catch this.

use layanan_integrasi::db_handle::DbHandle;
use tokio_postgres::NoTls;

fn database_url() -> String {
    std::env::var("DATABASE_URL").expect(
        "DATABASE_URL must point at a Postgres for this test \
         (locally: postgres://simpel:simpel@localhost:5456/postgres)",
    )
}

/// Terminate, from a second connection, the backend that serves `pid`.
async fn kill_backend(url: &str, pid: i32) {
    let (admin, connection) = tokio_postgres::connect(url, NoTls)
        .await
        .expect("a second connection to terminate the first one");
    tokio::spawn(connection);
    let terminated: bool = admin
        .query_one("SELECT pg_terminate_backend($1)", &[&pid])
        .await
        .expect("pg_terminate_backend")
        .get(0);
    assert!(terminated, "Postgres refused to terminate backend {pid}");
}

#[tokio::test]
async fn control_a_bare_client_never_recovers_once_its_connection_is_gone() {
    // What the server did before `DbHandle`: it proves this test really kills the connection, and
    // it is the behaviour the handle exists to remove.
    let url = database_url();
    let (client, connection) = tokio_postgres::connect(&url, NoTls).await.expect("connect");
    tokio::spawn(connection);
    let pid: i32 = client
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .expect("first query")
        .get(0);

    kill_backend(&url, pid).await;

    for attempt in 1..=3 {
        client
            .query_one("SELECT 1", &[])
            .await
            .expect_err("a bare client must fail after its backend was terminated");
        // The closed flag lags the first error (the server sends 57P01 before it closes the
        // socket), so give the connection task a moment before asserting it.
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(
            client.is_closed(),
            "attempt {attempt}: a bare client never comes back from this"
        );
    }
}

#[tokio::test]
async fn a_handle_reconnects_after_its_backend_was_terminated() {
    let url = database_url();
    let db = DbHandle::connect(&url).await.expect("connect");
    let pid: i32 = db
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .expect("first query")
        .get(0);

    kill_backend(&url, pid).await;

    // The very next call: the closed flag may not have propagated yet (the first attempt then
    // fails with the connection and is retried once), or it already has (reconnect at once).
    let one: i32 = db
        .query_one("SELECT 1::int", &[])
        .await
        .expect("the handle must reconnect and answer")
        .get(0);
    assert_eq!(one, 1);

    let new_pid: i32 = db
        .query_one("SELECT pg_backend_pid()", &[])
        .await
        .expect("second query")
        .get(0);
    assert_ne!(pid, new_pid, "a NEW backend must now serve the handle");
}

#[tokio::test]
async fn a_handle_keeps_working_across_repeated_terminations() {
    let url = database_url();
    let db = DbHandle::connect(&url).await.expect("connect");
    for round in 1..=3 {
        let pid: i32 = db
            .query_one("SELECT pg_backend_pid()", &[])
            .await
            .unwrap_or_else(|e| panic!("round {round}: query before termination: {e}"))
            .get(0);
        kill_backend(&url, pid).await;
        let rows = db
            .query("SELECT generate_series(1, 3)::int", &[])
            .await
            .unwrap_or_else(|e| panic!("round {round}: query after termination: {e}"));
        assert_eq!(rows.len(), 3, "round {round}");
    }
    // simple_query is the health check's call.
    let msgs = db.simple_query("SELECT 1").await.expect("simple_query");
    assert!(!msgs.is_empty());
}

#[tokio::test]
async fn a_database_that_is_down_is_an_error_not_a_hang() {
    // Nothing listens on 127.0.0.1:9: the first connect must fail fast and say so.
    let started = std::time::Instant::now();
    let result = DbHandle::connect("postgres://nobody:nothing@127.0.0.1:9/none").await;
    assert!(result.is_err(), "connecting to a closed port must fail");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(15),
        "the connect attempt took {:?}; it must be bounded",
        started.elapsed()
    );
}
