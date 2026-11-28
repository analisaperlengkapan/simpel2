//! App shell chrome — header, sidebar mount, footer.
//!
//! Extracted from `lib.rs` so the root component only wires routing
//! and the chrome owns its own markup. All inline styles were replaced
//! with Tailwind classes that reference the design tokens declared in
//! `tailwind.config.js`.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_node_ref::AnyNodeRef;
use leptos_router::components::A;
use lib_ui::components::floating::Popover;
use lib_ui::components::icon::AppIcon;
use lib_ui::utils::formatters::format_iso_local;
use lib_ui::utils::notification_url::resolve_action_url;
use phosphor_leptos::{BELL, LIST};
use wasm_bindgen::JsCast;

use crate::APP_VERSION;
use crate::api::integrasi::{IntegrasiCircuitStatus, fetch_circuit_status};
use crate::api::notifikasi::{
    NotifikasiItem, fetch_unread_count, list_notifikasi, mark_all_read, mark_read,
};
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
                            src=crate::assets::LOGO
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
/// 30s; clicking the bell opens a panel with the latest notifications (the
/// full inbox is one link away at `/notifikasi`). It used to jump straight to
/// the inbox page, so the bell offered no list at all. Failures are
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

    let is_open = RwSignal::new(false);
    let trigger_ref = AnyNodeRef::new();

    view! {
        <button
            node_ref=trigger_ref
            type="button"
            on:click=move |_| is_open.update(|open| *open = !*open)
            class="relative focus-ring rounded-md p-1.5 text-slate-300 hover:text-white"
            title="Notifikasi"
            aria-label=move || {
                match count.get() {
                    0 => "Notifikasi".to_string(),
                    n => format!("Notifikasi, {n} belum dibaca"),
                }
            }
            aria-haspopup="dialog"
            aria-expanded=move || is_open.get().to_string()
        >
            <span class="text-xl">
                <AppIcon icon=BELL />
            </span>
            <Show when=move || { count.get() > 0_i64 }>
                <span
                    aria-hidden="true"
                    class="absolute -top-0.5 -right-0.5 min-w-[1.25rem] h-5 px-1 rounded-full bg-rose-500 text-white text-[0.65rem] font-bold flex items-center justify-center"
                >
                    {move || {
                        let n = count.get();
                        if n > 99 { "99+".to_string() } else { n.to_string() }
                    }}
                </span>
            </Show>
        </button>

        <Popover
            open=is_open.read_only()
            set_open=is_open.write_only()
            trigger_ref=trigger_ref
            offset=10.0
            class="w-[360px] max-w-[calc(100vw-1rem)] rounded-[18px] border border-white/10 bg-surface-panel overflow-hidden shadow-panel"
                .to_string()
        >
            <NotifikasiPanel set_open=is_open.write_only() count=count set_count=set_count />
        </Popover>
    }
}

/// How many notifications the bell's panel lists; the rest are in the inbox.
const PANEL_SIZE: i64 = 8;

#[derive(Clone, Copy, PartialEq)]
enum PanelState {
    Loading,
    Ready,
    Failed,
}

/// The list behind the bell. The [`Popover`] renders its children only while it is open, so this
/// component is mounted afresh on every opening and that is when it fetches: what the panel lists
/// is what is there now, not what was there when the page loaded.
#[component]
fn NotifikasiPanel(
    set_open: WriteSignal<bool>,
    count: ReadSignal<i64>,
    set_count: WriteSignal<i64>,
) -> impl IntoView {
    let items = RwSignal::new(Vec::<NotifikasiItem>::new());
    let state = RwSignal::new(PanelState::Loading);

    let load = move || {
        state.set(PanelState::Loading);
        spawn_local(async move {
            match list_notifikasi(PANEL_SIZE, 0, false).await {
                Ok(rows) => {
                    items.set(rows);
                    state.set(PanelState::Ready);
                }
                Err(_) => state.set(PanelState::Failed),
            }
        });
    };
    Effect::new(move |_| load());

    // Opening a row reads it and closes the panel; the link it carries (if any) does the rest.
    let open_one = Callback::new(move |(id, was_unread): (String, bool)| {
        set_open.set(false);
        if was_unread {
            set_count.update(|c| *c = (*c - 1).max(0));
            spawn_local(async move {
                let _ = mark_read(&id).await;
            });
        }
    });

    let on_mark_all = move |_| {
        spawn_local(async move {
            if mark_all_read().await.is_ok() {
                items.update(|rows| rows.iter_mut().for_each(|r| r.read = true));
                set_count.set(0);
            }
        });
    };

    view! {
        <div class="flex items-center justify-between px-4 py-3 border-b border-white/[0.06]">
            <h3 class="text-[0.95rem] font-bold text-slate-100">"Notifikasi"</h3>
            <Show when=move || { count.get() > 0_i64 }>
                <button
                    type="button"
                    on:click=on_mark_all
                    class="focus-ring rounded text-[0.75rem] font-semibold text-amber-300 hover:underline"
                >
                    "Tandai semua dibaca"
                </button>
            </Show>
        </div>

        <div class="max-h-[26rem] overflow-y-auto">
            {move || match (state.get(), items.with(|rows| rows.is_empty())) {
                (PanelState::Loading, true) => {
                    view! {
                        <p
                            role="status"
                            class="px-4 py-8 text-center text-[0.82rem] text-slate-400"
                        >
                            "Memuat notifikasi…"
                        </p>
                    }
                        .into_any()
                }
                (PanelState::Failed, true) => {
                    view! {
                        <div class="px-4 py-8 text-center">
                            <p class="text-[0.82rem] text-slate-400">
                                "Notifikasi tidak dapat dimuat."
                            </p>
                            <button
                                type="button"
                                on:click=move |_| load()
                                class="focus-ring mt-3 rounded text-[0.78rem] font-semibold text-amber-300 hover:underline"
                            >
                                "Coba lagi"
                            </button>
                        </div>
                    }
                        .into_any()
                }
                (_, true) => {
                    view! {
                        <p class="px-4 py-8 text-center text-[0.82rem] text-slate-400">
                            "Belum ada notifikasi."
                        </p>
                    }
                        .into_any()
                }
                (_, false) => {
                    view! {
                        <For
                            each=move || items.get()
                            key=|n| (n.id.clone(), n.read)
                            children=move |n| view! { <NotifikasiRow n=n on_open=open_one /> }
                        />
                    }
                        .into_any()
                }
            }}
        </div>

        <a
            href=routes::path::NOTIFIKASI
            on:click=move |_| set_open.set(false)
            class="focus-ring block border-t border-white/[0.06] py-3 text-center text-[0.8rem] font-semibold text-sky-400 hover:underline"
        >
            "Lihat semua notifikasi"
        </a>
    }
}

/// One row of the panel: a link when the notification points somewhere (see
/// [`resolve_action_url`]), otherwise a button that only marks it read. A row that does nothing
/// when clicked is what the bell's panel replaces.
#[component]
fn NotifikasiRow(
    n: NotifikasiItem,
    /// `(id, was_unread)` of the row that was opened.
    #[prop(into)]
    on_open: Callback<(String, bool)>,
) -> impl IntoView {
    let href = n.action_url.as_deref().and_then(resolve_action_url);
    let id = n.id.clone();
    let is_unread = !n.read;
    let row_class = format!(
        "block w-full text-left px-4 py-3 border-b border-white/[0.06] border-l-[3px] hover:bg-white/[0.04] focus-visible:bg-white/[0.06] focus-visible:outline-none {}",
        if is_unread {
            "border-l-amber-400"
        } else {
            "border-l-transparent"
        },
    );
    let body = view! {
        <span class="block text-[0.85rem] font-bold text-slate-200">
            {n.title.clone()}
            {is_unread.then(|| view! { <span class="sr-only">" (belum dibaca)"</span> })}
        </span>
        <span class="mt-1 block text-[0.8rem] leading-snug text-slate-300">
            {n.message.clone()}
        </span>
        <span class="mt-1 block text-[0.72rem] text-slate-500">
            {format_iso_local(&n.created_at)}
        </span>
    };

    match href {
        Some(href) => view! {
            <a href=href class=row_class on:click=move |_| on_open.run((id.clone(), is_unread))>
                {body}
            </a>
        }
        .into_any(),
        None => view! {
            <button
                type="button"
                class=row_class
                on:click=move |_| on_open.run((id.clone(), is_unread))
            >
                {body}
            </button>
        }
        .into_any(),
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
                <span class="text-[0.65rem] text-slate-600">
                    {format!("© {} Biro Perlengkapan", crate::tahun::tahun_berjalan())}
                </span>
            </div>
        </footer>
    }
}
