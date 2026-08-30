use crate::api::{AnalisisKebutuhan, PaginatedResponse, fetch_analisis};
use crate::components::pagination_controls::PaginationControls;
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHART_PIE, EYE, PLUS};

/// leptos-fetch query — keyed by page number (`i32`). Pages 1, 2,
/// 3 each cache independently, so flipping back and forth keeps
/// already-loaded pages instantly visible.
async fn query_analisis_page(page: i32) -> Option<PaginatedResponse<AnalisisKebutuhan>> {
    fetch_analisis(page, 20)
        .await
        .map_err(|e| {
            leptos::logging::error!("Failed to fetch analisis: {:?}", e);
        })
        .ok()
}

#[component]
pub fn AnalisisList() -> impl IntoView {
    let (page, set_page) = signal(1);

    // Per-page leptos-fetch cache — switching pages re-uses cached
    // pages without a network round-trip.
    let client: QueryClient = expect_context();
    let data_resource = client.local_resource(query_analisis_page, move || page.get());

    view! {
        <div class="p-6 bg-surface-panel rounded-xl shadow-sm border border-white/[0.06]">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-slate-100">"Analisis Kebutuhan"</h2>
                <a
                    href=routes::path::ANALITIK_ROADMAP_BUAT
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <span class="mr-2">
                        <AppIcon icon=PLUS />
                    </span>
                    "Buat Analisis Baru"
                </a>
            </div>

            <Suspense fallback=move || {
                view! { <div class="text-center py-8">"Memuat data..."</div> }
            }>
                {move || {
                    data_resource
                        .get()
                        .flatten()
                        .map(|response| {
                            if response.data.is_empty() {
                                view! {
                                    <div class="text-center py-12 text-slate-500">
                                        <span class="text-4xl mb-3 text-slate-600">
                                            <AppIcon icon=CHART_PIE />
                                        </span>
                                        <p>"Belum ada data analisis kebutuhan."</p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="w-full text-left border-collapse">
                                            <thead>
                                                <tr class="bg-white/[0.03] text-slate-400 text-sm uppercase tracking-wider">
                                                    <th class="p-3 font-semibold border-b">"Judul"</th>
                                                    <th class="p-3 font-semibold border-b">"Kategori"</th>
                                                    <th class="p-3 font-semibold border-b">"Prioritas"</th>
                                                    <th class="p-3 font-semibold border-b">"Estimasi Biaya"</th>
                                                    <th class="p-3 font-semibold border-b">"Status"</th>
                                                    <th class="p-3 font-semibold border-b">"Aksi"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="text-slate-200 text-sm">
                                                <For
                                                    each=move || response.data.clone()
                                                    key=|item| item.id.clone()
                                                    children=move |item: AnalisisKebutuhan| {
                                                        view! {
                                                            <tr class="hover:bg-white/[0.04] border-b last:border-0 transition-colors">
                                                                <td class="p-3 font-medium">{item.judul}</td>
                                                                <td class="p-3">
                                                                    <span class="px-2 py-1 rounded-full text-xs bg-purple-50 text-purple-600">
                                                                        {item.kategori}
                                                                    </span>
                                                                </td>
                                                                <td class="p-3">
                                                                    {
                                                                        let prio = item.prioritas.to_lowercase();
                                                                        let color = match prio.as_str() {
                                                                            "tinggi" => "text-danger-400 bg-danger-500/10",
                                                                            "sedang" => "text-warning-400 bg-warning-500/10",
                                                                            _ => "text-success-400 bg-success-500/10",
                                                                        };
                                                                        view! {
                                                                            <span class=format!(
                                                                                "px-2 py-1 rounded-full text-xs capitalize {}",
                                                                                color,
                                                                            )>{item.prioritas}</span>
                                                                        }
                                                                    }
                                                                </td>
                                                                <td class="p-3">
                                                                    {item
                                                                        .estimasi_biaya
                                                                        .map(|n| format!("Rp {:.2}", n))
                                                                        .unwrap_or("-".to_string())}
                                                                </td>
                                                                <td class="p-3">
                                                                    <span class="px-2 py-1 rounded-full text-xs bg-white/[0.05] text-slate-200 capitalize">
                                                                        {item.status}
                                                                    </span>
                                                                </td>
                                                                <td class="p-3">
                                                                    <div class="flex gap-2">
                                                                        <button
                                                                            class="text-info-400 hover:text-blue-800"
                                                                            title="Detail"
                                                                        >
                                                                            <AppIcon icon=EYE />
                                                                        </button>
                                                                    </div>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>

                                        // Pagination — pass `page` signal (reactive) so the
                                        // prev/next button disabled state updates on click.
                                        // `total_pages` is captured from the response snapshot;
                                        // the component re-renders on each fetch so this stays
                                        // in sync with the current data.
                                        {
                                            let total_pages = response.total_pages;
                                            view! {
                                                <PaginationControls
                                                    current_page=page
                                                    total_pages=Signal::derive(move || total_pages)
                                                    total_items=None
                                                    on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
                                                    on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
                                                />
                                            }
                                        }
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
