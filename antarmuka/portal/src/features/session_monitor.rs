//! Session synchronization and token refresh monitor.

use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Setup cross-tab storage listener to sync logout/login state.
pub fn setup_cross_tab_session_sync(
    set_user_session: WriteSignal<Option<UserSession>>,
    set_show_timeout_warning: WriteSignal<bool>,
    set_timeout_countdown: WriteSignal<i64>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        Effect::new(move |_| {
            AuthService::setup_storage_listener(move |session| {
                let is_session_none = session.is_none();
                set_user_session.set(session);

                if is_session_none {
                    set_show_timeout_warning.set(false);
                    set_timeout_countdown.set(0);
                }
            });
        });
    }
}

/// Setup automatic token refresh and session timeout monitoring.
pub fn setup_session_refresh_monitor(
    user_session: ReadSignal<Option<UserSession>>,
    timeout_countdown: ReadSignal<i64>,
    set_user_session: WriteSignal<Option<UserSession>>,
    set_show_timeout_warning: WriteSignal<bool>,
    set_timeout_countdown: WriteSignal<i64>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::future::TimeoutFuture;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        Effect::new(move |_| {
            if let Some(_session) = user_session.get() {
                let is_active = Arc::new(AtomicBool::new(true));
                let is_active_clone = is_active.clone();

                on_cleanup(move || {
                    is_active_clone.store(false, Ordering::SeqCst);
                });

                spawn_local(async move {
                    'monitor: loop {
                        if !is_active.load(Ordering::SeqCst) {
                            break;
                        }

                        if let Some(mut current_session) = AuthService::load_session() {
                            if !AuthService::is_session_valid(&current_session) {
                                AuthService::broadcast_logout();
                                AuthService::logout();
                                set_user_session.set(None);
                                set_show_timeout_warning.set(false);
                                set_timeout_countdown.set(0);
                                break;
                            }

                            if AuthService::should_refresh_token(&current_session)
                                && let Some(refresh_token) = &current_session.refresh_token
                            {
                                match AuthService::refresh_token(refresh_token).await {
                                    Ok(token_response) => {
                                        AuthService::update_session_token(&token_response);
                                        set_user_session.set(AuthService::load_session());
                                        if let Some(refreshed_session) = AuthService::load_session() {
                                            current_session = refreshed_session;
                                        }
                                    }
                                    Err(_) => {
                                        AuthService::broadcast_logout();
                                        AuthService::logout();
                                        set_user_session.set(None);
                                        set_show_timeout_warning.set(false);
                                        set_timeout_countdown.set(0);
                                        break;
                                    }
                                }
                            }

                            if let Some(expires_at) = current_session.expires_at {
                                let now = chrono::Utc::now().timestamp();
                                let time_until_expiry = expires_at - now;

                                if time_until_expiry > 0 && time_until_expiry <= 120 {
                                    set_show_timeout_warning.set(true);
                                    set_timeout_countdown.set(time_until_expiry);
                                } else {
                                    set_show_timeout_warning.set(false);
                                    set_timeout_countdown.set(0);
                                }
                            }
                        } else {
                            break;
                        }

                        if !is_active.load(Ordering::SeqCst) {
                            break;
                        }

                        for _ in 0..30 {
                            if !is_active.load(Ordering::SeqCst) {
                                break;
                            }
                            TimeoutFuture::new(1_000).await;

                            let current = timeout_countdown.get_untracked();
                            if current > 0 {
                                set_timeout_countdown.set(current - 1);
                                if current - 1 <= 0 {
                                    AuthService::broadcast_logout();
                                    AuthService::logout();
                                    set_user_session.set(None);
                                    set_show_timeout_warning.set(false);
                                    break 'monitor;
                                }
                            }
                        }
                    }
                });
            }
        });
    }
}
