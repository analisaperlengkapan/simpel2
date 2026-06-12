//! Admin audit log viewer — paginated activity log with user / entity /
//! action / date-range filters. Consumes `GET /admin/audit` which lands in
//! plan commit 19.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CARET_LEFT, CARET_RIGHT, MAGNIFYING_GLASS};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, SubmitEvent};

use crate::api::admin::{AuditFilter, AuditLogEntry, fetch_audit_logs};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};

const PER_PAGE: i32 = 25;

#[component]
pub fn AdminAuditPage() -> impl IntoView {
    let (entries, set_entries) = signal::<Vec<AuditLogEntry>>(Vec::new());
    let (total, set_total) = signal(0_i64);
    let (page, set_page) = signal(1_i32);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);

    let (actor, set_actor) = signal::<String>(String::new());
    let (entity_type, set_entity_type) = signal::<String>(String::new());
    let (action, set_action) = signal::<String>(String::new());
    let (date_from, set_date_from) = signal::<String>(String::new());
    let (date_to, set_date_to) = signal::<String>(String::new());
    let (search, set_search) = signal::<String>(String::new());
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let filter = AuditFilter {
            page: page.get(),
            per_page: PER_PAGE,
            actor: Some(actor.get()).filter(|s| !s.trim().is_empty()),
            entity_type: Some(entity_type.get()).filter(|s| !s.trim().is_empty()),
            action: Some(action.get()).filter(|s| !s.trim().is_empty()),
            date_from: Some(date_from.get()).filter(|s| !s.trim().is_empty()),
            date_to: Some(date_to.get()).filter(|s| !s.trim().is_empty()),
            search: Some(search.get()).filter(|s| !s.trim().is_empty()),
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match fetch_audit_logs(&filter).await {
                Ok(resp) => {
                    set_entries.set(resp.data);
                    set_total.set(resp.total);
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let apply_filters = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let clear_filters = move |_| {
        set_actor.set(String::new());
        set_entity_type.set(String::new());
        set_action.set(String::new());
        set_date_from.set(String::new());
        set_date_to.set(String::new());
        set_search.set(String::new());
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", "/perlengkapan/dashboard"),
        PageBreadcrumb::new("Admin", "/perlengkapan/admin/workflow"),
        PageBreadcrumb::leaf("Audit Log"),
    ];

    let total_pages = move || {
        let t = total.get();
        if t == 0 {
            1
        } else {
            ((t + PER_PAGE as i64 - 1) / PER_PAGE as i64) as i32
        }
    };

    view! {
        <PageLayout
            title="Audit Log"
            description="Jejak audit seluruh aktivitas sistem: siapa, apa, kapan, dan entitas yang terdampak."
            icon="fas fa-history"
            breadcrumbs=breadcrumbs
        >
            <SectionCard title="Filter" icon="fas fa-filter" dense=true>
                <form class="flex flex-col gap-3" on:submit=apply_filters>
                    <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-3">
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Pengguna"</span>
                            <input
                                type="text"
                                placeholder="nama atau NIP"
                                prop:value=move || actor.get()
                                on:input=move |e| set_actor.set(event_target_value(&e))
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                        </label>
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Jenis Entitas"</span>
                            <select
                                on:change=move |e| {
                                    let v = e
                                        .target()
                                        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                                    if let Some(sel) = v {
                                        set_entity_type.set(sel.value());
                                    }
                                }
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                            >
                                <option value="" selected=move || entity_type.get().is_empty()>
                                    "Semua"
                                </option>
                                <option
                                    value="kebutuhan_bmn"
                                    selected=move || entity_type.get() == "kebutuhan_bmn"
                                >
                                    "Kebutuhan BMN"
                                </option>
                                <option
                                    value="pakaian_dinas"
                                    selected=move || entity_type.get() == "pakaian_dinas"
                                >
                                    "Pakaian Dinas"
                                </option>
                                <option
                                    value="pemakaian_bmn"
                                    selected=move || entity_type.get() == "pemakaian_bmn"
                                >
                                    "Pemakaian BMN"
                                </option>
                                <option
                                    value="penghapusan_bmn"
                                    selected=move || entity_type.get() == "penghapusan_bmn"
                                >
                                    "Penghapusan BMN"
                                </option>
                                <option
                                    value="bank_aset"
                                    selected=move || entity_type.get() == "bank_aset"
                                >
                                    "Bank Aset"
                                </option>
                                <option
                                    value="workflow"
                                    selected=move || entity_type.get() == "workflow"
                                >
                                    "Workflow"
                                </option>
                                <option
                                    value="master_data"
                                    selected=move || entity_type.get() == "master_data"
                                >
                                    "Master Data"
                                </option>
                                <option value="user" selected=move || entity_type.get() == "user">
                                    "Pengguna"
                                </option>
                            </select>
                        </label>
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Aksi"</span>
                            <select
                                on:change=move |e| {
                                    let v = e
                                        .target()
                                        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                                    if let Some(sel) = v {
                                        set_action.set(sel.value());
                                    }
                                }
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                            >
                                <option value="" selected=move || action.get().is_empty()>
                                    "Semua"
                                </option>
                                <option value="create" selected=move || action.get() == "create">
                                    "Create"
                                </option>
                                <option value="update" selected=move || action.get() == "update">
                                    "Update"
                                </option>
                                <option value="delete" selected=move || action.get() == "delete">
                                    "Delete"
                                </option>
                                <option value="approve" selected=move || action.get() == "approve">
                                    "Approve"
                                </option>
                                <option value="reject" selected=move || action.get() == "reject">
                                    "Reject"
                                </option>
                                <option value="login" selected=move || action.get() == "login">
                                    "Login"
                                </option>
                                <option value="logout" selected=move || action.get() == "logout">
                                    "Logout"
                                </option>
                            </select>
                        </label>
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Dari tanggal"</span>
                            <input
                                type="date"
                                prop:value=move || date_from.get()
                                on:input=move |e| {
                                    let v = e
                                        .target()
                                        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                                    if let Some(inp) = v {
                                        set_date_from.set(inp.value());
                                    }
                                }
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                            />
                        </label>
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Sampai tanggal"</span>
                            <input
                                type="date"
                                prop:value=move || date_to.get()
                                on:input=move |e| {
                                    let v = e
                                        .target()
                                        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                                    if let Some(inp) = v {
                                        set_date_to.set(inp.value());
                                    }
                                }
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
                            />
                        </label>
                        <label class="flex flex-col gap-1 text-xs">
                            <span class="font-semibold text-slate-300">"Pencarian bebas"</span>
                            <input
                                type="search"
                                placeholder="id entitas, ringkasan..."
                                prop:value=move || search.get()
                                on:input=move |e| set_search.set(event_target_value(&e))
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                        </label>
                    </div>
                    <div class="flex flex-wrap justify-end gap-2 pt-1">
                        <button
                            type="button"
                            class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-4 py-2 text-sm font-medium text-slate-200 transition hover:bg-white/[0.08]"
                            on:click=clear_filters
                        >
                            "Reset"
                        </button>
                        <button
                            type="submit"
                            class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105"
                        >
                            <span class="mr-1.5">
                                <AppIcon icon=MAGNIFYING_GLASS />
                            </span>
                            "Terapkan"
                        </button>
                    </div>
                </form>
            </SectionCard>

            <SectionCard title="Aktivitas" icon="fas fa-scroll">
                <div class="mb-3 text-xs text-slate-400">
                    "Menampilkan " <strong class="text-white">{move || total.get()}</strong>
                    " entri."
                </div>
                {move || {
                    if loading.get() {
                        view! { <LoadingState message="Memuat audit log..." /> }.into_any()
                    } else if let Some(e) = error.get() {
                        let retry: Box<dyn Fn()> = Box::new(move || {
                            set_reload_tick.update(|t| *t += 1);
                        });
                        view! { <ErrorState error=e on_retry=retry /> }.into_any()
                    } else if entries.with(Vec::is_empty) {
                        view! {
                            <EmptyState
                                title="Tidak ada aktivitas"
                                description="Belum ada entri audit untuk filter saat ini."
                                icon="fas fa-inbox"
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            <AuditTable rows=entries />
                            <Pagination
                                page=page
                                set_page=set_page
                                total_pages=Signal::derive(total_pages)
                                on_change=Box::new(move || set_reload_tick.update(|t| *t += 1))
                            />
                        }
                            .into_any()
                    }
                }}
            </SectionCard>
        </PageLayout>
    }
}

#[component]
fn AuditTable(rows: ReadSignal<Vec<AuditLogEntry>>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
            <table class="min-w-full border-collapse">
                <thead>
                    <tr class="bg-white/[0.02]">
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "Waktu"
                        </th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "Pengguna"
                        </th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "Aksi"
                        </th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "Entitas"
                        </th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "Ringkasan"
                        </th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">
                            "IP"
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows
                            .get()
                            .into_iter()
                            .map(|entry| {
                                let actor_label = entry
                                    .actor_name
                                    .clone()
                                    .unwrap_or_else(|| "—".to_string());
                                let role = entry.actor_role.clone().unwrap_or_default();
                                let action_class = match entry.action.as_str() {
                                    "create" => {
                                        "border-success-500/30 bg-success-500/10 text-success-400"
                                    }
                                    "update" => "border-info-500/30 bg-info-500/10 text-info-300",
                                    "delete" => {
                                        "border-danger-500/30 bg-danger-500/10 text-danger-400"
                                    }
                                    "approve" => {
                                        "border-success-500/30 bg-success-500/10 text-success-400"
                                    }
                                    "reject" => {
                                        "border-warning-500/30 bg-warning-500/10 text-warning-400"
                                    }
                                    _ => "border-white/[0.08] bg-white/[0.04] text-slate-300",
                                };
                                let entity_id = entry.entity_id.clone().unwrap_or_default();
                                let summary = entry.summary.clone().unwrap_or_default();
                                let ip = entry
                                    .ip_address
                                    .clone()
                                    .unwrap_or_else(|| "—".to_string());
                                view! {
                                    <tr class="border-b border-white/[0.04] align-top text-xs">
                                        <td class="px-3 py-2 whitespace-nowrap text-slate-400">
                                            {entry.occurred_at.clone()}
                                        </td>
                                        <td class="px-3 py-2">
                                            <div class="font-semibold text-white">{actor_label}</div>
                                            {(!role.is_empty())
                                                .then(|| {
                                                    view! {
                                                        <div class="text-[0.65rem] text-slate-500">{role}</div>
                                                    }
                                                })}
                                        </td>
                                        <td class="px-3 py-2">
                                            <span class=format!(
                                                "inline-flex items-center gap-1.5 rounded-lg border px-2 py-0.5 text-[0.7rem] font-semibold {}",
                                                action_class,
                                            )>{entry.action.clone()}</span>
                                        </td>
                                        <td class="px-3 py-2">
                                            <div class="font-medium text-slate-200">
                                                {entry.entity_type.clone()}
                                            </div>
                                            {(!entity_id.is_empty())
                                                .then(|| {
                                                    view! {
                                                        <div class="text-[0.65rem] text-slate-500">{entity_id}</div>
                                                    }
                                                })}
                                        </td>
                                        <td class="px-3 py-2 text-slate-300">{summary}</td>
                                        <td class="px-3 py-2 text-[0.7rem] text-slate-500">{ip}</td>
                                    </tr>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn Pagination(
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
    total_pages: Signal<i32>,
    on_change: Box<dyn Fn()>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_change = Rc::new(on_change);
    let prev_handler = {
        let on_change = Rc::clone(&on_change);
        move |_: leptos::ev::MouseEvent| {
            let cur = page.get();
            if cur > 1 {
                set_page.set(cur - 1);
                on_change();
            }
        }
    };
    let _next_handler = {
        let on_change = Rc::clone(&on_change);
        move |_: leptos::ev::MouseEvent| {
            let cur = page.get();
            if cur < total_pages.get() {
                set_page.set(cur + 1);
                on_change();
            }
        }
    };
    view! {
        <div class="mt-3 flex items-center justify-between text-xs text-slate-400">
            <div>
                "Halaman " <strong class="text-white">{move || page.get()}</strong> " dari "
                <strong class="text-white">{move || total_pages.get()}</strong>
            </div>
            <div class="flex gap-2">
                <button
                    type="button"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get() <= 1
                    on:click=prev_handler
                >
                    <span class="mr-1">
                        <AppIcon icon=CARET_LEFT />
                    </span>
                    "Sebelumnya"
                </button>
                <button
                    type="button"
                    class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get()
                >
                    = total_pages.get()
                    on:click=next_handler
                    >
                    "Berikutnya"
                    <span class="ml-1">
                        <AppIcon icon=CARET_RIGHT />
                    </span>
                </button>
            </div>
        </div>
    }
}
