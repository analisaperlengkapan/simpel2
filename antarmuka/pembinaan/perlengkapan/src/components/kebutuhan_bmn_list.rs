//! Kebutuhan BMN List Component
//!
//! Displays paginated list of BMN needs analysis requests with filtering.

use crate::api::{
    KebutuhanBmnQuery, KebutuhanBmnStatus, KebutuhanBmnSummary, fetch_kebutuhan_bmn_list,
};
use leptos::prelude::*;

#[component]
pub fn KebutuhanBmnList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (per_page, _set_per_page) = signal(20);
    let (tahun_filter, set_tahun_filter) = signal::<Option<i32>>(None);
    let (status_filter, set_status_filter) = signal::<Option<i32>>(None);
    let (search_query, set_search_query) = signal(String::new());

    // Create query from filters
    let query = Memo::new(move |_| KebutuhanBmnQuery {
        tahun: tahun_filter.get(),
        status_kode: status_filter.get(),
        satker_id: None,
        search: {
            let s = search_query.get();
            if s.is_empty() { None } else { Some(s) }
        },
    });

    // Fetch data resource
    let data_resource = LocalResource::new(move || {
        let q = query.get();
        let p = page.get();
        let pp = per_page.get();
        async move {
            match fetch_kebutuhan_bmn_list(q, p, pp).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch kebutuhan BMN: {:?}", e);
                    None
                }
            }
        }
    });

    // Reset page when filters change
    Effect::new(move || {
        let _ = query.get();
        set_page.set(1);
    });

    // Current year for filter dropdown
    let current_year = 2025;
    let years: Vec<i32> = (2020..=current_year + 1).rev().collect();

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            // Header
            <div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4 mb-6">
                <div>
                    <h2 class="text-xl font-bold text-gray-800">"Analisis Kebutuhan BMN"</h2>
                    <p class="text-sm text-gray-500 mt-1">"Kelola pengajuan kebutuhan barang milik negara"</p>
                </div>
                <a
                    href="/dashboard/kebutuhan-bmn/baru"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Buat Pengajuan"
                </a>
            </div>

            // Filters
            <div class="flex flex-wrap gap-3 mb-6 p-4 bg-gray-50 rounded-lg">
                // Search
                <div class="flex-1 min-w-[200px]">
                    <input
                        type="text"
                        placeholder="Cari nama pengajuan..."
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        prop:value=move || search_query.get()
                    />
                </div>

                // Year filter
                <select
                    class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 bg-white"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_tahun_filter.set(val.parse().ok());
                    }
                >
                    <option value="">"Semua Tahun"</option>
                    <For
                        each=move || years.clone()
                        key=|y| *y
                        children=move |y| view! { <option value=y.to_string()>{y}</option> }
                    />
                </select>

                // Status filter
                <select
                    class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 bg-white"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_status_filter.set(val.parse().ok());
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
            </div>

            // Data table
            <Suspense fallback=move || view! {
                <div class="text-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-3"></div>
                    <p class="text-gray-500">"Memuat data..."</p>
                </div>
            }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        // Pre-clone data for multiple uses
                        let data_for_for = response.data.clone();
                        let data_len = response.data.len();
                        let total = response.total;
                        let total_pages = response.total_pages;

                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-folder-open text-4xl mb-3 text-gray-300"></i>
                                    <p class="font-medium">"Belum ada pengajuan kebutuhan BMN"</p>
                                    <p class="text-sm mt-1">"Klik tombol \"Buat Pengajuan\" untuk memulai"</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                <th class="p-3 font-semibold border-b">"Nama Pengajuan"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Tahun"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Satker"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Barang"</th>
                                                <th class="p-3 font-semibold border-b text-right">"Diminta"</th>
                                                <th class="p-3 font-semibold border-b text-right">"Disetujui"</th>
                                                <th class="p-3 font-semibold border-b">"Status"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Aksi"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="text-gray-700 text-sm">
                                            <For
                                                each=move || data_for_for.clone()
                                                key=|item| item.id.clone()
                                                children=move |item: KebutuhanBmnSummary| {
                                                    let status = KebutuhanBmnStatus::from_code(item.status_kode);
                                                    let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-gray-100 text-gray-800");
                                                    let id_for_link = item.id.clone();
                                                    let id_for_edit = item.id.clone();

                                                    view! {
                                                        <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                            <td class="p-3">
                                                                <a
                                                                    href=format!("/dashboard/kebutuhan-bmn/{}", id_for_link)
                                                                    class="font-medium text-blue-600 hover:text-blue-800"
                                                                >
                                                                    {item.nama.clone()}
                                                                </a>
                                                            </td>
                                                            <td class="p-3 text-center">{item.tahun}</td>
                                                            <td class="p-3 text-center">
                                                                <span class="bg-blue-50 text-blue-700 px-2 py-1 rounded-full text-xs font-medium">
                                                                    {item.total_satker}
                                                                </span>
                                                            </td>
                                                            <td class="p-3 text-center">
                                                                <span class="bg-purple-50 text-purple-700 px-2 py-1 rounded-full text-xs font-medium">
                                                                    {item.total_barang}
                                                                </span>
                                                            </td>
                                                            <td class="p-3 text-right font-mono">{item.total_jumlah_diminta}</td>
                                                            <td class="p-3 text-right font-mono text-green-600">{item.total_jumlah_disetujui}</td>
                                                            <td class="p-3">
                                                                <span class=format!("px-2 py-1 rounded-full text-xs font-medium {}", badge_class)>
                                                                    {item.status_nama.clone()}
                                                                </span>
                                                            </td>
                                                            <td class="p-3">
                                                                <div class="flex justify-center gap-2">
                                                                    <a
                                                                        href=format!("/dashboard/kebutuhan-bmn/{}", id_for_edit)
                                                                        class="text-blue-600 hover:text-blue-800 p-1"
                                                                        title="Detail"
                                                                    >
                                                                        <i class="fas fa-eye"></i>
                                                                    </a>
                                                                    <a
                                                                        href=format!("/dashboard/kebutuhan-bmn/{}/edit", item.id)
                                                                        class="text-gray-600 hover:text-gray-800 p-1"
                                                                        title="Edit"
                                                                    >
                                                                        <i class="fas fa-edit"></i>
                                                                    </a>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
                                </div>

                                // Pagination
                                <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                    <div class="text-sm text-gray-500">
                                        "Menampilkan "
                                        <span class="font-medium">{response.data.len()}</span>
                                        " dari "
                                        <span class="font-medium">{response.total}</span>
                                        " pengajuan"
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            class="px-4 py-2 border rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                            prop:disabled=move || page.get() <= 1
                                            on:click=move |_| set_page.update(|p| *p -= 1)
                                        >
                                            <i class="fas fa-chevron-left mr-1"></i>
                                            "Sebelumnya"
                                        </button>
                                        <span class="px-4 py-2 text-gray-600">
                                            "Halaman " {move || page.get()} " / " {response.total_pages}
                                        </span>
                                        <button
                                            class="px-4 py-2 border rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                                            prop:disabled=move || page.get() >= response.total_pages
                                            on:click=move |_| set_page.update(|p| *p += 1)
                                        >
                                            "Selanjutnya"
                                            <i class="fas fa-chevron-right ml-1"></i>
                                        </button>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
