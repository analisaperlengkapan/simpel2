use crate::api::{PenghapusanBmnWorkflow, fetch_penghapusan_bmn_list, PenghapusanBmnFilters};
use leptos::prelude::*;

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

    // Resource to fetch items when page or filter changes
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let status = filter_status.get();
        async move {
            let filters = PenghapusanBmnFilters {
                status_kode: status,
                satker_id: None,
                status: None,
                metode_penghapusan: None,
                tahun: None,
            };
            match fetch_penghapusan_bmn_list(p, 20, filters).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch penghapusan: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Usulan SK Penghapusan BMN"</h2>
                <a
                    href="/dashboard/pengelolaan/penghapusan/baru"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Usul Penghapusan"
                </a>
            </div>

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

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-clipboard-list text-4xl mb-3 text-gray-300"></i>
                                    <p>"Belum ada data Usulan SK Penghapusan BMN."</p>
                                </div>
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
                                                let detail_url = format!("/dashboard/pengelolaan/penghapusan/{}", item.id);
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
                                                                <i class="fas fa-eye"></i>
                                                                <span class="text-xs">"Detail"</span>
                                                            </a>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>

                                // Pagination
                                <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                    <div class="text-sm text-gray-500">
                                        "Menampilkan halaman " <span class="font-medium">{response.page}</span> " dari " <span class="font-medium">{response.total_pages}</span>
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || page.get() <= 1
                                            on:click=move |_| set_page.update(|p| *p -= 1)
                                        >
                                            "Sebelumnya"
                                        </button>
                                        <button
                                            class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || page.get() >= response.total_pages
                                            on:click=move |_| set_page.update(|p| *p += 1)
                                        >
                                            "Selanjutnya"
                                        </button>
                                    </div>
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
