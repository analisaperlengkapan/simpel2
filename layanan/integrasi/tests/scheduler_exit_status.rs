//! The `scheduler` binary's EXIT STATUS is the only thing a Kubernetes CronJob sees.
//!
//! These run the real binary. A run that fetched nothing must not exit 0: before this, a failed
//! sync was logged (`❌ Manual sync failed`) and `main` then returned `Ok(())`, so the Job ended
//! `Complete`, `lastSuccessfulTime` advanced, and no alert could ever fire for a sync that had
//! never once worked (found in the production rehearsal, 2026-11-09).

use std::process::{Command, Output};

fn run_scheduler(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_scheduler"));
    cmd.args(args)
        // Nothing from the developer's/CI shell may leak in: no real tokens, no real API hosts.
        .env_clear()
        .env("RUST_LOG", "info")
        // EMPTY, not absent: `load_base_config` falls back to
        // `postgres://postgres:postgres@localhost:5432/perlengkapan` when the variable is unset.
        // A developer box with a local Postgres connected and these tests passed; the CI job has
        // none and the binary died on "Connection refused" before reaching the code under test
        // (first CI run of this file). Empty = no database, which is all these tests need.
        .env("DATABASE_URL", "");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("the scheduler binary must start")
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

#[test]
fn an_unknown_source_is_a_failed_run() {
    let out = run_scheduler(&["--source", "bukan-sumber"], &[]);
    assert!(
        !out.status.success(),
        "a typo in a CronJob's --source exited 0:\n{}",
        text(&out)
    );
    assert!(text(&out).contains("Invalid source"), "{}", text(&out));
}

#[test]
fn a_sync_that_could_not_reach_its_api_is_a_failed_run() {
    // 127.0.0.1:9 (discard) has nothing listening: the request is refused at once, offline.
    let out = run_scheduler(
        &["--source", "mysimkari"],
        &[
            ("MYSIMKARI_BASE_URL", "http://127.0.0.1:9"),
            ("MYSIMKARI_TOKEN", "bukan-token-sungguhan"),
        ],
    );
    let log = text(&out);
    assert!(
        log.contains("Manual sync failed"),
        "the failure must still be logged for whoever reads the pod log:\n{log}"
    );
    assert!(
        !out.status.success(),
        "a sync that fetched NOTHING exited 0, so the Job would be marked Complete:\n{log}"
    );
}

#[test]
fn a_siman_sync_whose_every_category_failed_is_a_failed_run() {
    // SIMAN fetches ten categories one after another. Each category that cannot be fetched used
    // to be logged as `Failed to fetch …` and skipped, then the run returned Ok: the rehearsal's
    // `siman` Job ended `Complete` with 10 of 10 categories failed. SIMAN is the asset source of
    // record, so an expired client secret must turn the CronJob red, not green.
    let out = run_scheduler(
        &["--source", "siman"],
        &[
            ("SIMAN_BASE_URL", "http://127.0.0.1:9"),
            ("SIMAN_TOKEN_URL", "http://127.0.0.1:9/connect/token"),
            ("SIMAN_CLIENT_ID", "bukan-client"),
            ("SIMAN_CLIENT_SECRET", "bukan-rahasia-sungguhan"),
            ("SIMAN_BA_KEY", "bukan-kunci"),
        ],
    );
    let log = text(&out);
    assert!(
        log.contains("Failed to fetch"),
        "the per-category failure must still be logged for whoever reads the pod log:\n{log}"
    );
    assert!(
        !out.status.success(),
        "a SIMAN sync in which EVERY category failed exited 0, so the Job would be Complete:\n{log}"
    );
}

#[test]
fn a_monsakti_sync_that_could_not_reach_its_api_is_a_failed_run() {
    // The third CronJob. It already propagated before the SIMAN fix; this keeps it that way.
    let out = run_scheduler(
        &["--source", "monsakti"],
        &[
            ("MONSAKTI_BASE_URL", "http://127.0.0.1:9"),
            ("MONSAKTI_TOKEN_ADM", "bukan-token-sungguhan"),
        ],
    );
    let log = text(&out);
    assert!(
        log.contains("Manual sync failed"),
        "the failure must still be logged for whoever reads the pod log:\n{log}"
    );
    assert!(
        !out.status.success(),
        "a MonSAKTI sync that fetched NOTHING exited 0:\n{log}"
    );
}

#[test]
fn a_cronjob_run_does_not_depend_on_the_daemon_flag() {
    // `SCHEDULER_ENABLED` is the switch for the long-running DAEMON. Every CronJob runs
    // `./scheduler --source <p>`, an explicit request for one run, and the daemon is off by
    // default (the replicated gRPC server must not schedule syncs on its own). If the flag gated
    // `--source` too, each CronJob would log an error and exit 0 having synced nothing.
    // `None` is what the chart ships (it never sets the variable); `false` is what an operator
    // writes to be sure.
    for flag in [None, Some("false")] {
        let mut envs = vec![
            ("MYSIMKARI_BASE_URL", "http://127.0.0.1:9"),
            ("MYSIMKARI_TOKEN", "bukan-token-sungguhan"),
        ];
        if let Some(v) = flag {
            envs.push(("SCHEDULER_ENABLED", v));
        }
        let out = run_scheduler(&["--source", "mysimkari"], &envs);
        let log = text(&out);
        assert!(
            log.contains("Manual sync failed"),
            "SCHEDULER_ENABLED={flag:?}: the sync was never attempted:\n{log}"
        );
        assert!(
            !log.contains("daemon is disabled"),
            "SCHEDULER_ENABLED={flag:?}: --source was treated as the daemon:\n{log}"
        );
        assert!(
            !out.status.success(),
            "SCHEDULER_ENABLED={flag:?}: a sync that fetched NOTHING exited 0:\n{log}"
        );
    }
}

#[test]
fn the_daemon_does_not_start_unless_asked_to() {
    // No `--source`, nothing in the environment: this is the shape of `server`'s own decision
    // too, since both read the same default. It must stand down at once. Before, the default was
    // ON, so the process went on to start a cron that never exits.
    let out = run_scheduler(&[], &[]);
    let log = text(&out);
    assert!(
        log.contains("daemon is disabled"),
        "the daemon must stand down by default:\n{log}"
    );
    assert!(
        out.status.success(),
        "standing down is not a failure:\n{log}"
    );
}
