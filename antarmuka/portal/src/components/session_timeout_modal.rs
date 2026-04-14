//! Session timeout warning modal component.

use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn SessionTimeoutModal(
    show: ReadSignal<bool>,
    countdown: ReadSignal<i64>,
    user_session: ReadSignal<Option<UserSession>>,
    set_user_session: WriteSignal<Option<UserSession>>,
    set_show: WriteSignal<bool>,
    set_countdown: WriteSignal<i64>,
) -> impl IntoView {
    move || {
        if show.get() {
            let secs = countdown.get();
            let minutes = secs / 60;
            let seconds = secs % 60;

            Some(view! {
                <div
                    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
                    role="dialog"
                    aria-modal="true"
                    aria-label="Peringatan sesi akan berakhir"
                >
                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700">
                        <div class="flex items-center mb-4">
                            <div class="flex-shrink-0 w-12 h-12 bg-amber-100 dark:bg-amber-900/30 rounded-full flex items-center justify-center">
                                <svg class="h-6 w-6 text-amber-600 dark:text-amber-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            </div>
                            <div class="ml-4">
                                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                                    "Sesi Akan Berakhir"
                                </h3>
                            </div>
                        </div>

                        <p class="text-gray-600 dark:text-gray-400 mb-6">
                            "Sesi Anda akan berakhir dalam "
                            <span class="font-bold text-red-600 dark:text-red-400 tabular-nums">
                                {format!("{:02}:{:02}", minutes, seconds)}
                            </span>
                            ". Silakan simpan pekerjaan Anda."
                        </p>

                        <div class="flex justify-end space-x-3">
                            <button
                                on:click=move |_| set_show.set(false)
                                class="px-4 py-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 rounded-xl hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
                            >
                                "Tutup"
                            </button>
                            <button
                                on:click=move |_| {
                                    if let Some(session) = user_session.get()
                                        && let Some(refresh_token) = &session.refresh_token
                                    {
                                        let refresh_token = refresh_token.clone();
                                        spawn_local(async move {
                                            if let Ok(token_response) = AuthService::refresh_token(&refresh_token).await {
                                                AuthService::update_session_token(&token_response);
                                                set_user_session.set(AuthService::load_session());
                                                set_show.set(false);
                                                set_countdown.set(0);
                                            }
                                        });
                                    }
                                }
                                class="px-4 py-2.5 text-sm font-medium text-white bg-navy-700 hover:bg-navy-800 dark:bg-gold-500 dark:hover:bg-gold-600 dark:text-navy-900 rounded-xl shadow-sm transition-colors"
                            >
                                "Perpanjang Sesi"
                            </button>
                        </div>
                    </div>
                </div>
            })
        } else {
            None
        }
    }
}
