//! # Cancel-safe mutation middleware
//!
//! Hyper drops a handler future the moment the client goes away. Over HTTP/2 —
//! which is what nginx and the Istio sidecar both speak to this service — that
//! is a RST_STREAM, and it arrives as soon as the browser navigates, closes the
//! tab, or the user hits Escape. The handler is then truncated at whatever
//! `.await` it happened to be sitting on.
//!
//! For a read that is harmless. For a mutation it is not, because several
//! service methods here commit MORE THAN ONCE and the DB has no way to undo the
//! earlier commits once the future is gone:
//!
//!   * `PemakaianBmnService::approver_satker_approve` commits APPROVED, then
//!     auto-activates (nomor_izin + SK) in a second unit of work. Truncated in
//!     between, the permit is APPROVED forever: no number, no SK, and nothing
//!     retries it. The degraded path the code documents ("aktivasi dapat
//!     di-retry") never even logs, because a dropped future does not error.
//!   * `KebutuhanBmnService::validator_pusat_keputusan` records the decision,
//!     then advances SUBMIT_PUSAT → ANALISIS_KELAYAKAN → APPROVED as two
//!     further transitions. Truncated, the satker sits in ANALISIS_KELAYAKAN
//!     carrying a decision that was never applied.
//!
//! Both were observed live, in one CI run, as nginx `499` (client closed
//! request) on `approver-satker-action` and `keputusan-pusat` — the e2e specs
//! clicked and immediately reloaded, which is exactly what an impatient user
//! does.
//!
//! Running the inner service on a spawned task decouples it from the caller:
//! dropping a `JoinHandle` detaches the task rather than aborting it, so the
//! work runs to completion and the database still reaches its settled state.
//! The response is discarded when nobody is left to receive it.
//!
//! Scope is derived from the method, not from a list of endpoints, so a new
//! mutating route is covered the day it is added. Safe methods are left alone
//! deliberately: cancelling a GET should stay free, and WebSocket upgrades and
//! SSE streams arrive as GET — detaching those would leak a task per abandoned
//! connection.

use axum::{
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::Instrument;

/// True for methods whose handlers may leave torn state if truncated.
fn is_mutation(method: &Method) -> bool {
    !matches!(
        *method,
        Method::GET | Method::HEAD | Method::OPTIONS | Method::TRACE
    )
}

pub async fn cancel_safe_middleware(request: Request, next: Next) -> Response {
    if !is_mutation(request.method()) {
        return next.run(request).await;
    }

    // `tokio::spawn` does not inherit the caller's span, and the request span
    // (method + uri) is what makes these logs diagnosable at all — carry it in
    // explicitly or the mutation logs land at the root.
    let span = tracing::Span::current();
    let handle = tokio::spawn(async move { next.run(request).await }.instrument(span));

    match handle.await {
        Ok(response) => response,
        Err(join_err) => {
            // Only reachable if the handler panicked; the task is never
            // aborted (nothing calls `abort`, and detaching does not cancel).
            tracing::error!(error = %join_err, "mutation handler panicked");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Terjadi kesalahan internal saat memproses permintaan",
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_methods_are_not_detached() {
        // GET carries WebSocket upgrades and SSE; detaching those would leak a
        // task per abandoned connection.
        assert!(!is_mutation(&Method::GET));
        assert!(!is_mutation(&Method::HEAD));
        assert!(!is_mutation(&Method::OPTIONS));
        assert!(!is_mutation(&Method::TRACE));
    }

    #[test]
    fn mutating_methods_are_detached() {
        assert!(is_mutation(&Method::POST));
        assert!(is_mutation(&Method::PUT));
        assert!(is_mutation(&Method::PATCH));
        assert!(is_mutation(&Method::DELETE));
    }

    // The property that matters: dropping the JoinHandle must NOT stop the
    // task. If tokio ever changed that, every multi-commit handler would go
    // back to tearing, so pin it here rather than trusting the doc comment.
    #[tokio::test]
    async fn dropping_the_join_handle_lets_the_task_finish() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let finished = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&finished);

        let handle = tokio::spawn(async move {
            tokio::task::yield_now().await;
            flag.store(true, Ordering::SeqCst);
        });
        drop(handle); // the caller went away mid-request

        for _ in 0..100 {
            if finished.load(Ordering::SeqCst) {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("detached task did not run to completion");
    }
}
