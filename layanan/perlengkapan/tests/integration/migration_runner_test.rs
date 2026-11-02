//! A rolled-back release must be able to start on a newer schema.
//!
//! `helm rollback` runs the OLDER binary against a database the newer release
//! has already migrated. refinery's default (`abort_missing = true`) turns the
//! migrations that binary has never heard of into `MissingVersion`, and
//! `main` propagates that error — so the pod exits before serving and the
//! expand/contract rule in layanan/AGENTS.md ("agar helm rollback aman tanpa
//! restore DB") protected nothing. This is the only test that exercises the
//! runner itself: every other integration test applies the SQL files directly,
//! bypassing refinery and its history table.

use crate::common::{setup_test_db, teardown_test_db};

#[tokio::test]
async fn the_runner_starts_on_a_schema_newer_than_itself() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

    // `setup_test_db` applied the files directly; start from nothing so that
    // refinery builds its own history, as it does on a real deploy. The
    // baseline spreads over several schemas (`perlengkapan`, `dokumen`,
    // `bantuan`, `notifikasi`, and `public`), so every schema it made goes —
    // everything except the two upstream stubs the harness stands up.
    client
        .batch_execute(
            "DO $$ DECLARE s text; BEGIN \
               FOR s IN SELECT nspname FROM pg_namespace \
                        WHERE nspname NOT LIKE 'pg\\_%' \
                          AND nspname NOT IN ('information_schema', 'authenc', 'integrasi') \
               LOOP EXECUTE format('DROP SCHEMA %I CASCADE', s); END LOOP; \
             END $$; \
             CREATE SCHEMA public;",
        )
        .await
        .unwrap();

    layanan_perlengkapan::migrations::run(db.pool())
        .await
        .expect("a fresh database must migrate");

    // What the older binary sees after the newer release ran: one applied
    // migration it has no file for.
    client
        .batch_execute(
            "INSERT INTO refinery_schema_history (version, name, applied_on, checksum) \
             VALUES (9999, 'from_a_newer_release', '2030-01-01T00:00:00Z', '0')",
        )
        .await
        .unwrap();

    layanan_perlengkapan::migrations::run(db.pool())
        .await
        .expect(
            "a binary older than the schema must start (log the unknown migration, \
             do not abort) — otherwise helm rollback is a crash loop",
        );

    teardown_test_db(&db_name).await;
}
