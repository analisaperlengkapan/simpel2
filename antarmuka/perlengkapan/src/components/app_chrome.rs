//! App shell chrome — header, sidebar mount, footer.
//!
//! Extracted from `lib.rs` so the root component only wires routing
//! and the chrome owns its own markup. All inline styles were replaced
//! with Tailwind classes that reference the design tokens declared in
//! `tailwind.config.js`.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{BELL, LIST};
use wasm_bindgen::JsCast;

use crate::APP_VERSION;
use crate::api::integrasi::{IntegrasiCircuitStatus, fetch_circuit_status};
use crate::api::notifikasi::fetch_unread_count;
use crate::components::profile_menu::ProfileMenu;
use crate::routes;

#[component]
pub fn AppHeader(#[prop(into)] on_toggle_sidebar: Callback<()>) -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 border-b border-white/[0.06] bg-surface-panel">
            <div class="flex h-14 items-center justify-between px-5">
                <div class="flex items-center gap-3">
                    <button
                        type="button"
                        on:click=move |_| on_toggle_sidebar.run(())
                        class="focus-ring rounded-md p-1 text-slate-400 hover:text-slate-100 lg:hidden"
                        aria-label="Buka menu"
                    >
                        <span class="text-lg">
                            <AppIcon icon=LIST />
                        </span>
                    </button>
                    <A
                        href=routes::path::DASHBOARD
                        attr:class="flex items-center gap-3 no-underline"
                    >
                        <img
                            src="/perlengkapan/assets/kejaksaan-logo.png"
                            alt="Kejaksaan RI"
                            class="h-8 w-8 object-contain"
                        />
                        <span class="text-lg font-extrabold tracking-tight text-white">
                            "SIMPEL"
                        </span>
                    </A>
                </div>
                <div class="flex items-center gap-3">
                    <NotifikasiBadge />
                    <ProfileMenu />
                </div>
            </div>
        </header>
    }
}

/// Toolbar bell + unread count. Polls `GET /notifikasi/unread-count` every
/// 30s; clicking the bell navigates to `/notifikasi`. Failures are
/// swallowed silently so a transient backend hiccup doesn't blow up the
/// shell — the count just stays at its last known value.
#[component]
fn NotifikasiBadge() -> impl IntoView {
    let (count, set_count) = signal::<i64>(0);

    let load = move || {
        spawn_local(async move {
            if let Ok(n) = fetch_unread_count().await {
                set_count.set(n);
            }
        });
    };

    // Initial fetch + 30s polling.
    Effect::new(move |_| {
        load();
    });

    // Set up the interval once. Leptos has no built-in interval helper,
    // so call into `web_sys::Window::set_interval_with_callback_and_timeout`.
    Effect::new(move |handle: Option<i32>| {
        if let Some(id) = handle {
            return id;
        }
        let window = match web_sys::window() {
            Some(w) => w,
            None => return 0,
        };
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            load();
        }) as Box<dyn Fn()>);
        let func: &::js_sys::Function = closure.as_ref().unchecked_ref();
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(func, 30_000)
            .unwrap_or(0);
        // The interval owns the closure for the page lifetime, so leak it.
        closure.forget();
        id
    });

    view! {
        <A
            href=routes::path::NOTIFIKASI
            attr:class="relative focus-ring rounded-md p-1.5 text-slate-300 hover:text-white no-underline"
            attr:aria_label="Notifikasi"
        >
            <span class="text-xl">
                <AppIcon icon=BELL />
            </span>
            <Show when=move || { count.get() > 0_i64 }>
                <span class="absolute -top-0.5 -right-0.5 min-w-[1.25rem] h-5 px-1 rounded-full bg-rose-500 text-white text-[0.65rem] font-bold flex items-center justify-center">
                    {move || {
                        let n = count.get();
                        if n > 99 { "99+".to_string() } else { n.to_string() }
                    }}
                </span>
            </Show>
        </A>
    }
}

/// Global banner warning when an integrasi data source (SIMAN / MySIMKARI /
/// MonSAKTI) has tripped its circuit breaker (Fase 2.2). Polls
/// `GET /integrasi/circuit-status` every 60s; renders nothing while all
/// sources are healthy, so it stays invisible in the common case. Failures
/// are swallowed — a flaky status probe must not itself raise an alarm.
#[component]
pub fn IntegrasiHealthBanner() -> impl IntoView {
    let (degraded, set_degraded) = signal::<Vec<IntegrasiCircuitStatus>>(Vec::new());

    let load = move || {
        spawn_local(async move {
            if let Ok(list) = fetch_circuit_status().await {
                let down: Vec<IntegrasiCircuitStatus> =
                    list.into_iter().filter(|s| !s.healthy).collect();
                set_degraded.set(down);
            }
        });
    };

    Effect::new(move |_| {
        load();
    });

    // Poll every 60s — breaker state changes on the order of tens of seconds
    // (open_duration 30s), so a minute is responsive enough without noise.
    Effect::new(move |handle: Option<i32>| {
        if let Some(id) = handle {
            return id;
        }
        let window = match web_sys::window() {
            Some(w) => w,
            None => return 0,
        };
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            load();
        }) as Box<dyn Fn()>);
        let func: &::js_sys::Function = closure.as_ref().unchecked_ref();
        let id = window
            .set_interval_with_callback_and_timeout_and_arguments_0(func, 60_000)
            .unwrap_or(0);
        closure.forget();
        id
    });

    view! {
        <Show when=move || {
            !degraded.get().is_empty()
        }>
            {move || {
                let list = degraded.get();
                let names = list
                    .iter()
                    .map(|s| format!("{} ({})", s.source_display(), s.label))
                    .collect::<Vec<_>>()
                    .join(", ");
                view! {
                    <div
                        role="status"
                        class="border-b border-amber-500/30 bg-amber-500/10 px-5 py-2 text-sm text-amber-200"
                    >
                        <span class="font-semibold">"Integrasi terganggu: "</span>
                        {names}
                        ". Data dari sumber tsb mungkin berasal dari cache / belum terbaru."
                    </div>
                }
            }}
        </Show>
    }
}

#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-white/[0.04] bg-navy-950/90 px-5 py-2.5 lg:ml-[250px]">
            <div class="flex items-center justify-between">
                <span class="text-[0.7rem] text-slate-500">
                    "SIMPEL v" {APP_VERSION} " · Kejaksaan Agung RI"
                </span>
                <span class="text-[0.65rem] text-slate-600">"© 2026 Biro Perlengkapan"</span>
            </div>
        </footer>
    }
}
