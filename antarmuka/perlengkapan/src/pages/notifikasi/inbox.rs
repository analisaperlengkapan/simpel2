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

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api::notifikasi::{NotifikasiItem, list_notifikasi, mark_all_read, mark_read};

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
        <div class="max-w-4xl mx-auto p-6">
            <div class="flex items-center justify-between mb-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800">"Notifikasi"</h1>
                    <p class="text-sm text-gray-500">
                        "Notifikasi sistem (workflow, tiket bantuan, SLA)."
                    </p>
                </div>
                <div class="flex items-center gap-3">
                    <label class="text-sm text-gray-600 flex items-center gap-2">
                        <input
                            type="checkbox"
                            prop:checked=move || unread_only.get()
                            on:change=move |ev| set_unread_only.set(event_target_checked(&ev))
                        />
                        "Hanya yang belum dibaca"
                    </label>
                    <button
                        type="button"
                        class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded"
                        on:click=on_mark_all
                    >
                        "Tandai semua dibaca"
                    </button>
                </div>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="mb-3 p-3 bg-red-50 text-red-700 rounded border border-red-200 text-sm">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || loading.get()>
                <div class="text-sm text-gray-400">"Memuat notifikasi..."</div>
            </Show>

            <ul class="space-y-2">
                <For
                    each=move || items.get()
                    key=|n| n.id.clone()
                    children=move |n| {
                        let id = n.id.clone();
                        let is_unread = !n.read;
                        view! {
                            <li
                                class:bg-blue-50=is_unread
                                class:border-blue-200=is_unread
                                class="p-3 bg-white border rounded-lg flex items-start justify-between"
                            >
                                <div class="flex-1 mr-3">
                                    <div class="flex items-center gap-2">
                                        <span class="font-medium text-sm">{n.title.clone()}</span>
                                        {category_chip(n.category.clone())}
                                        {priority_chip(n.priority.clone())}
                                    </div>
                                    <p class="text-sm text-gray-700 mt-1">{n.message.clone()}</p>
                                    <div class="text-xs text-gray-400 mt-2">
                                        {n.created_at.clone()}
                                        {n
                                            .action_url
                                            .clone()
                                            .map(|u| {
                                                view! {
                                                    " · "
                                                    <a href=u class="text-blue-600 hover:underline">
                                                        "Buka"
                                                    </a>
                                                }
                                            })}
                                    </div>
                                </div>
                                <Show when=move || is_unread>
                                    <button
                                        type="button"
                                        class="text-xs px-2 py-1 bg-blue-100 hover:bg-blue-200 rounded"
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
                <p class="text-sm text-gray-400 text-center py-12">"Belum ada notifikasi."</p>
            </Show>
        </div>
    }
}

fn category_chip(category: String) -> impl IntoView {
    let (bg, label) = match category.as_str() {
        "warning" => ("bg-amber-100 text-amber-700", "Peringatan"),
        "error" => ("bg-red-100 text-red-700", "Kesalahan"),
        "success" => ("bg-green-100 text-green-700", "Sukses"),
        "system" => ("bg-gray-100 text-gray-700", "Sistem"),
        _ => ("bg-blue-100 text-blue-700", "Info"),
    };
    view! { <span class=format!("text-[10px] px-2 py-0.5 rounded {}", bg)>{label}</span> }
}

fn priority_chip(priority: String) -> impl IntoView {
    let visible = matches!(priority.as_str(), "high" | "urgent");
    let (bg, label) = match priority.as_str() {
        "urgent" => ("bg-rose-100 text-rose-700", "URGENT"),
        "high" => ("bg-orange-100 text-orange-700", "HIGH"),
        _ => ("", ""),
    };
    view! {
        <Show when=move || visible>
            <span class=format!("text-[10px] px-2 py-0.5 rounded {}", bg)>{label}</span>
        </Show>
    }
}
