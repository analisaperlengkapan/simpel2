//! Pakaian Dinas Laporan (Reports) Component
//!
//! Dashboard for viewing various reports on uniform data.

use crate::api::{
    JenisPakaianDinas, LaporanDaftarPegawai, LaporanQuery, LaporanRekapUkuran,
    PengajuanPakaianDinas, fetch_jenis_pakaian_dinas, fetch_laporan_daftar_pegawai,
    fetch_laporan_rekap_ukuran, fetch_pengajuan_pakaian_dinas,
};
use leptos::prelude::*;

#[component]
pub fn PakaianDinasLaporan() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("rekap");
    let (selected_pengajuan, set_selected_pengajuan) = signal(Option::<String>::None);
    let (selected_satker, set_selected_satker) = signal(Option::<String>::None);
    let (selected_jenis, set_selected_jenis) = signal(Option::<String>::None);

    // Fetch filter options
    let pengajuan_options = LocalResource::new(|| async move {
        match fetch_pengajuan_pakaian_dinas(1, 100, None).await {
            Ok(r) => r.data,
            Err(_) => vec![],
        }
    });

    let jenis_options = LocalResource::new(|| async move {
        match fetch_jenis_pakaian_dinas(1, 100).await {
            Ok(r) => r.data,
            Err(_) => vec![],
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Laporan Pakaian Dinas"</h2>
                <p class="text-sm text-gray-500 mt-1">
                    "Lihat rekap dan daftar pegawai berdasarkan ukuran pakaian dinas"
                </p>
            </div>

            // Tabs
            <div class="flex border-b border-gray-200 mb-6">
                <button
                    class=move || {
                        format!(
                            "px-4 py-2 font-medium text-sm border-b-2 transition-colors {}",
                            if active_tab.get() == "rekap" {
                                "border-blue-600 text-blue-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700"
                            },
                        )
                    }
                    on:click=move |_| set_active_tab.set("rekap")
                >
                    <i class="fas fa-chart-pie mr-2"></i>
                    "Rekap Ukuran"
                </button>
                <button
                    class=move || {
                        format!(
                            "px-4 py-2 font-medium text-sm border-b-2 transition-colors {}",
                            if active_tab.get() == "pegawai" {
                                "border-blue-600 text-blue-600"
                            } else {
                                "border-transparent text-gray-500 hover:text-gray-700"
                            },
                        )
                    }
                    on:click=move |_| set_active_tab.set("pegawai")
                >
                    <i class="fas fa-users mr-2"></i>
                    "Daftar Pegawai"
                </button>
            </div>

            // Filters
            <div class="mb-6 p-4 bg-gray-50 rounded-lg">
                <h3 class="text-sm font-semibold text-gray-700 mb-3">"Filter Laporan"</h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    // Pengajuan filter
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">
                            "Periode Pengajuan"
                        </label>
                        <Suspense fallback=move || {
                            view! { <select class="w-full px-3 py-2 border rounded-lg"></select> }
                        }>
                            {move || {
                                pengajuan_options
                                    .get()
                                    .map(|options| {
                                        view! {
                                            <select
                                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_selected_pengajuan
                                                        .set(if val.is_empty() { None } else { Some(val) });
                                                }
                                            >
                                                <option value="">"Semua Periode"</option>
                                                <For
                                                    each=move || options.clone()
                                                    key=|p| p.id.clone()
                                                    children=move |p: PengajuanPakaianDinas| {
                                                        view! {
                                                            <option value=p.id.clone()>
                                                                {format!("{} ({})", p.nama, p.tahun)}
                                                            </option>
                                                        }
                                                    }
                                                />
                                            </select>
                                        }
                                    })
                            }}
                        </Suspense>
                    </div>

                    // Jenis Pakaian filter
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">
                            "Jenis Pakaian"
                        </label>
                        <Suspense fallback=move || {
                            view! { <select class="w-full px-3 py-2 border rounded-lg"></select> }
                        }>
                            {move || {
                                jenis_options
                                    .get()
                                    .map(|options| {
                                        view! {
                                            <select
                                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                                                on:change=move |ev| {
                                                    let val = event_target_value(&ev);
                                                    set_selected_jenis
                                                        .set(if val.is_empty() { None } else { Some(val) });
                                                }
                                            >
                                                <option value="">"Semua Jenis"</option>
                                                <For
                                                    each=move || options.clone()
                                                    key=|j| j.id.clone()
                                                    children=move |j: JenisPakaianDinas| {
                                                        view! { <option value=j.id.clone()>{j.nama}</option> }
                                                    }
                                                />
                                            </select>
                                        }
                                    })
                            }}
                        </Suspense>
                    </div>

                    // Satker filter (text input for now)
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Satker"</label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                            placeholder="ID Satker (opsional)"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_selected_satker.set(if val.is_empty() { None } else { Some(val) });
                            }
                        />
                    </div>
                </div>
            </div>

            // Tab Content
            <Show when=move || active_tab.get() == "rekap">
                <RekapUkuranTab
                    pengajuan_id=selected_pengajuan.get()
                    satker_id=selected_satker.get()
                    jenis_pakaian_id=selected_jenis.get()
                />
            </Show>

            <Show when=move || active_tab.get() == "pegawai">
                <DaftarPegawaiTab
                    pengajuan_id=selected_pengajuan.get()
                    satker_id=selected_satker.get()
                    jenis_pakaian_id=selected_jenis.get()
                />
            </Show>
        </div>
    }
}

#[component]
fn RekapUkuranTab(
    pengajuan_id: Option<String>,
    satker_id: Option<String>,
    jenis_pakaian_id: Option<String>,
) -> impl IntoView {
    let data = LocalResource::new(move || {
        let query = LaporanQuery {
            pengajuan_id: pengajuan_id.clone(),
            satker_id: satker_id.clone(),
            jenis_pakaian_id: jenis_pakaian_id.clone(),
        };
        async move {
            match fetch_laporan_rekap_ukuran(query).await {
                Ok(r) => Some(r.data),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch rekap: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { <div class="text-center py-8">"Memuat data rekap..."</div> }
        }>
            {move || {
                data.get()
                    .flatten()
                    .map(|rekap| {
                        if rekap.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-chart-bar text-4xl mb-3 text-gray-300"></i>
                                    <p>"Tidak ada data untuk filter yang dipilih."</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            // Group by group (BAJU, CELANA, SEPATU)
                            let baju: Vec<_> = rekap
                                .iter()
                                .filter(|r| r.group == "BAJU")
                                .cloned()
                                .collect();
                            let celana: Vec<_> = rekap
                                .iter()
                                .filter(|r| r.group == "CELANA")
                                .cloned()
                                .collect();
                            let sepatu: Vec<_> = rekap
                                .iter()
                                .filter(|r| r.group == "SEPATU")
                                .cloned()
                                .collect();

                            // Clone for iteration in For loop
                            let baju_for_for = baju.clone();
                            let celana_for_for = celana.clone();
                            let sepatu_for_for = sepatu.clone();

                            // Clone for sum calculation
                            let baju_for_sum = baju.clone();
                            let celana_for_sum = celana.clone();
                            let sepatu_for_sum = sepatu.clone();

                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                    // Baju card
                                    <div class="bg-blue-50 rounded-lg p-4 border border-blue-200">
                                        <h4 class="font-semibold text-blue-800 mb-3">
                                            <i class="fas fa-tshirt mr-2"></i>
                                            "Rekap Ukuran Baju"
                                        </h4>
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="text-blue-600">
                                                    <th class="text-left p-1">"Ukuran"</th>
                                                    <th class="text-right p-1">"Jumlah"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || baju_for_for.clone()
                                                    key=|r| format!("{}-{}", r.group, r.size)
                                                    children=move |r: LaporanRekapUkuran| {
                                                        view! {
                                                            <tr class="border-t border-blue-100">
                                                                <td class="p-1 font-medium">{r.size}</td>
                                                                <td class="p-1 text-right">{r.jumlah.to_string()}</td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                            <tfoot>
                                                <tr class="border-t-2 border-blue-300 font-semibold">
                                                    <td class="p-1">"Total"</td>
                                                    <td class="p-1 text-right">
                                                        {baju_for_sum.iter().map(|r| r.jumlah).sum::<i64>().to_string()}
                                                    </td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    </div>

                                    // Celana card
                                    <div class="bg-green-50 rounded-lg p-4 border border-green-200">
                                        <h4 class="font-semibold text-green-800 mb-3">
                                            <i class="fas fa-male mr-2"></i>
                                            "Rekap Ukuran Celana"
                                        </h4>
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="text-green-600">
                                                    <th class="text-left p-1">"Ukuran"</th>
                                                    <th class="text-right p-1">"Jumlah"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || celana_for_for.clone()
                                                    key=|r| format!("{}-{}", r.group, r.size)
                                                    children=move |r: LaporanRekapUkuran| {
                                                        view! {
                                                            <tr class="border-t border-green-100">
                                                                <td class="p-1 font-medium">{r.size}</td>
                                                                <td class="p-1 text-right">{r.jumlah.to_string()}</td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                            <tfoot>
                                                <tr class="border-t-2 border-green-300 font-semibold">
                                                    <td class="p-1">"Total"</td>
                                                    <td class="p-1 text-right">
                                                        {celana_for_sum.iter().map(|r| r.jumlah).sum::<i64>().to_string()}
                                                    </td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    </div>

                                    // Sepatu card
                                    <div class="bg-orange-50 rounded-lg p-4 border border-orange-200">
                                        <h4 class="font-semibold text-orange-800 mb-3">
                                            <i class="fas fa-shoe-prints mr-2"></i>
                                            "Rekap Ukuran Sepatu"
                                        </h4>
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="text-orange-600">
                                                    <th class="text-left p-1">"Ukuran"</th>
                                                    <th class="text-right p-1">"Jumlah"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || sepatu_for_for.clone()
                                                    key=|r| format!("{}-{}", r.group, r.size)
                                                    children=move |r: LaporanRekapUkuran| {
                                                        view! {
                                                            <tr class="border-t border-orange-100">
                                                                <td class="p-1 font-medium">{r.size}</td>
                                                                <td class="p-1 text-right">{r.jumlah.to_string()}</td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                            <tfoot>
                                                <tr class="border-t-2 border-orange-300 font-semibold">
                                                    <td class="p-1">"Total"</td>
                                                    <td class="p-1 text-right">
                                                        {sepatu_for_sum.iter().map(|r| r.jumlah).sum::<i64>().to_string()}
                                                    </td>
                                                </tr>
                                            </tfoot>
                                        </table>
                                    </div>
                                </div>

                                // Export buttons
                                <div class="mt-6 flex gap-2">
                                    <button class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700">
                                        <i class="fas fa-file-excel mr-2"></i>
                                        "Export Excel"
                                    </button>
                                    <button class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700">
                                        <i class="fas fa-file-pdf mr-2"></i>
                                        "Export PDF"
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn DaftarPegawaiTab(
    pengajuan_id: Option<String>,
    satker_id: Option<String>,
    jenis_pakaian_id: Option<String>,
) -> impl IntoView {
    let (page, set_page) = signal(1);

    let data = LocalResource::new(move || {
        let query = LaporanQuery {
            pengajuan_id: pengajuan_id.clone(),
            satker_id: satker_id.clone(),
            jenis_pakaian_id: jenis_pakaian_id.clone(),
        };
        let p = page.get();
        async move {
            match fetch_laporan_daftar_pegawai(query, p, 20).await {
                Ok(r) => Some(r),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch daftar pegawai: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { <div class="text-center py-8">"Memuat daftar pegawai..."</div> }
        }>
            {move || {
                data.get()
                    .flatten()
                    .map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-users text-4xl mb-3 text-gray-300"></i>
                                    <p>"Tidak ada data untuk filter yang dipilih."</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                        <thead>
                                            <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                <th class="p-3 font-semibold border-b">"No"</th>
                                                <th class="p-3 font-semibold border-b">"NIP"</th>
                                                <th class="p-3 font-semibold border-b">"Nama"</th>
                                                <th class="p-3 font-semibold border-b">"Satker"</th>
                                                <th class="p-3 font-semibold border-b">"Jabatan"</th>
                                                <th class="p-3 font-semibold border-b text-center">"L/P"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Baju"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Celana"</th>
                                                <th class="p-3 font-semibold border-b text-center">"Sepatu"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="text-gray-700 text-sm">
                                            <For
                                                each=move || response.data.clone().into_iter().enumerate()
                                                key=|(_, item)| item.nip.clone()
                                                children=move |(idx, item): (usize, LaporanDaftarPegawai)| {
                                                    view! {
                                                        <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                            <td class="p-3">
                                                                {((page.get() - 1) * 20 + idx as i32 + 1).to_string()}
                                                            </td>
                                                            <td class="p-3 font-mono text-sm">{item.nip}</td>
                                                            <td class="p-3 font-medium">{item.nama}</td>
                                                            <td class="p-3">{item.satker_nama}</td>
                                                            <td class="p-3">
                                                                {item.jabatan.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center">
                                                                {item.jenis_kelamin.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item.ukuran_baju.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item.ukuran_celana.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item.ukuran_sepatu.unwrap_or_else(|| "-".to_string())}
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
                                            "Menampilkan halaman "
                                            <span class="font-medium">{response.page}</span> " dari "
                                            <span class="font-medium">{response.total_pages}</span>
                                            " (" <span class="font-medium">{response.total}</span>
                                            " pegawai)"
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

                                // Export buttons
                                <div class="mt-6 flex gap-2">
                                    <button class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700">
                                        <i class="fas fa-file-excel mr-2"></i>
                                        "Export Excel"
                                    </button>
                                    <button class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700">
                                        <i class="fas fa-file-pdf mr-2"></i>
                                        "Export PDF"
                                    </button>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
