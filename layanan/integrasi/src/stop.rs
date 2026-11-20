//! A request, from outside, that the running sync stop.
//!
//! Kubernetes ends a Job pod that has run past `activeDeadlineSeconds` (and one that is evicted or
//! whose node is drained) with SIGTERM, and with SIGKILL thirty seconds later. Until this existed
//! the process had no SIGTERM handler, so it died on the spot, wherever it was: a SIMAN category
//! half stored. The rows of that half-run stayed next to the last complete copy and every reader
//! saw the register twice, for up to a week (found 2026-11-20 on staging: 311,478 extra rows).
//!
//! A `Stop` is a flag a run polls and awaits. The scheduler binary triggers it on SIGTERM and
//! SIGINT; a run that sees it stops fetching, discards what it wrote and fails with a reason the
//! administrator can read, all inside the grace period.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct Stop(Arc<Inner>);

#[derive(Default)]
struct Inner {
    flag: AtomicBool,
    notify: Notify,
    active: AtomicUsize,
    idle: Notify,
}

/// Held for as long as a run is in progress; see `Stop::guard`.
pub struct RunGuard(Stop);

impl Drop for RunGuard {
    fn drop(&mut self) {
        if self.0.0.active.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.0.0.idle.notify_waiters();
        }
    }
}

impl Stop {
    pub fn new() -> Self {
        Self::default()
    }

    /// A stop that nothing ever triggers: for callers that have no process to be told about.
    pub fn never() -> Self {
        Self::default()
    }

    pub fn trigger(&self) {
        self.0.flag.store(true, Ordering::SeqCst);
        self.0.notify.notify_waiters();
    }

    pub fn is_set(&self) -> bool {
        self.0.flag.load(Ordering::SeqCst)
    }

    /// Marks a run as in progress until the guard is dropped. The long-running daemon holds one while a
    /// scheduled sync runs, so that on shutdown it can wait for the run to finish its clean-up.
    pub fn guard(&self) -> RunGuard {
        self.0.active.fetch_add(1, Ordering::SeqCst);
        RunGuard(self.clone())
    }

    /// Waits until no run is in progress, or `limit` has passed. Returns whether it is idle.
    pub async fn wait_idle(&self, limit: Duration) -> bool {
        let wait = async {
            loop {
                let notified = self.0.idle.notified();
                tokio::pin!(notified);
                notified.as_mut().enable();
                if self.0.active.load(Ordering::SeqCst) == 0 {
                    return;
                }
                notified.await;
            }
        };
        tokio::time::timeout(limit, wait).await.is_ok()
    }

    /// Completes once the stop has been triggered (immediately if it already was).
    pub async fn cancelled(&self) {
        loop {
            let notified = self.0.notify.notified();
            tokio::pin!(notified);
            // Register before checking the flag: a `trigger` between the check and the await would
            // otherwise be missed, and the run would sleep through the very signal it waits for.
            notified.as_mut().enable();
            if self.is_set() {
                return;
            }
            notified.await;
        }
    }
}

/// Triggers `stop` on SIGTERM or SIGINT, once. Spawned on the current runtime.
#[cfg(unix)]
pub fn trigger_on_signals(stop: Stop) {
    use tokio::signal::unix::{SignalKind, signal};
    tokio::spawn(async move {
        let (Ok(mut term), Ok(mut int)) = (
            signal(SignalKind::terminate()),
            signal(SignalKind::interrupt()),
        ) else {
            tracing::error!(
                "could not install the SIGTERM/SIGINT handlers: the run cannot stop cleanly"
            );
            return;
        };
        tokio::select! {
            _ = term.recv() => tracing::warn!("🛑 SIGTERM: stopping the sync, discarding what it has written so far"),
            _ = int.recv() => tracing::warn!("🛑 SIGINT: stopping the sync, discarding what it has written so far"),
        }
        stop.trigger();
    });
}

#[cfg(not(unix))]
pub fn trigger_on_signals(_stop: Stop) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn a_stop_nobody_triggers_never_completes() {
        let stop = Stop::never();
        assert!(!stop.is_set());
        let waited = tokio::time::timeout(Duration::from_millis(50), stop.cancelled()).await;
        assert!(waited.is_err(), "cancelled() returned without a trigger");
    }

    #[tokio::test]
    async fn a_trigger_wakes_a_waiter_that_was_already_waiting() {
        let stop = Stop::new();
        let waiter = {
            let stop = stop.clone();
            tokio::spawn(async move { stop.cancelled().await })
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        stop.trigger();
        tokio::time::timeout(Duration::from_secs(2), waiter)
            .await
            .expect("the waiter slept through the trigger")
            .expect("join");
        assert!(stop.is_set());
    }

    #[tokio::test]
    async fn wait_idle_returns_when_the_last_guard_is_dropped() {
        let stop = Stop::new();
        let guard = stop.guard();
        let waiter = {
            let stop = stop.clone();
            tokio::spawn(async move { stop.wait_idle(Duration::from_secs(5)).await })
        };
        tokio::time::sleep(Duration::from_millis(30)).await;
        drop(guard);
        assert!(
            waiter.await.expect("join"),
            "idle was not reported after the guard was dropped"
        );
    }

    #[tokio::test]
    async fn wait_idle_gives_up_after_its_limit_while_a_run_is_still_going() {
        let stop = Stop::new();
        let _guard = stop.guard();
        assert!(!stop.wait_idle(Duration::from_millis(60)).await);
    }

    #[tokio::test]
    async fn nothing_in_progress_is_idle_at_once() {
        assert!(Stop::new().wait_idle(Duration::from_millis(10)).await);
    }

    #[tokio::test]
    async fn a_trigger_that_came_first_is_still_seen() {
        let stop = Stop::new();
        stop.trigger();
        tokio::time::timeout(Duration::from_millis(200), stop.cancelled())
            .await
            .expect("an already-triggered stop must complete at once");
    }
}
