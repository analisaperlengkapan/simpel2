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
        // Nothing from the developer's/CI shell may leak in: no DATABASE_URL (no DB is touched),
        // no real tokens, no real API hosts.
        .env_clear()
        .env("RUST_LOG", "info");
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
