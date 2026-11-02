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

/// The real runner repairs rows that predate V014's triggers, on a start where
/// refinery skips V014.
///
/// V014 is already in the history, so refinery does nothing — and anything the
/// normaliser has to fix must still be fixed before the service serves. The
/// triggers are switched off while the legacy rows are written because with
/// them on every legacy insert is normalised at once (that is what
/// `legacy_writes_after_startup_are_readable_at_once` proves); this is the
/// other half, the rows nothing fired for. Driven through the second `run`,
/// not through the SQL file: an earlier version of this claim was "tested" by
/// re-running the file, which is not what the runner does.
#[tokio::test]
async fn the_runner_repairs_rows_that_predate_the_triggers_although_refinery_skips_v014() {
    let (db, db_name) = setup_test_db().await;
    let client = db.pool().get().await.unwrap();

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

    // The deploy that introduced V014.
    layanan_perlengkapan::migrations::run(db.pool())
        .await
        .expect("fresh database must migrate");

    // Rows written in the previous release's shapes with nothing to normalise
    // them — what existed before V014's triggers did.
    client
        .batch_execute(
            "ALTER TABLE perlengkapan.izin_pemakaian_bmn \
               DISABLE TRIGGER trg_izin_pemakaian_legacy_item_one; \
             ALTER TABLE perlengkapan.pemakaian_bmn_items \
               DISABLE TRIGGER trg_pemakaian_items_legacy_defaults; \
             INSERT INTO perlengkapan.izin_pemakaian_bmn \
               (id, jenis_bmn, bmn_nup, bmn_kode_barang, bmn_nama_barang, \
                pegawai_nip, pegawai_nama, pegawai_satker_id, tanggal_mulai, tanggal_selesai, status) \
             VALUES ('a0000000-0000-4000-8000-0000000000d1', 'LAPTOP', '601', '3060201003', \
                'Laptop Induk', '198501012010011001', 'Budi', \
                '00000000-0000-0000-0000-000000000001', '2026-01-01', '2026-12-31', 'DRAFT'); \
             INSERT INTO perlengkapan.pemakaian_bmn_items \
               (izin_pemakaian_id, bmn_nup, bmn_kode_barang, bmn_nama_barang, created_at, updated_at) \
             VALUES ('a0000000-0000-4000-8000-0000000000d1', '602', '3060201003', \
                'Laptop Tambahan', now(), now()); \
             ALTER TABLE perlengkapan.izin_pemakaian_bmn \
               ENABLE TRIGGER trg_izin_pemakaian_legacy_item_one; \
             ALTER TABLE perlengkapan.pemakaian_bmn_items \
               ENABLE TRIGGER trg_pemakaian_items_legacy_defaults;",
        )
        .await
        .unwrap();
    let nulls: i64 = client
        .query_one(
            "SELECT count(*) FROM perlengkapan.pemakaian_bmn_items WHERE urutan IS NULL",
            &[],
        )
        .await
        .unwrap()
        .get(0);
    assert_eq!(nulls, 1, "precondition: the legacy item has no position");

    // Roll forward. V014 is in refinery's history, so the runner skips it.
    layanan_perlengkapan::migrations::run(db.pool())
        .await
        .expect("roll-forward start");

    let rows = client
        .query(
            "SELECT urutan, jenis_bmn, bmn_nup, tanggal_mulai::text \
               FROM perlengkapan.pemakaian_bmn_items \
              WHERE izin_pemakaian_id = 'a0000000-0000-4000-8000-0000000000d1' ORDER BY urutan",
            &[],
        )
        .await
        .unwrap();
    let got: Vec<(i32, String, String, String)> = rows
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3)))
        .collect();
    assert_eq!(
        got,
        vec![
            (
                1,
                "LAPTOP".to_string(),
                "601".to_string(),
                "2026-01-01".to_string()
            ),
            (
                2,
                "LAINNYA".to_string(),
                "602".to_string(),
                "2026-01-01".to_string()
            ),
        ],
        "the parent's asset is slot 1, the legacy item follows with a position \
         and a period, and nothing is left NULL for from_row to panic on"
    );

    teardown_test_db(&db_name).await;
}
