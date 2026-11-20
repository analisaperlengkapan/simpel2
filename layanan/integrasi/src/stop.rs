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
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct Stop(Arc<Inner>);

#[derive(Default)]
struct Inner {
    flag: AtomicBool,
    notify: Notify,
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
    async fn a_trigger_that_came_first_is_still_seen() {
        let stop = Stop::new();
        stop.trigger();
        tokio::time::timeout(Duration::from_millis(200), stop.cancelled())
            .await
            .expect("an already-triggered stop must complete at once");
    }
}
