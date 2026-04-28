use crate::api::{
    PaginatedResponse, PenghapusanBmnFilters, PenghapusanBmnWorkflow, fetch_penghapusan_bmn_list,
};
use crate::components::list_feedback::{EmptyState, LoadingState};
use crate::components::page_header::PageHeader;
use crate::components::pagination_controls::PaginationControls;
use crate::routes;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::EYE;

/// leptos-fetch query keyed by `(page, status_kode)`. Other filter
/// dimensions (satker, status string, metode, tahun) are pinned to
/// `None` for now; if the screen grows extra filter controls,
/// extend the key tuple to match.
async fn query_penghapusan_page(
    key: (i32, Option<i32>),
) -> Option<PaginatedResponse<PenghapusanBmnWorkflow>> {
    let (page, status_kode) = key;
    let filters = PenghapusanBmnFilters {
        status_kode,
        satker_id: None,
        status: None,
        metode_penghapusan: None,
        tahun: None,
    };
    fetch_penghapusan_bmn_list(page, 20, filters)
        .await
        .map_err(|e| {
            leptos::logging::error!("Failed to fetch penghapusan: {:?}", e);
        })
        .ok()
}

/// Returns (bg_class, text_class, label) for a given status_kode
fn status_badge(kode: i32) -> (&'static str, &'static str, &'static str) {
    match kode {
        4000 => ("bg-gray-100", "text-gray-700", "Draft"),
        4001 => ("bg-blue-100", "text-blue-700", "Submit Wilayah"),
        4002 => ("bg-yellow-100", "text-yellow-800", "Revisi Operator"),
        4003 => ("bg-indigo-100", "text-indigo-700", "Submit Pusat"),
        4004 => ("bg-purple-100", "text-purple-700", "Verifikasi Pusat"),
        4005 => ("bg-cyan-100", "text-cyan-700", "Konsep SK Generated"),
        4006 => ("bg-teal-100", "text-teal-700", "SK Ditandatangani"),
        4007 => ("bg-green-100", "text-green-700", "Selesai"),
        4008 => ("bg-red-100", "text-red-700", "Ditolak"),
        _ => ("bg-gray-100", "text-gray-600", "Unknown"),
    }
}

#[component]
pub fn PenghapusanList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (filter_status, set_filter_status) = signal(None::<i32>);

    // Resource — keyed cache per (page, status). Each combination
    // gets its own cache slot so toggling filters or paging back is
    // instant after the first fetch.
    let client: QueryClient = expect_context();
    let data_resource = client.local_resource(query_penghapusan_page, move || {
        (page.get(), filter_status.get())
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <PageHeader
                title="Usulan SK Penghapusan BMN"
                action_href=routes::path::PENGELOLAAN_PENGHAPUSAN_BUAT
                action_label="Usul Penghapusan"
            />

            // Status filter bar
            <div class="flex flex-wrap gap-2 mb-4">
                <button
                    class=move || if filter_status.get().is_none() {
                        "px-3 py-1 text-sm rounded-full bg-blue-600 text-white"
                    } else {
                        "px-3 py-1 text-sm rounded-full bg-gray-100 text-gray-700 hover:bg-gray-200"
                    }
                    on:click=move |_| { set_filter_status.set(None); set_page.set(1); }
                >"Semua"</button>
                {[
                    (4000, "Draft"), (4001, "Submit Wilayah"), (4003, "Submit Pusat"),
                    (4005, "Konsep SK"), (4007, "Selesai"), (4008, "Ditolak"),
                ].into_iter().map(|(kode, label)| {
                    view! {
                        <button
                            class=move || if filter_status.get() == Some(kode) {
                                "px-3 py-1 text-sm rounded-full bg-blue-600 text-white"
                            } else {
                                "px-3 py-1 text-sm rounded-full bg-gray-100 text-gray-700 hover:bg-gray-200"
                            }
                            on:click=move |_| { set_filter_status.set(Some(kode)); set_page.set(1); }
                        >{label}</button>
                    }
                }).collect_view()}
            </div>

            <Suspense fallback=move || view! { <LoadingState /> }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <EmptyState
                                    title="Belum ada data Usulan SK Penghapusan BMN."
                                    icon_class="fas fa-clipboard-list"
                                />
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                            <th class="p-3 font-semibold border-b">"Kode Barang"</th>
                                            <th class="p-3 font-semibold border-b">"Nama Barang"</th>
                                            <th class="p-3 font-semibold border-b">"Metode"</th>
                                            <th class="p-3 font-semibold border-b">"Tanggal"</th>
                                            <th class="p-3 font-semibold border-b">"Status"</th>
                                            <th class="p-3 font-semibold border-b">"Aksi"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="text-gray-700 text-sm">
                                        <For
                                            each=move || response.data.clone()
                                            key=|item| item.id.clone()
                                            children=move |item: PenghapusanBmnWorkflow| {
                                                let (bg, tc, lbl) = status_badge(item.status_kode);
                                                let detail_url = format!("/perlengkapan/pengelolaan/penghapusan/{}", item.id);
                                                view! {
                                                    <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                        <td class="p-3 font-mono text-xs">{item.kode_barang}</td>
                                                        <td class="p-3 font-medium">{item.nama_barang}</td>
                                                        <td class="p-3">{item.metode_penghapusan}</td>
                                                        <td class="p-3">{item.tanggal_penghapusan}</td>
                                                        <td class="p-3">
                                                            <span class=format!("px-2 py-1 rounded-full text-xs font-medium {} {}", bg, tc)>
                                                                {lbl}
                                                            </span>
                                                        </td>
                                                        <td class="p-3">
                                                            <a
                                                                href=detail_url
                                                                class="text-blue-600 hover:text-blue-800 inline-flex items-center gap-1"
                                                                title="Lihat Detail"
                                                            >
                                                                <AppIcon icon=EYE />
                                                                <span class="text-xs">"Detail"</span>
                                                            </a>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>

                                <PaginationControls
                                    current_page=response.page
                                    total_pages=response.total_pages
                                    total_items=None
                                    on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
                                    on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
                                />
                            </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
