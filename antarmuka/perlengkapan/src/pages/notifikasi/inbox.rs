//! `/notifikasi` — in-app notification inbox.
//!
//! Lists notifications written by the backend `NotifikasiService::send`
//! into `notifikasi.in_app_notifications`. Two filters: all / unread.
//! Each row exposes a "Tandai dibaca" button (calls
//! `PATCH /notifikasi/{id}/read`) and the page header has a
//! "Tandai semua dibaca" action (`POST /notifikasi/read-all`).
//!
//! Naming: keep the user-facing label to plain "Notifikasi" — "Center" /
//! "Pusat" was an English-ism that didn't fit the Indonesian context.
//!
//! Styling: this page was written against Tailwind's light palette (`bg-white`,
//! `text-gray-*`) while the app is navy. The consequence was not merely
//! inconsistent: the title `<span>` carried no colour class, so it inherited
//! the app's light body colour and rendered white-on-white — every title on
//! the page was invisible, as were both action buttons. The palette below is
//! the one the dashboard and laporan pages use.

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::notifikasi::{NotifikasiItem, list_notifikasi, mark_all_read, mark_read};
// Shared with the helpdesk tickets and the bank-aset dashboard, which
// printed the same stored format the same wrong way.
use lib_ui::utils::formatters::format_iso_local;

const PAGE_SIZE: i64 = 50;

#[component]
pub fn NotifikasiInboxPage() -> impl IntoView {
    let (items, set_items) = signal::<Vec<NotifikasiItem>>(Vec::new());
    let (unread_only, set_unread_only) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let reload = move || {
        let unread = unread_only.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match list_notifikasi(PAGE_SIZE, 0, unread).await {
                Ok(rows) => set_items.set(rows),
                Err(e) => set_error.set(Some(format!("Gagal memuat notifikasi: {}", e))),
            }
            set_loading.set(false);
        });
    };

    // Initial load + reload whenever `unread_only` toggles.
    Effect::new(move |_| {
        let _ = unread_only.get();
        reload();
    });

    // Helper kept in scope; each row's button closes over its own
    // `id.clone()` and re-invokes this fn via spawn_local.
    fn dispatch_mark_one(
        id: String,
        set_items: WriteSignal<Vec<NotifikasiItem>>,
        set_error: WriteSignal<Option<String>>,
    ) {
        let id_for_set = id.clone();
        spawn_local(async move {
            if let Err(e) = mark_read(&id).await {
                set_error.set(Some(format!("Gagal menandai dibaca: {}", e)));
                return;
            }
            set_items.update(|rows| {
                if let Some(row) = rows.iter_mut().find(|r| r.id == id_for_set) {
                    row.read = true;
                }
            });
        });
    }

    let on_mark_all = move |_| {
        spawn_local(async move {
            match mark_all_read().await {
                Ok(_) => {
                    set_items.update(|rows| {
                        for r in rows.iter_mut() {
                            r.read = true;
                        }
                    });
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menandai semua dibaca: {}", e)));
                }
            }
        });
    };

    view! {
        <div style="max-width: 900px; margin: 0 auto;">
            <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 20px;">
                <div>
                    <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">
                        "Notifikasi"
                    </h1>
                    <p style="font-size: 0.82rem; color: #7b8ba1; margin: 4px 0 0;">
                        "Notifikasi sistem (workflow, tiket bantuan, SLA)."
                    </p>
                </div>
                <div style="display: flex; align-items: center; gap: 12px;">
                    <label style="display: flex; align-items: center; gap: 8px; font-size: 0.8rem; color: #94a3b8; cursor: pointer;">
                        <input
                            type="checkbox"
                            style="accent-color: #fbbf24;"
                            prop:checked=move || unread_only.get()
                            on:change=move |ev| set_unread_only.set(event_target_checked(&ev))
                        />
                        "Hanya yang belum dibaca"
                    </label>
                    <button
                        type="button"
                        style="padding: 7px 14px; font-size: 0.78rem; font-weight: 600; color: #e2e8f0; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.1); border-radius: 8px; cursor: pointer;"
                        on:click=on_mark_all
                    >
                        "Tandai semua dibaca"
                    </button>
                </div>
            </div>

            <Show when=move || error.get().is_some()>
                <div style="margin-bottom: 12px; padding: 12px 14px; background: rgba(248,113,113,0.1); border: 1px solid rgba(248,113,113,0.3); border-radius: 10px; color: #fca5a5; font-size: 0.8rem;">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div style="font-size: 0.82rem; color: #7b8ba1; padding: 8px 0;">
                    "Memuat notifikasi..."
                </div>
            </Show>

            <ul style="list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px;">
                <For
                    each=move || items.get()
                    // Keyed on `read` as well as `id`. With the id alone the
                    // row was never re-rendered after "Tandai dibaca", because
                    // `is_unread` below is a plain bool captured at render
                    // time — the signal updated and the card did not.
                    key=|n| (n.id.clone(), n.read)
                    children=move |n| {
                        let id = n.id.clone();
                        let is_unread = !n.read;
                        // Unread is marked by an amber left edge rather than a
                        // filled background: on a dark surface a tinted card
                        // reads as "disabled", the opposite of the intent.
                        let card = format!(
                            "padding: 14px 16px 14px 18px; background: rgba(255,255,255,0.04); \
                             border: 1px solid {}; border-left: 3px solid {}; \
                             border-radius: 12px; display: flex; align-items: flex-start; \
                             justify-content: space-between; gap: 12px;",
                            if is_unread {
                                "rgba(251,191,36,0.28)"
                            } else {
                                "rgba(255,255,255,0.07)"
                            },
                            if is_unread { "#fbbf24" } else { "transparent" },
                        );
                        let waktu = format_iso_local(&n.created_at);
                        view! {
                            <li style=card>
                                <div style="flex: 1; min-width: 0;">
                                    <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                        <span style="font-size: 0.85rem; font-weight: 700; color: #e2e8f0;">
                                            {n.title.clone()}
                                        </span>
                                        {category_chip(n.category.clone())}
                                        {priority_chip(n.priority.clone())}
                                    </div>
                                    <p style="font-size: 0.82rem; color: #cbd5e1; margin: 6px 0 0; line-height: 1.5;">
                                        {n.message.clone()}
                                    </p>
                                    <div style="font-size: 0.72rem; color: #7b8ba1; margin-top: 8px;">
                                        {waktu}
                                        {n
                                            .action_url
                                            .clone()
                                            .map(|u| {
                                                view! {
                                                    " · "
                                                    <a
                                                        href=u
                                                        style="color: #60a5fa; text-decoration: none; font-weight: 600;"
                                                    >
                                                        "Buka"
                                                    </a>
                                                }
                                            })}
                                    </div>
                                </div>
                                <Show when=move || is_unread>
                                    <button
                                        type="button"
                                        style="flex-shrink: 0; padding: 5px 10px; font-size: 0.72rem; font-weight: 600; color: #fbbf24; background: rgba(251,191,36,0.12); border: 1px solid rgba(251,191,36,0.25); border-radius: 7px; cursor: pointer;"
                                        on:click={
                                            let id = id.clone();
                                            move |_| dispatch_mark_one(id.clone(), set_items, set_error)
                                        }
                                    >
                                        "Tandai dibaca"
                                    </button>
                                </Show>
                            </li>
                        }
                    }
                />
            </ul>

            <Show when=move || !loading.get() && items.get().is_empty()>
                <p style="font-size: 0.85rem; color: #7b8ba1; text-align: center; padding: 48px 0;">
                    "Belum ada notifikasi."
                </p>
            </Show>
        </div>
    }
}

/// Tinted on the dark surface rather than Tailwind's `-100/-700` pairs, which
/// were built for a white page and washed out here.
fn chip_style(color: &str) -> String {
    format!(
        "font-size: 0.65rem; font-weight: 700; letter-spacing: 0.02em; padding: 2px 8px; border-radius: 6px; background: {c}1f; color: {c}; border: 1px solid {c}33; white-space: nowrap;",
        c = color
    )
}

fn category_chip(category: String) -> impl IntoView {
    let (color, label) = match category.as_str() {
        "warning" => ("#fbbf24", "Peringatan"),
        "error" => ("#f87171", "Kesalahan"),
        "success" => ("#34d399", "Sukses"),
        "system" => ("#94a3b8", "Sistem"),
        _ => ("#60a5fa", "Info"),
    };
    view! { <span style=chip_style(color)>{label}</span> }
}

/// Only `high` and `urgent` get a chip — a badge on every row is a badge on
/// none. Wording is Indonesian like everything else on the page; "URGENT" /
/// "HIGH" were the raw enum values showing through.
fn priority_chip(priority: String) -> impl IntoView {
    let (color, label) = match priority.as_str() {
        "urgent" => ("#fb7185", "SEGERA"),
        "high" => ("#fb923c", "PENTING"),
        _ => ("", ""),
    };
    let visible = !label.is_empty();
    let style = chip_style(color);
    view! {
        <Show when=move || visible>
            <span style=style.clone()>{label}</span>
        </Show>
    }
}
