//! Kebutuhan BMN List Component
//!
//! Displays paginated list of BMN needs analysis requests with filtering and batch operations.

use crate::api::{
    AppError, KebutuhanBmnQuery, KebutuhanBmnStatus, KebutuhanBmnSummary, PaginatedResponse,
    fetch_kebutuhan_bmn_list,
};
use crate::components::batch_operations_toolbar::{
    BatchOperationResult, BatchOperationsToolbar, BatchResultSummary,
};
use crate::components::layout::{EmptyState, ErrorState, LoadingState, PageLayout, SectionCard};
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{EYE, PENCIL_SIMPLE, PLUS, X};

/// leptos-fetch query keyed by `(query, page, per_page, refresh)`.
/// `KebutuhanBmnQuery` impls Hash + Eq + Clone so leptos-fetch can
/// dedupe on it directly. The trailing `refresh` integer lets
/// post-CRUD code force a refetch via
/// `refresh_trigger.update(|v| *v += 1)` without invalidating the
/// other cached pages.
async fn query_kebutuhan_bmn_page(
    key: (KebutuhanBmnQuery, i32, i32, i32),
) -> Result<PaginatedResponse<KebutuhanBmnSummary>, AppError> {
    let (query, page, per_page, _refresh) = key;
    fetch_kebutuhan_bmn_list(query, page, per_page).await
}
use leptos_meta::Title;
use lib_ui::components::DarkPagination;
use uuid::Uuid;

#[component]
pub fn KebutuhanBmnList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (per_page, _set_per_page) = signal(20);
    let (tahun_filter, set_tahun_filter) = signal::<Option<i32>>(None);
    let (status_filter, set_status_filter) = signal::<Option<i32>>(None);
    let (search_query, set_search_query) = signal(String::new());

    // Batch operations state
    let (selected_ids, set_selected_ids) = signal::<Vec<Uuid>>(Vec::new());
    let (batch_result, set_batch_result) = signal::<Option<BatchOperationResult>>(None);
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    let query = Memo::new(move |_| KebutuhanBmnQuery {
        tahun: tahun_filter.get(),
        status_kode: status_filter.get(),
        satker_id: None,
        search: {
            let s = search_query.get();
            if s.is_empty() { None } else { Some(s) }
        },
    });

    let client: QueryClient = expect_context();
    let data_resource = client.local_resource(query_kebutuhan_bmn_page, move || {
        (
            query.get(),
            page.get(),
            per_page.get(),
            refresh_trigger.get(),
        )
    });

    // Page reset is handled atomically inside each filter `on:change`
    // handler below (search input, year select, status select). Doing
    // it in an `Effect` instead would cause leptos-fetch to evaluate
    // its key tuple twice in quick succession — first with
    // `(new_query, OLD_page)`, then with `(new_query, 1)` after the
    // effect fires — allocating two cache slots and potentially
    // issuing a duplicate network request on every filter change.

    let handle_operation_complete = Callback::new(move |result: BatchOperationResult| {
        set_batch_result.set(Some(result));
        set_selected_ids.set(Vec::new());
        set_refresh_trigger.update(|v| *v += 1);
    });

    let toggle_selection = move |id: Uuid| {
        set_selected_ids.update(|ids| {
            if ids.contains(&id) {
                ids.retain(|&x| x != id);
            } else {
                ids.push(id);
            }
        });
    };

    let select_all = move |items: Vec<Uuid>| {
        set_selected_ids.set(items);
    };

    let current_year = 2026;
    let years: Vec<i32> = (2020..=current_year + 1).rev().collect();

    view! {
        <Title text="Kebutuhan BMN — SIMPEL Perlengkapan" />
        <PageLayout
            title="Analisis Kebutuhan BMN"
            icon="fas fa-clipboard-list"
            description="Kelola pengajuan kebutuhan barang milik negara"
        >
            // Batch result summary
            <BatchResultSummary
                result=batch_result
                on_close=Callback::new(move |_| set_batch_result.set(None))
            />

            // Action bar
            <div class="mb-4 flex items-center justify-between">
                <div></div>
                <a
                    href=routes::path::KEBUTUHAN_BUAT
                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                >
                    <span class="text-xs"><AppIcon icon=PLUS /></span>
                    "Buat Pengajuan"
                </a>
            </div>

            // Filters
            <SectionCard title="Filter" dense=true>
                <div class="flex flex-wrap gap-3">
                    // Search
                    <div class="min-w-[200px] flex-1">
                        <input
                            type="text"
                            placeholder="Cari nama pengajuan..."
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                                set_page.set(1);
                            }
                            prop:value=move || search_query.get()
                        />
                    </div>

                    // Year filter
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_tahun_filter.set(val.parse().ok());
                            set_page.set(1);
                        }
                    >
                        <option value="">"Semua Tahun"</option>
                        {years.iter().map(|y| view! { <option value=y.to_string()>{*y}</option> }).collect_view()}
                    </select>

                    // Status filter
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_status_filter.set(val.parse().ok());
                            set_page.set(1);
                        }
                    >
                        <option value="">"Semua Status"</option>
                        <option value="2000">"Draft"</option>
                        <option value="2001">"Input Barang"</option>
                        <option value="2002">"Diajukan ke Validator"</option>
                        <option value="2003">"Revisi Satker"</option>
                        <option value="2004">"Analisis Kelayakan"</option>
                        <option value="2005">"Penyusunan Prioritas"</option>
                        <option value="2006">"Disetujui"</option>
                        <option value="2007">"Ditolak"</option>
                        <option value="2008">"Selesai"</option>
                        <option value="2009">"Dibatalkan"</option>
                    </select>

                    // Clear selection
                    <Show when=move || !selected_ids.get().is_empty()>
                        <button
                            class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                            on:click=move |_| set_selected_ids.set(Vec::new())
                        >
                            <span class="text-xs"><AppIcon icon=X /></span>
                            "Batal Pilih"
                        </button>
                    </Show>
                </div>
            </SectionCard>

            // Data table
            <div class="mt-4">
                <Suspense fallback=move || view! { <LoadingState /> }>
                    {move || match data_resource.get() {
                        None => view! { <LoadingState /> }.into_any(),
                        Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                        Some(Ok(response)) => {
                            if response.data.is_empty() {
                                view! {
                                    <EmptyState
                                        icon="fas fa-clipboard-list"
                                        title="Belum Ada Pengajuan"
                                        description="Klik tombol \"Buat Pengajuan\" untuk memulai."
                                    />
                                }.into_any()
                            } else {
                                let data_for_for = response.data.clone();
                                let data_for_select_all = response.data.clone();
                                let total = response.total;
                                let total_pages = response.total_pages;

                                view! {
                                    <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-white/[0.04]">
                                                <thead class="bg-white/[0.02]">
                                                    <tr>
                                                        <th class="w-12 px-3 py-3">
                                                            <input
                                                                type="checkbox"
                                                                class="h-4 w-4 rounded border-white/20 bg-white/[0.04] text-gold-500 focus:ring-gold-500/30"
                                                                prop:checked={
                                                                    let data_for_checked = data_for_select_all.clone();
                                                                    move || {
                                                                        let selected = selected_ids.get();
                                                                        let all_ids: Vec<Uuid> = data_for_checked.iter()
                                                                            .filter_map(|item| Uuid::parse_str(&item.id).ok())
                                                                            .collect();
                                                                        !all_ids.is_empty() && all_ids.iter().all(|id| selected.contains(id))
                                                                    }
                                                                }
                                                                on:change=move |_| {
                                                                    let all_ids: Vec<Uuid> = data_for_select_all.iter()
                                                                        .filter_map(|item| Uuid::parse_str(&item.id).ok())
                                                                        .collect();
                                                                    let selected = selected_ids.get();
                                                                    if all_ids.iter().all(|id| selected.contains(id)) {
                                                                        set_selected_ids.set(Vec::new());
                                                                    } else {
                                                                        select_all(all_ids);
                                                                    }
                                                                }
                                                            />
                                                        </th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Pengajuan"</th>
                                                        <th class="px-3 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Tahun"</th>
                                                        <th class="px-3 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Satker"</th>
                                                        <th class="px-3 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Barang"</th>
                                                        <th class="px-3 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Diminta"</th>
                                                        <th class="px-3 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">"Disetujui"</th>
                                                        <th class="px-3 py-3 text-xs font-semibold uppercase tracking-wide text-slate-400">"Status"</th>
                                                        <th class="px-3 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Aksi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {data_for_for.into_iter().map(|item| {
                                                        let status = KebutuhanBmnStatus::from_code(item.status_kode);
                                                        let badge_class = status_badge_class(status.as_ref());
                                                        let id_for_link = item.id.clone();
                                                        let id_for_edit = item.id.clone();
                                                        let item_uuid = Uuid::parse_str(&item.id).ok();

                                                        view! {
                                                            <tr class="border-b border-white/[0.04] transition hover:bg-white/[0.02]">
                                                                <td class="px-3 py-3">
                                                                    {item_uuid.map(|uuid| view! {
                                                                        <input
                                                                            type="checkbox"
                                                                            class="h-4 w-4 rounded border-white/20 bg-white/[0.04] text-gold-500 focus:ring-gold-500/30"
                                                                            prop:checked=move || selected_ids.get().contains(&uuid)
                                                                            on:change=move |_| toggle_selection(uuid)
                                                                        />
                                                                    })}
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <a
                                                                        href=routes::url::kebutuhan_detail(&id_for_link)
                                                                        class="text-sm font-medium text-gold-400 transition hover:text-gold-300"
                                                                    >
                                                                        {item.nama.clone()}
                                                                    </a>
                                                                </td>
                                                                <td class="px-3 py-3 text-center text-sm text-slate-300">{item.tahun}</td>
                                                                <td class="px-3 py-3 text-center">
                                                                    <span class="inline-flex items-center rounded-full bg-info-500/15 px-2 py-0.5 text-xs font-medium text-info-300 ring-1 ring-info-500/25">
                                                                        {item.total_satker}
                                                                    </span>
                                                                </td>
                                                                <td class="px-3 py-3 text-center">
                                                                    <span class="inline-flex items-center rounded-full bg-purple-500/15 px-2 py-0.5 text-xs font-medium text-purple-300 ring-1 ring-purple-500/25">
                                                                        {item.total_barang}
                                                                    </span>
                                                                </td>
                                                                <td class="px-3 py-3 text-right font-mono text-sm text-slate-300">{item.total_jumlah_diminta}</td>
                                                                <td class="px-3 py-3 text-right font-mono text-sm text-success-400">{item.total_jumlah_disetujui}</td>
                                                                <td class="px-3 py-3">
                                                                    <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ring-1 {}", badge_class)>
                                                                        {item.status_nama.clone()}
                                                                    </span>
                                                                </td>
                                                                <td class="px-3 py-3">
                                                                    <div class="flex justify-center gap-3">
                                                                        <a
                                                                            href=routes::url::kebutuhan_detail(&id_for_edit)
                                                                            class="text-info-400 transition hover:text-info-300"
                                                                            title="Detail"
                                                                        >
                                                                            <span class="text-xs"><AppIcon icon=EYE /></span>
                                                                        </a>
                                                                        <a
                                                                            href=routes::url::kebutuhan_edit(&item.id)
                                                                            class="text-slate-400 transition hover:text-slate-200"
                                                                            title="Edit"
                                                                        >
                                                                            <span class="text-xs"><AppIcon icon=PENCIL_SIMPLE /></span>
                                                                        </a>
                                                                    </div>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        </div>

                                        // Pagination
                                        <DarkPagination
                                            current_page=Signal::derive(move || page.get() as i64)
                                            total_pages=total_pages as i64
                                            total_items=total
                                            items_shown=response.data.len()
                                            on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
                                            on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
                                        />
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>

            // Batch operations toolbar (floating at bottom)
            <BatchOperationsToolbar
                selected_ids=selected_ids
                on_operation_complete=handle_operation_complete
            />
        </PageLayout>
    }
}

/// Map KebutuhanBmnStatus to dark-theme badge classes.
fn status_badge_class(status: Option<&KebutuhanBmnStatus>) -> &'static str {
    use KebutuhanBmnStatus::*;
    match status {
        Some(Draft) | Some(Cancelled) => "bg-slate-500/15 text-slate-300 ring-slate-500/25",
        Some(InputBarang) | Some(PenyusunanPrioritas) => {
            "bg-info-500/15 text-info-300 ring-info-500/25"
        }
        Some(SubmitSatker) => "bg-gold-500/15 text-gold-300 ring-gold-500/25",
        Some(RevisiSatker) => "bg-warning-500/15 text-warning-300 ring-warning-500/25",
        Some(AnalisisKelayakan) => "bg-purple-500/15 text-purple-300 ring-purple-500/25",
        Some(Approved) | Some(Completed) => {
            "bg-success-500/15 text-success-300 ring-success-500/25"
        }
        Some(Rejected) => "bg-danger-500/15 text-danger-300 ring-danger-500/25",
        None => "bg-slate-500/15 text-slate-300 ring-slate-500/25",
    }
}
