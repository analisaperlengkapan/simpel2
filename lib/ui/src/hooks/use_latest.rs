//! Newest-request-wins guard for loaders that re-run on every filter change.
//!
//! # Why this exists
//!
//! A list page that re-fetches inside `Effect::new(.. spawn_local(..))` lets two
//! requests overlap, and HTTP answers do not come back in the order the questions
//! were asked. Without a guard the answer that lands **last** wins, not the one that
//! was asked **last**.
//!
//! Measured on staging, 2026-11-06, Bank Aset list: searching `E2E-` over 624 533
//! rows takes about ten seconds, and the page sent that search **twice** (the input
//! event and the "Terapkan" click). The user then picked "Rusak Ringan"; the second
//! search answered after the "Rusak Ringan" answer and replaced it. The dropdown said
//! *Rusak Ringan* while the table listed five *Baik* rows — 4 of 12 runs, always the
//! same order of events.
//!
//! # Use
//!
//! ```ignore
//! let latest = LatestOnly::new();
//! Effect::new(move |_| {
//!     let filter = read_filters();
//!     let ticket = latest.ticket();          // when the request STARTS
//!     spawn_local(async move {
//!         let result = fetch(&filter).await;
//!         if !latest.is_current(ticket) {    // a newer request has started since
//!             return;                        // its answer is the one that counts
//!         }
//!         publish(result);
//!     });
//! });
//! ```
//!
//! The superseded request returns **before** touching any signal — including the
//! `loading` flag — so the spinner stays up until the newest request answers.
//!
//! A page that was unmounted while its request was in flight has a disposed guard;
//! `is_current` then answers `false`, so a late answer is dropped instead of being
//! written to signals nobody renders.

use leptos::prelude::*;

/// Hands out one ticket per request; only the newest ticket is "current".
///
/// `Copy`, so it moves into `Effect` and `spawn_local` closures like a signal.
#[derive(Clone, Copy)]
pub struct LatestOnly(StoredValue<u64>);

impl LatestOnly {
    /// Create the guard. Call once per loader, in the component body.
    pub fn new() -> Self {
        Self(StoredValue::new(0))
    }

    /// Call when a request STARTS and carry the result into the async block.
    /// Issuing a ticket supersedes every earlier one.
    pub fn ticket(&self) -> u64 {
        // A disposed guard hands out 0, which `is_current` never accepts.
        self.0
            .try_update_value(|n| {
                *n += 1;
                *n
            })
            .unwrap_or(0)
    }

    /// `true` while no newer request has started since `ticket` was issued.
    pub fn is_current(&self, ticket: u64) -> bool {
        self.0.try_get_value() == Some(ticket)
    }
}

impl Default for LatestOnly {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reactive values need an owner to live in; tests have no component.
    fn with_owner<T>(f: impl FnOnce() -> T) -> T {
        let owner = Owner::new();
        owner.set();
        f()
    }

    #[test]
    fn a_lone_request_is_current() {
        with_owner(|| {
            let latest = LatestOnly::new();
            let t = latest.ticket();
            assert!(latest.is_current(t));
        });
    }

    #[test]
    fn a_newer_request_supersedes_the_older_one() {
        with_owner(|| {
            let latest = LatestOnly::new();
            let first = latest.ticket();
            let second = latest.ticket();
            assert!(!latest.is_current(first), "the older request is stale");
            assert!(latest.is_current(second), "the newest request wins");
        });
    }

    /// The staging incident in miniature: asked A, asked B, B answers, then A answers.
    /// Only B may publish — arrival order must not matter, ask order must.
    #[test]
    fn the_last_asked_wins_even_when_it_answers_first() {
        with_owner(|| {
            let latest = LatestOnly::new();
            let mut published = Vec::new();

            let a = latest.ticket();
            let b = latest.ticket();

            // B (newest) answers first, A (stale) answers last.
            if latest.is_current(b) {
                published.push("B");
            }
            if latest.is_current(a) {
                published.push("A");
            }

            assert_eq!(published, ["B"]);
        });
    }

    #[test]
    fn tickets_keep_counting_after_a_request_finished() {
        with_owner(|| {
            let latest = LatestOnly::new();
            let first = latest.ticket();
            assert!(latest.is_current(first));
            // Finishing a request does not retire the guard; the next one is new.
            let second = latest.ticket();
            assert!(!latest.is_current(first));
            assert!(latest.is_current(second));
        });
    }

    #[test]
    fn copies_share_one_counter() {
        with_owner(|| {
            let latest = LatestOnly::new();
            let moved_into_closure = latest; // `Copy`: what an Effect closure captures
            let t = latest.ticket();
            assert!(moved_into_closure.is_current(t));
            let t2 = moved_into_closure.ticket();
            assert!(!latest.is_current(t));
            assert!(latest.is_current(t2));
        });
    }

    #[test]
    fn a_disposed_guard_is_never_current() {
        let latest = {
            let owner = Owner::new();
            owner.set();
            let latest = LatestOnly::new();
            let _ = latest.ticket();
            owner.cleanup(); // the page was unmounted
            latest
        };
        // Late answer after unmount: dropped, no panic.
        assert!(!latest.is_current(1));
        assert_eq!(latest.ticket(), 0);
    }
}
