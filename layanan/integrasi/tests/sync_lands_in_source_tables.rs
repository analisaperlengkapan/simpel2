//! A sync must land in the tables the application reads, once per record.
//!
//! Every CronJob runs `./scheduler --source <provider>`. These tests run that binary against a
//! local HTTP server that answers like MySIMKARI and SIMAN, and against a database that the real
//! `integrasi-migrate` migrated. They then count rows in the tables the rest of the system reads:
//!
//! * `integrasi.mysimkari_satker` / `integrasi.mysimkari_pegawai`. The employee lookup by NIP,
//!   the satker roster, `v_mysimkari_pegawai_summary` and `v_satker_code_map` all read these.
//! * `integrasi.siman_aset`. Bank aset, the dashboard, pemakaian, kebutuhan and the derived
//!   snapshots all read this.
//!
//! Found by reading the code before the first production install (2026-11-12). Run against the
//! code it was found in, these tests showed:
//!
//! * MySIMKARI wrote to `mysimkari_satker_satker` and `mysimkari_pegawai_pegawai`, two tables
//!   it created for itself. It fetched employees for the first five satker only ("as sample").
//!   The tables above stayed empty, so no NIP typed into any form could ever be found.
//! * SIMAN has no key the insert can conflict on, so every weekly run appended the whole asset
//!   register again. A run cut short added its half on top of the last complete copy.
//!
//! The same run is also where `SIMAN_BA_KEY`, a credential, got out: in the path of
//! `getRowCount/<BA_KEY>/<table>`, it was printed at INFO, stored in `integrasi.api_log.url`, and
//! quoted again by every reqwest error about that URL. The BA-key tests read the run's whole
//! output and `api_log` for any 8 characters of it.
//!
//! The upstream records below are made up. Their SHAPE follows `dokumentasi_mysimkari.md`: a
//! `{status, message, data}` envelope, satker `id` as a string, and the employee key `GOL_KD` in
//! upper case.
//!
//! `DATABASE_URL` is REQUIRED. A missing or unreachable database FAILS the test (see
//! `db_reconnect.rs` for why a test that skips itself is worth nothing here).

use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio_postgres::{Client, NoTls};
use wiremock::matchers::{body_string_contains, header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SATKER_TOKEN: &str = "uji-token-satker";
const PEGAWAI_TOKEN: &str = "uji-token-pegawai";
/// A credential (operator decision 2026-11-12). Long and distinctive so that any 8 characters of
/// it can be searched for in what a run printed or stored.
const BA_KEY: &str = "BAKEY-q7Rm2XvL9tWc4H";
const SIMAN: &str = "/gateway/SLDKSimanKL/2.0";

// ---------------------------------------------------------------------------------------------
// A database of its own, migrated like production's.
// ---------------------------------------------------------------------------------------------

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

struct Db {
    admin: String,
    database: String,
    url: String,
    client: Client,
}

impl Drop for Db {
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

impl Db {
    async fn new() -> Self {
        static SEQ: AtomicU32 = AtomicU32::new(0);
        let admin = admin_url();
        // Lower-case letters, digits and underscores only: it is spliced into CREATE DATABASE.
        let database = format!(
            "integrasi_sync_{}_{}",
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

        let (client, connection) = tokio_postgres::connect(&url, NoTls)
            .await
            .expect("connect to the test database");
        tokio::spawn(connection);
        Db {
            admin,
            database,
            url,
            client,
        }
    }

    async fn count(&self, sql: &str) -> i64 {
        self.client
            .query_one(sql, &[])
            .await
            .unwrap_or_else(|e| panic!("{sql}: {e:?}"))
            .get(0)
    }

    async fn text(&self, sql: &str) -> Option<String> {
        self.client
            .query_one(sql, &[])
            .await
            .unwrap_or_else(|e| panic!("{sql}: {e:?}"))
            .get(0)
    }

    /// Every table the service could have written to.
    async fn tables(&self) -> BTreeSet<String> {
        self.client
            .query(
                "SELECT schemaname || '.' || tablename FROM pg_tables \
                 WHERE schemaname IN ('integrasi', 'public')",
                &[],
            )
            .await
            .expect("list tables")
            .iter()
            .map(|r| r.get(0))
            .collect()
    }
}

/// Run the CronJob's command line and return whether it exited 0, and what it printed.
async fn scheduler(db: &Db, source: &str, envs: Vec<(&'static str, String)>) -> (bool, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_scheduler"));
    cmd.args(["--source", source])
        // Nothing from the developer's or CI's shell: no real tokens, no real API hosts.
        .env_clear()
        .env("RUST_LOG", "info")
        .env("STORAGE_TYPE", "database")
        .env("DATABASE_URL", &db.url);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    // Blocking wait off the runtime: the mock server answers on this runtime meanwhile.
    let out = tokio::task::spawn_blocking(move || cmd.output())
        .await
        .expect("join")
        .expect("the scheduler binary must start");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), log)
}

// ---------------------------------------------------------------------------------------------
// MySIMKARI
// ---------------------------------------------------------------------------------------------

/// `9001` … `9007`.
fn satker_id(n: u32) -> String {
    format!("{}", 9000 + n)
}

/// 18 digits: `9000000000000` + satker (3) + employee (2). Characters 14–16 are the satker, so
/// SQL can tell which satker an employee was fetched for.
fn nip(satker: u32, k: u32) -> String {
    format!("9000000000000{satker:03}{k:02}")
}

fn satker(n: u32) -> Value {
    json!({
        "id": satker_id(n),
        "parent_id": if n == 1 { Value::Null } else { json!(satker_id(1)) },
        "nama_satker": format!("Satker Uji {n}"),
        "tipe_satker": "3",
        "alamat_satker": format!("Jalan Uji No. {n}"),
        "kode_satker": format!("99.{n:02}"),
        "telp_satker": "000",
        "website_satker": "https://uji.invalid/",
        "city": "Kota Uji",
        "long": "106.8",
        "lat": "-6.2",
        "provinsi": "Provinsi Uji",
        "wilayah": "I",
        "kategori_satker": "Uji",
        "pulau": "Jawa"
    })
}

fn pegawai(satker: u32, k: u32) -> Value {
    json!({
        "nama": format!("Pegawai Uji {satker}-{k}"),
        "nip": nip(satker, k),
        "no_hp": "000",
        "email_dinas": format!("pegawai{satker}{k}@uji.invalid"),
        "bidang": "Pembinaan",
        "foto": null,
        "jk": "L",
        "agama": "Islam",
        "nrp": null,
        "jabatan": "Pengadministrasi Uji",
        "golpang": "Penata Muda (III/a)",
        "jenis_jabatan_terakhir": "Pelaksana",
        "jabat_tmt": "2024-01-01",
        "eselon": null,
        "GOL_KD": "3A"
    })
}

fn envelope(data: Value) -> Value {
    json!({ "status": true, "message": "Berhasil", "data": data })
}

/// Satker `n` and its roster; `None` = that satker's roster answers HTTP 500.
type Plan = Vec<(u32, Option<Vec<Value>>)>;

/// Seven satker, two employees each.
fn full_plan() -> Plan {
    (1..=7)
        .map(|n| (n, Some(vec![pegawai(n, 1), pegawai(n, 2)])))
        .collect()
}

async fn mount_mysimkari(server: &MockServer, plan: &Plan) {
    server.reset().await;
    let satkers: Vec<Value> = plan.iter().map(|(n, _)| satker(*n)).collect();
    Mock::given(method("GET"))
        .and(path("/get-satker"))
        .and(header(
            "Authorization",
            format!("Bearer {SATKER_TOKEN}").as_str(),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(json!(satkers))))
        .mount(server)
        .await;
    for (n, roster) in plan {
        let reply = match roster {
            Some(list) => ResponseTemplate::new(200).set_body_json(envelope(json!(list))),
            None => ResponseTemplate::new(500).set_body_string("Internal Server Error"),
        };
        Mock::given(method("GET"))
            .and(path(format!("/pegawai-satker/{}", satker_id(*n))))
            .and(header(
                "Authorization",
                format!("Bearer {PEGAWAI_TOKEN}").as_str(),
            ))
            .respond_with(reply)
            .mount(server)
            .await;
    }
}

fn mysimkari_env(server: &MockServer) -> Vec<(&'static str, String)> {
    vec![
        ("MYSIMKARI_BASE_URL", server.uri()),
        ("MYSIMKARI_TOKEN", SATKER_TOKEN.to_string()),
        ("MYSIMKARI_TOKEN_PEGAWAI", PEGAWAI_TOKEN.to_string()),
    ]
}

#[tokio::test(flavor = "multi_thread")]
async fn a_mysimkari_sync_fills_the_tables_the_application_reads() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_mysimkari(&server, &full_plan()).await;
    let before = db.tables().await;

    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;
    assert!(ok, "the sync failed:\n{log}");

    let created: Vec<String> = db.tables().await.difference(&before).cloned().collect();
    assert!(
        created.is_empty(),
        "the sync created tables of its own, which nothing reads: {created:?}\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_satker")
            .await,
        7,
        "every satker MySIMKARI listed:\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai")
            .await,
        14,
        "the employees of EVERY satker, not a sample:\n{log}"
    );
    // The roster reply does not say which satker it belongs to; the sync must record it.
    assert_eq!(
        db.count(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai p \
             JOIN integrasi.mysimkari_satker s ON s.api_id = p.satker_id \
             WHERE p.satker_id = '9' || substr(p.nip, 14, 3)"
        )
        .await,
        14,
        "each employee must carry the id of the satker it was fetched for"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE gol_kd = '3A'")
            .await,
        14,
        "the API spells this key GOL_KD"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE status_pegawai = 'aktif'")
            .await,
        14
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_mysimkari_sync_updates_in_place_and_retires_who_left() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_mysimkari(&server, &full_plan()).await;
    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;
    assert!(ok, "first sync failed:\n{log}");

    // Six hours later: one employee has left satker 1, one in satker 2 has a new position.
    let mut plan = full_plan();
    plan[0].1 = Some(vec![pegawai(1, 1)]);
    let mut promoted = pegawai(2, 1);
    promoted["jabatan"] = json!("Kepala Subbagian Uji");
    plan[1].1 = Some(vec![promoted, pegawai(2, 2)]);
    mount_mysimkari(&server, &plan).await;
    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;
    assert!(ok, "second sync failed:\n{log}");

    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_satker")
            .await,
        7,
        "a second run must not add the satker again:\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai")
            .await,
        14,
        "a second run must neither duplicate nor delete employees:\n{log}"
    );
    assert_eq!(
        db.text(&format!(
            "SELECT status_pegawai FROM integrasi.mysimkari_pegawai WHERE nip = '{}'",
            nip(1, 2)
        ))
        .await
        .as_deref(),
        Some("nonaktif"),
        "an employee MySIMKARI no longer lists is retired, not left active"
    );
    assert_eq!(
        db.count(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE status_pegawai = 'nonaktif'"
        )
        .await,
        1
    );
    assert_eq!(
        db.text(&format!(
            "SELECT jabatan FROM integrasi.mysimkari_pegawai WHERE nip = '{}'",
            nip(2, 1)
        ))
        .await
        .as_deref(),
        Some("Kepala Subbagian Uji"),
        "a changed record is updated in place"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_roster_that_could_not_be_read_fails_the_run_and_retires_nobody() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_mysimkari(&server, &full_plan()).await;
    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;
    assert!(ok, "first sync failed:\n{log}");

    // The roster of satker 3 cannot be read this time. Its two employees did not leave: nobody
    // knows who is in it. Retiring them would lock them out of every form that looks up a NIP.
    let mut plan = full_plan();
    plan[2].1 = None;
    mount_mysimkari(&server, &plan).await;
    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;

    assert!(
        !ok,
        "a sync that could not read a satker's roster exited 0:\n{log}"
    );
    assert!(
        log.contains(&satker_id(3)),
        "the failure must name the satker whose roster was not read:\n{log}"
    );
    assert_eq!(
        db.count(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE status_pegawai = 'nonaktif'"
        )
        .await,
        0,
        "employees were retired because their roster could not be READ:\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE status_pegawai = 'aktif'")
            .await,
        14,
        "{log}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_record_that_cannot_be_stored_fails_the_run_and_stays_out_of_the_log() {
    let db = Db::new().await;
    let server = MockServer::start().await;

    // Satker 4 lists an employee without an NIP (`nip` is NOT NULL). Everything about that
    // person is personal data; none of it may reach the pod log.
    let mut nameless = pegawai(4, 3);
    nameless["nip"] = Value::Null;
    nameless["nama"] = json!("Nama Rahasia Uji");
    nameless["email_dinas"] = json!("rahasia.uji@uji.invalid");
    nameless["no_hp"] = json!("081299990000");
    let mut plan = full_plan();
    plan[3].1 = Some(vec![pegawai(4, 1), pegawai(4, 2), nameless]);
    mount_mysimkari(&server, &plan).await;

    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;

    assert!(
        !ok,
        "a sync that could not store a record exited 0, so the Job is Complete:\n{log}"
    );
    assert!(
        log.contains(&satker_id(4)) && log.contains("mysimkari_pegawai"),
        "the failure must say which satker and which table:\n{log}"
    );
    for secret in [
        "Nama Rahasia Uji",
        "rahasia.uji@uji.invalid",
        "081299990000",
    ] {
        assert!(
            !log.contains(secret),
            "the log carries `{secret}`, a value from the record that failed:\n{log}"
        );
    }
    // The run failed, so nobody is retired; the records that could be stored are.
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_pegawai")
            .await,
        14,
        "{log}"
    );
    assert_eq!(
        db.count(
            "SELECT count(*) FROM integrasi.mysimkari_pegawai WHERE status_pegawai = 'nonaktif'"
        )
        .await,
        0
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn upstream_keys_that_are_reserved_words_or_not_names_do_not_sink_the_reply() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_mysimkari(&server, &full_plan()).await;

    // One key that is a reserved word, one that cannot be a column name at all. Unquoted, the
    // first made `ALTER TABLE … ADD COLUMN order` a syntax error; then every row of the reply
    // failed, because the INSERT named a column that did not exist.
    let satkers: Vec<Value> = (1..=7)
        .map(|n| {
            let mut s = satker(n);
            s["order"] = json!(n);
            s["kode satker; lama"] = json!("x");
            s
        })
        .collect();
    Mock::given(method("GET"))
        .and(path("/get-satker"))
        .respond_with(ResponseTemplate::new(200).set_body_json(envelope(json!(satkers))))
        .with_priority(1)
        .mount(&server)
        .await;

    let (ok, log) = scheduler(&db, "mysimkari", mysimkari_env(&server)).await;
    assert!(ok, "the sync failed:\n{log}");
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_satker")
            .await,
        7,
        "{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.mysimkari_satker WHERE \"order\" IS NOT NULL")
            .await,
        7,
        "a key named like a SQL keyword is still a column:\n{log}"
    );
    assert!(
        log.contains("kode satker; lama"),
        "the key that was not stored must be named in the log:\n{log}"
    );
}

// ---------------------------------------------------------------------------------------------
// SIMAN
// ---------------------------------------------------------------------------------------------

fn aset(i: u32) -> Value {
    json!({
        "id": i,
        "kdsatker_keu": format!("99{:04}", i % 7),
        "nama_satker": "Satker Uji",
        "kd_brg": "2010104001",
        "nup": i.to_string(),
        "nama": format!("Tanah Uji {i}"),
        "ur_sskel": "Tanah Bangunan Kantor Pemerintah",
        "kondisi": "1",
        "ur_kondisi": "Baik",
        "rph_aset": "1000000",
        "tgl_perlh": "2020-01-01"
    })
}

/// SIMAN with `total` assets in category Tanah and none in the other nine. The data endpoint
/// pages by `ID_1`..`ID_2` in blocks of 1000; every block from `fail_from` on answers HTTP 500.
async fn mount_siman(server: &MockServer, total: u32, fail_from: Option<u32>) {
    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/connect/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "uji-akses",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!(
            "{SIMAN}/getRowCount/{BA_KEY}/SIMAN2_M_ASET_TANAH"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "row_count": total }])))
        .with_priority(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2\.0/getRowCount/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "row_count": 0 }])))
        .with_priority(5)
        .mount(server)
        .await;
    for start in (1..=total).step_by(1000) {
        let end = (start + 999).min(total);
        let reply = if fail_from.is_some_and(|f| start >= f) {
            ResponseTemplate::new(500).set_body_string("Internal Server Error")
        } else {
            let records: Vec<Value> = (start..=end).map(aset).collect();
            ResponseTemplate::new(200).set_body_json(json!({ "results": records }))
        };
        Mock::given(method("POST"))
            .and(path(format!("{SIMAN}/getAsetTanah")))
            // The form body is `BA_KEY=…&ID_1=<start>&ID_2=<end>`.
            .and(body_string_contains(format!("ID_1={start}&")))
            .respond_with(reply)
            .mount(server)
            .await;
    }
}

fn siman_env(server: &MockServer) -> Vec<(&'static str, String)> {
    vec![
        ("SIMAN_BASE_URL", server.uri()),
        ("SIMAN_TOKEN_URL", format!("{}/connect/token", server.uri())),
        ("SIMAN_CLIENT_ID", "uji-client".to_string()),
        ("SIMAN_CLIENT_SECRET", "uji-rahasia".to_string()),
        ("SIMAN_BA_KEY", BA_KEY.to_string()),
    ]
}

#[tokio::test(flavor = "multi_thread")]
async fn siman_keeps_one_row_per_asset_however_often_it_runs() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_siman(&server, 3, None).await;

    for run in 1..=2 {
        let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
        assert!(ok, "run {run} failed:\n{log}");
        assert_eq!(
            db.count("SELECT count(*) FROM integrasi.siman_aset").await,
            3,
            "after run {run}: SIMAN lists 3 assets\n{log}"
        );
    }
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.siman_aset WHERE jenis_aset = 'Tanah'")
            .await,
        3
    );

    // An asset SIMAN no longer lists (disposed of) is no longer in the local copy.
    mount_siman(&server, 2, None).await;
    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(ok, "run 3 failed:\n{log}");
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.siman_aset").await,
        2,
        "after run 3: SIMAN lists 2 assets\n{log}"
    );
}

/// Fail if `text` holds any 8 consecutive characters of the BA key.
fn assert_no_ba_key(what: &str, text: &str) {
    let chars: Vec<char> = BA_KEY.chars().collect();
    for window in chars.windows(8) {
        let fragment: String = window.iter().collect();
        assert!(
            !text.contains(&fragment),
            "{what} contains `{fragment}`, a fragment of SIMAN_BA_KEY.\n--- {what} ---\n{text}"
        );
    }
}

/// Everything a run left in `integrasi.api_log` that a person or a monitoring endpoint can read.
async fn api_log_text(db: &Db) -> (i64, String) {
    let rows = db
        .client
        .query(
            "SELECT concat_ws(' | ', endpoint, kode_kl, kdsatker, full_url, error_message) \
             FROM integrasi.api_log WHERE module = 'SIMAN' ORDER BY id",
            &[],
        )
        .await
        .expect("read api_log");
    let text = rows
        .iter()
        .map(|r| r.get::<_, String>(0))
        .collect::<Vec<_>>()
        .join("\n");
    (rows.len() as i64, text)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_siman_run_keeps_the_ba_key_out_of_its_log_and_out_of_api_log() {
    // SIMAN asks for the key in the PATH of getRowCount, so the URL of that call is a credential.
    // The old code printed it at INFO ("Fetching SIMAN row count: …") and stored it in
    // `integrasi.api_log.url`, a table four unauthenticated HTTP endpoints served.
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_siman(&server, 3, None).await;

    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(ok, "the run failed:\n{log}");
    // The run really used the key: the only getRowCount the mock answers with rows is the one
    // whose path carries it, and the assets landed.
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.siman_aset").await,
        3
    );
    let (rows, stored) = api_log_text(&db).await;
    assert!(rows > 0, "the run logged no SIMAN call in api_log:\n{log}");

    assert_no_ba_key("the run's log", &log);
    assert_no_ba_key("integrasi.api_log", &stored);

    // Hidden, not lost: both still say which call was made.
    assert!(
        log.contains("Fetching SIMAN row count") && log.contains("<SIMAN_BA_KEY>"),
        "the log must show the row-count call, with the key replaced:\n{log}"
    );
    assert!(
        stored.contains("getRowCount/<SIMAN_BA_KEY>/SIMAN2_M_ASET_TANAH"),
        "api_log must still say which call was made:\n{stored}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_siman_failure_does_not_carry_the_ba_key_into_the_log() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    let token = || {
        Mock::given(method("POST"))
            .and(path("/connect/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "uji-akses",
                "token_type": "Bearer",
                "expires_in": 3600
            })))
    };

    // 1. The row count is refused, and the refusal quotes the request.
    server.reset().await;
    token().mount(&server).await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2\.0/getRowCount/"))
        .respond_with(ResponseTemplate::new(500).set_body_string(format!(
            "getRowCount/{BA_KEY}: BA_KEY {BA_KEY} tidak dikenal"
        )))
        .mount(&server)
        .await;
    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(!ok, "a run whose every row count failed exited 0:\n{log}");
    assert!(log.contains("500"), "the log must say what failed:\n{log}");
    assert_no_ba_key("the log of a refused row count", &log);

    // 2. The data call is refused, and the refusal echoes its form body (`BA_KEY=…&ID_1=…`).
    server.reset().await;
    token().mount(&server).await;
    Mock::given(method("GET"))
        .and(path(format!(
            "{SIMAN}/getRowCount/{BA_KEY}/SIMAN2_M_ASET_TANAH"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "row_count": 1 }])))
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/gateway/SLDKSimanKL/2\.0/getRowCount/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{ "row_count": 0 }])))
        .with_priority(5)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("{SIMAN}/getAsetTanah")))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string(format!("permintaan ditolak: BA_KEY={BA_KEY}&ID_1=1&ID_2=1")),
        )
        .mount(&server)
        .await;
    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(!ok, "a run whose data call failed exited 0:\n{log}");
    assert!(
        log.contains("getAsetTanah"),
        "the log must say which call failed:\n{log}"
    );
    assert_no_ba_key("the log of a refused data call", &log);

    // 3. Nothing listens where the API should be. reqwest's error quotes the whole URL, and the
    //    URL of the row count holds the key.
    server.reset().await;
    token().mount(&server).await;
    let mut env = siman_env(&server);
    for (k, v) in env.iter_mut() {
        if *k == "SIMAN_BASE_URL" {
            *v = "http://127.0.0.1:9".to_string();
        }
    }
    let (ok, log) = scheduler(&db, "siman", env).await;
    assert!(!ok, "a run that reached no API exited 0:\n{log}");
    assert!(
        log.contains("error sending request"),
        "the log must say the request could not be sent:\n{log}"
    );
    assert_no_ba_key("the log of an unreachable API", &log);

    let (_, stored) = api_log_text(&db).await;
    assert_no_ba_key("integrasi.api_log", &stored);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_siman_category_cut_short_keeps_the_last_complete_copy() {
    let db = Db::new().await;
    let server = MockServer::start().await;
    mount_siman(&server, 1500, None).await;
    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(ok, "first run failed:\n{log}");
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.siman_aset").await,
        1500,
        "{log}"
    );

    // Assets 1001-1500 cannot be fetched this time; 1-1000 can.
    mount_siman(&server, 1500, Some(1001)).await;
    let (ok, log) = scheduler(&db, "siman", siman_env(&server)).await;
    assert!(
        !ok,
        "a run that fetched 1000 of 1500 assets exited 0:\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(*) FROM integrasi.siman_aset").await,
        1500,
        "a run cut short must leave the last COMPLETE copy, not add its half to it:\n{log}"
    );
    assert_eq!(
        db.count("SELECT count(DISTINCT nup) FROM integrasi.siman_aset")
            .await,
        1500
    );
}
