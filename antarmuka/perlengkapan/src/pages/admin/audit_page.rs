//! Admin audit log viewer — the paginated cross-module trail
//! (`perlengkapan.audit_log`, via `GET /audit`) with user / module / entity /
//! action / date-range filters.
//!
//! The module, entity and action dropdowns are filled from `GET /audit/facets`,
//! i.e. from what the trail actually contains. They used to be hand-written and
//! described a system that was never audited (`bank_aset`, `login`, ...) while
//! omitting what was (`tiket`, `pengajuan_satker`), and the page itself read two
//! workflow tables nothing writes, so it was empty everywhere.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use lib_ui::hooks::LatestOnly;
use lib_ui::utils::formatters::format_iso_local;
use phosphor_leptos::{CARET_LEFT, CARET_RIGHT, MAGNIFYING_GLASS};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement, SubmitEvent};

use crate::api::admin::{
    AuditFacets, AuditFilter, AuditTrailEntry, fetch_audit_facets, fetch_audit_logs,
};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};

const PER_PAGE: i32 = 25;

#[component]
pub fn AdminAuditPage() -> impl IntoView {
    let (entries, set_entries) = signal::<Vec<AuditTrailEntry>>(Vec::new());
    let (total, set_total) = signal(0_i64);
    let (page, set_page) = signal(1_i32);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);

    let (actor, set_actor) = signal::<String>(String::new());
    let (module, set_module) = signal::<String>(String::new());
    let (entity, set_entity) = signal::<String>(String::new());
    let (action, set_action) = signal::<String>(String::new());
    let (date_from, set_date_from) = signal::<String>(String::new());
    let (date_to, set_date_to) = signal::<String>(String::new());
    let (search, set_search) = signal::<String>(String::new());
    let (reload_tick, set_reload_tick) = signal(0_u32);
    // Jawaban yang tiba paling akhir tak boleh menang atas yang ditanyakan paling
    // akhir. Lihat `lib_ui::hooks::use_latest`.
    let latest = LatestOnly::new();

    // What the trail actually contains, for the dropdowns. One answer per visit,
    // so there is no later question for an earlier one to overwrite.
    let (facets, set_facets) = signal(AuditFacets::default());
    spawn_local(async move {
        match fetch_audit_facets().await {
            Ok(f) => set_facets.set(f),
            Err(e) => leptos::logging::warn!("audit facets unavailable: {e:?}"),
        }
    });

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let filter = AuditFilter {
            page: page.get(),
            per_page: PER_PAGE,
            actor: Some(actor.get()).filter(|s| !s.trim().is_empty()),
            module: Some(module.get()).filter(|s| !s.trim().is_empty()),
            entity: Some(entity.get()).filter(|s| !s.trim().is_empty()),
            action: Some(action.get()).filter(|s| !s.trim().is_empty()),
            date_from: Some(date_from.get()).filter(|s| !s.trim().is_empty()),
            date_to: Some(date_to.get()).filter(|s| !s.trim().is_empty()),
            search: Some(search.get()).filter(|s| !s.trim().is_empty()),
        };
        set_loading.set(true);
        set_error.set(None);
        let ticket = latest.ticket();
        spawn_local(async move {
            let result = fetch_audit_logs(&filter).await;
            if !latest.is_current(ticket) {
                return;
            }
            match result {
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
        set_module.set(String::new());
        set_entity.set(String::new());
        set_action.set(String::new());
        set_date_from.set(String::new());
        set_date_to.set(String::new());
        set_search.set(String::new());
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", crate::routes::path::DASHBOARD),
        PageBreadcrumb::new("Admin", crate::routes::path::ADMIN_WORKFLOW),
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
            description="Jejak audit lintas modul: siapa, apa, kapan, dan rekaman yang terdampak."
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
                                placeholder="nama pengguna atau ID pengguna"
                                prop:value=move || actor.get()
                                on:input=move |e| set_actor.set(event_target_value(&e))
                                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white placeholder:text-slate-500"
                            />
                        </label>
                        <FacetSelect
                            label="Modul"
                            value=module
                            set_value=set_module
                            options=Signal::derive(move || facets.with(|f| f.modules.clone()))
                        />
                        <FacetSelect
                            label="Jenis Entitas"
                            value=entity
                            set_value=set_entity
                            options=Signal::derive(move || facets.with(|f| f.entities.clone()))
                        />
                        <FacetSelect
                            label="Aksi"
                            value=action
                            set_value=set_action
                            options=Signal::derive(move || facets.with(|f| f.actions.clone()))
                        />
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
                                placeholder="ringkasan, nama aksi, pengguna..."
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
                    "Menampilkan "
                    <strong class="text-white" data-testid="audit-total">
                        {move || total.get()}
                    </strong>
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

/// A filter dropdown whose options are the values the trail really holds
/// (`GET /audit/facets`), so choosing one can never match nothing.
#[component]
fn FacetSelect(
    label: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    options: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <label class="flex flex-col gap-1 text-xs">
            <span class="font-semibold text-slate-300">{label}</span>
            // A <select> inside its <label> is named after the label's whole text,
            // option texts included ("Modul Semua bantuan ..."), which is what a
            // screen reader announces and what `getByLabel("Modul")` fails to match.
            // Name it explicitly.
            <select
                aria-label=label
                on:change=move |e| {
                    let v = e.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
                    if let Some(sel) = v {
                        set_value.set(sel.value());
                    }
                }
                class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white"
            >
                <option value="" selected=move || value.get().is_empty()>
                    "Semua"
                </option>
                {move || {
                    options
                        .get()
                        .into_iter()
                        .map(|opt| {
                            // The macro moves a child before it reads the attributes, so
                            // each use gets its own copy.
                            let current = opt.clone();
                            let text = opt.clone();
                            view! {
                                <option value=opt selected=move || value.get() == current>
                                    {text}
                                </option>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </select>
        </label>
    }
}

/// Badge colour per action verb (the verbs `lib_perlengkapan::audit::AuditAction`
/// can write). Anything else — `custom`, `export`, ... — stays neutral.
fn action_class(verb: &str) -> &'static str {
    match verb {
        "create" | "approve" | "submit" => {
            "border-success-500/30 bg-success-500/10 text-success-400"
        }
        "update" | "import" => "border-info-500/30 bg-info-500/10 text-info-300",
        "delete" => "border-danger-500/30 bg-danger-500/10 text-danger-400",
        "reject" | "cancel" => "border-warning-500/30 bg-warning-500/10 text-warning-400",
        _ => "border-white/[0.08] bg-white/[0.04] text-slate-300",
    }
}

const TH: &str = "border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400";

#[component]
fn AuditTable(rows: ReadSignal<Vec<AuditTrailEntry>>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
            <table class="min-w-full border-collapse">
                <thead>
                    <tr class="bg-white/[0.02]">
                        <th class=TH>"Waktu"</th>
                        <th class=TH>"Pengguna"</th>
                        <th class=TH>"Aksi"</th>
                        <th class=TH>"Entitas"</th>
                        <th class=TH>"Hasil"</th>
                        <th class=TH>"Ringkasan"</th>
                        <th class=TH>"IP"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        rows.get()
                            .into_iter()
                            .map(|entry| {
                                // The name the writer recorded, else the user id: never blank.
                                let actor_label = entry
                                    .actor_name
                                    .clone()
                                    .filter(|n| !n.trim().is_empty())
                                    .or_else(|| entry.actor_id.clone())
                                    .unwrap_or_else(|| "—".to_string());
                                let verb_class = action_class(&entry.action);
                                let action_name = entry.action_name.clone().unwrap_or_default();
                                let resource_id = entry.resource_id.clone().unwrap_or_default();
                                let resource_id_attr = resource_id.clone();
                                let message = entry.message.clone().unwrap_or_default();
                                let ip = entry
                                    .actor_ip
                                    .clone()
                                    .filter(|i| !i.trim().is_empty())
                                    .unwrap_or_else(|| "—".to_string());
                                let (result_label, result_class) = if entry.success {
                                    (
                                        "Berhasil",
                                        "border-success-500/30 bg-success-500/10 text-success-400",
                                    )
                                } else {
                                    (
                                        "Gagal",
                                        "border-danger-500/30 bg-danger-500/10 text-danger-400",
                                    )
                                };
                                view! {
                                    <tr
                                        class="border-b border-white/[0.04] align-top text-xs"
                                        data-testid="audit-row"
                                        data-module=entry.module.clone()
                                        data-entity=entry.resource_type.clone()
                                        data-action=entry.action.clone()
                                        data-resource-id=resource_id_attr
                                    >
                                        <td
                                            class="px-3 py-2 whitespace-nowrap text-slate-400"
                                            title=entry.occurred_at.clone()
                                        >
                                            {format_iso_local(&entry.occurred_at)}
                                        </td>
                                        <td class="px-3 py-2 font-semibold text-white break-all">
                                            {actor_label}
                                        </td>
                                        <td class="px-3 py-2">
                                            <span class=format!(
                                                "inline-flex items-center gap-1.5 rounded-lg border px-2 py-0.5 text-[0.7rem] font-semibold {verb_class}",
                                            )>{entry.action.clone()}</span>
                                            {(!action_name.is_empty())
                                                .then(|| {
                                                    view! {
                                                        <div class="mt-1 text-[0.65rem] text-slate-500">
                                                            {action_name}
                                                        </div>
                                                    }
                                                })}
                                        </td>
                                        <td class="px-3 py-2">
                                            <div class="font-medium text-slate-200">
                                                {entry.resource_type.clone()}
                                            </div>
                                            <div class="text-[0.65rem] text-slate-500">
                                                {entry.module.clone()}
                                            </div>
                                            {(!resource_id.is_empty())
                                                .then(|| {
                                                    view! {
                                                        <div class="text-[0.65rem] text-slate-500 break-all">
                                                            {resource_id}
                                                        </div>
                                                    }
                                                })}
                                        </td>
                                        <td class="px-3 py-2">
                                            <span class=format!(
                                                "inline-flex items-center rounded-lg border px-2 py-0.5 text-[0.7rem] font-semibold {result_class}",
                                            )>{result_label}</span>
                                        </td>
                                        <td class="px-3 py-2 text-slate-300">{message}</td>
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
    let next_handler = {
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
                    prop:disabled=move || { page.get() >= total_pages.get() }
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
