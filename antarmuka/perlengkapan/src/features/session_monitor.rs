//! Background refresh loop so Perlengkapan can stand alone.
//!
//! Portal already runs its own refresh ticker (`portal/src/features/session_monitor.rs`).
//! When a user opens Perlengkapan in a separate tab while Portal is also open,
//! storage events keep this tab's session in sync — see `AuthService::setup_storage_listener`.
//! But if Perlengkapan is the only open tab, nobody else refreshes the JWT and the
//! user gets booted the moment `exp` passes. This monitor closes that gap.

// AuthService is referenced only from the wasm refresh loop below; gating the
// import keeps the host-target build warning-free (and immune to `cargo fix`).
#[cfg(target_arch = "wasm32")]
use crate::features::auth::AuthService;
#[cfg(target_arch = "wasm32")]
use leptos::prelude::*;

/// Spawn a long-lived task that refreshes the JWT before it expires and
/// pushes a fresh `UserSession` into the provided write signal. Cancels
/// itself when the owning scope is dropped.
#[cfg(target_arch = "wasm32")]
pub fn spawn_refresh_loop(
    set_user_session: WriteSignal<Option<crate::features::auth::UserSession>>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::future::TimeoutFuture;
        use leptos::task::spawn_local;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        // Refresh window: trigger when the token has <= 120s left. Matches
        // Portal's heuristic so both tabs agree on when to renew.
        const REFRESH_THRESHOLD_SECONDS: i64 = 120;
        // Tick interval. 30s is short enough to react before expiry without
        // hammering the refresh endpoint.
        const TICK_MS: u32 = 30_000;

        let alive = Arc::new(AtomicBool::new(true));
        let alive_cleanup = alive.clone();
        on_cleanup(move || {
            alive_cleanup.store(false, Ordering::SeqCst);
        });

        spawn_local(async move {
            loop {
                if !alive.load(Ordering::SeqCst) {
                    break;
                }

                match AuthService::load_session() {
                    None => {
                        // No session at all — Portal-side logout already
                        // cleared the token. Reflect that in the UI and
                        // stop ticking.
                        set_user_session.set(None);
                        break;
                    }
                    Some(session) => {
                        if !session.is_active() {
                            // Token already past `exp`. Drop the session
                            // and let the auth guards redirect to /login.
                            AuthService::clear_session();
                            set_user_session.set(None);
                            break;
                        }

                        if session.expires_within(REFRESH_THRESHOLD_SECONDS) {
                            match AuthService::try_refresh().await {
                                Ok(fresh) => set_user_session.set(Some(fresh)),
                                Err(_) => {
                                    // Refresh failed (refresh token expired,
                                    // network down, authenc unreachable).
                                    // Hard logout so the user sees the
                                    // login screen instead of stale UI.
                                    AuthService::broadcast_logout();
                                    AuthService::clear_session();
                                    set_user_session.set(None);
                                    break;
                                }
                            }
                        }
                    }
                }

                TimeoutFuture::new(TICK_MS).await;
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = set_user_session;
    }
}
