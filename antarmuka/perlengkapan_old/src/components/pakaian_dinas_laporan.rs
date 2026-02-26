//! Pakaian Dinas Laporan (Reports) Component
//!
//! Dashboard for viewing various reports on uniform data.
//! Format follows simpel_web-main: cetakRekapTemplateV & cetakDaftarTemplateV

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
    let (selected_jenis_kelamin, set_selected_jenis_kelamin) = signal(Option::<String>::None);
    let (selected_jenis_pegawai, set_selected_jenis_pegawai) = signal(Option::<String>::None);
    let (selected_eselon, set_selected_eselon) = signal(Option::<String>::None);

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

            // Filters — matching simpel_web-main laporanV.blade.php
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

                    // Satker filter
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

                    // Jenis Kelamin filter
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Jenis Kelamin"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_selected_jenis_kelamin.set(if val.is_empty() { None } else { Some(val) });
                            }
                        >
                            <option value="">"Semua"</option>
                            <option value="L">"Laki-laki"</option>
                            <option value="P">"Perempuan"</option>
                        </select>
                    </div>

                    // Jenis Pegawai filter (Jaksa / TU)
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Jenis Pegawai"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_selected_jenis_pegawai.set(if val.is_empty() { None } else { Some(val) });
                            }
                        >
                            <option value="">"Semua"</option>
                            <option value="0">"Jaksa"</option>
                            <option value="1">"Tata Usaha"</option>
                        </select>
                    </div>

                    // Eselon filter
                    <div>
                        <label class="block text-xs font-medium text-gray-600 mb-1">"Eselon"</label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_selected_eselon.set(if val.is_empty() { None } else { Some(val) });
                            }
                        >
                            <option value="">"Semua Eselon"</option>
                            <option value="I">"Eselon I"</option>
                            <option value="II">"Eselon II"</option>
                            <option value="III">"Eselon III"</option>
                            <option value="IV">"Eselon IV"</option>
                        </select>
                    </div>
                </div>
            </div>

            // Export buttons — matching simpel_web-main cetak functionality
            <div class="mb-6 flex gap-3">
                <button
                    class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors flex items-center gap-2 text-sm font-medium"
                    on:click=move |_| {
                        let jenis_laporan = active_tab.get();
                        let mut url = format!(
                            "/api/pembinaan/perlengkapan/pakaian-dinas/laporan/cetak?jenis_laporan={}&jenis_file=excel",
                            jenis_laporan
                        );
                        if let Some(ref pid) = selected_pengajuan.get() {
                            url.push_str(&format!("&pengajuan_id={}", pid));
                        }
                        if let Some(ref sid) = selected_satker.get() {
                            url.push_str(&format!("&satker_id={}", sid));
                        }
                        if let Some(ref jk) = selected_jenis_kelamin.get() {
                            url.push_str(&format!("&jenis_kelamin={}", jk));
                        }
                        if let Some(ref e) = selected_eselon.get() {
                            url.push_str(&format!("&eselon={}", e));
                        }
                        if let Some(ref j) = selected_jenis_pegawai.get() {
                            url.push_str(&format!("&jenis={}", j));
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.open_with_url_and_target(&url, "_blank");
                            }
                        }
                    }
                >
                    <i class="fas fa-file-excel"></i>
                    "Cetak Excel"
                </button>
                <button
                    class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors flex items-center gap-2 text-sm font-medium"
                    on:click=move |_| {
                        let jenis_laporan = active_tab.get();
                        let mut url = format!(
                            "/api/pembinaan/perlengkapan/pakaian-dinas/laporan/cetak?jenis_laporan={}&jenis_file=pdf",
                            jenis_laporan
                        );
                        if let Some(ref pid) = selected_pengajuan.get() {
                            url.push_str(&format!("&pengajuan_id={}", pid));
                        }
                        if let Some(ref sid) = selected_satker.get() {
                            url.push_str(&format!("&satker_id={}", sid));
                        }
                        if let Some(ref jk) = selected_jenis_kelamin.get() {
                            url.push_str(&format!("&jenis_kelamin={}", jk));
                        }
                        if let Some(ref e) = selected_eselon.get() {
                            url.push_str(&format!("&eselon={}", e));
                        }
                        if let Some(ref j) = selected_jenis_pegawai.get() {
                            url.push_str(&format!("&jenis={}", j));
                        }
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let _ = window.open_with_url_and_target(&url, "_blank");
                            }
                        }
                    }
                >
                    <i class="fas fa-file-pdf"></i>
                    "Cetak PDF"
                </button>
            </div>

            // Tab Content
            <Show when=move || active_tab.get() == "rekap">
                <RekapUkuranTab
                    pengajuan_id=selected_pengajuan.get()
                    satker_id=selected_satker.get()
                    jenis_pakaian_id=selected_jenis.get()
                    jenis_kelamin=selected_jenis_kelamin.get()
                    jenis=selected_jenis_pegawai.get()
                    eselon=selected_eselon.get()
                />
            </Show>

            <Show when=move || active_tab.get() == "pegawai">
                <DaftarPegawaiTab
                    pengajuan_id=selected_pengajuan.get()
                    satker_id=selected_satker.get()
                    jenis_pakaian_id=selected_jenis.get()
                    jenis_kelamin=selected_jenis_kelamin.get()
                    jenis=selected_jenis_pegawai.get()
                    eselon=selected_eselon.get()
                />
            </Show>
        </div>
    }
}

/// Rekap Ukuran Tab — matching simpel_web-main cetakRekapTemplateV.blade.php
///
/// Shows a table per spesifikasi (pakaian_nama) with:
/// - Columns: Ukuran values
/// - Rows: jumlah_laki, jumlah_perempuan, jumlah_total
#[component]
fn RekapUkuranTab(
    pengajuan_id: Option<String>,
    satker_id: Option<String>,
    jenis_pakaian_id: Option<String>,
    jenis_kelamin: Option<String>,
    jenis: Option<String>,
    eselon: Option<String>,
) -> impl IntoView {
    let data = LocalResource::new(move || {
        let query = LaporanQuery {
            pengajuan_id: pengajuan_id.clone(),
            satker_id: satker_id.clone(),
            jenis_pakaian_id: jenis_pakaian_id.clone(),
            jenis_kelamin: jenis_kelamin.clone(),
            eselon: eselon.clone(),
            jenis: jenis.clone(),
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
                            // Group by pakaian_nama (spesifikasi)
                            let mut groups: Vec<String> = vec![];
                            for r in &rekap {
                                if !groups.contains(&r.pakaian_nama) {
                                    groups.push(r.pakaian_nama.clone());
                                }
                            }

                            let groups_for_view = groups.clone();
                            let rekap_for_view = rekap.clone();

                            view! {
                                <div class="space-y-8">
                                    {groups_for_view
                                        .into_iter()
                                        .map(|group_name| {
                                            let items: Vec<LaporanRekapUkuran> = rekap_for_view
                                                .iter()
                                                .filter(|r| r.pakaian_nama == group_name)
                                                .cloned()
                                                .collect();
                                            let ukuran_group = items
                                                .first()
                                                .map(|r| r.ukuran_group.clone())
                                                .unwrap_or_default();
                                            let items_for_header = items.clone();
                                            let items_for_body_l = items.clone();
                                            let items_for_body_p = items.clone();
                                            let items_for_body_total = items.clone();
                                            let total_l: i64 = items.iter().map(|r| r.jumlah_laki).sum();
                                            let total_p: i64 = items.iter().map(|r| r.jumlah_perempuan).sum();
                                            let total_all: i64 = items.iter().map(|r| r.jumlah_total).sum();
                                            let color_class = match ukuran_group.as_str() {
                                                "BAJU" => "blue",
                                                "CELANA" => "green",
                                                "SEPATU" => "orange",
                                                _ => "gray",
                                            };

                                            view! {
                                                <div class=format!(
                                                    "bg-{}-50 rounded-lg p-4 border border-{}-200",
                                                    color_class, color_class,
                                                )>
                                                    <h4 class=format!(
                                                        "font-semibold text-{}-800 mb-3 text-lg",
                                                        color_class,
                                                    )>
                                                        {format!("{} ({})", group_name, ukuran_group)}
                                                    </h4>
                                                    <div class="overflow-x-auto">
                                                        <table class="w-full text-sm border-collapse">
                                                            <thead>
                                                                <tr class=format!(
                                                                    "bg-{}-100 text-{}-700",
                                                                    color_class, color_class,
                                                                )>
                                                                    <th class="p-2 text-left border font-semibold">
                                                                        "Gender"
                                                                    </th>
                                                                    {items_for_header
                                                                        .iter()
                                                                        .map(|r| {
                                                                            view! {
                                                                                <th class="p-2 text-center border font-semibold">
                                                                                    {r.ukuran.clone()}
                                                                                </th>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
                                                                    <th class="p-2 text-center border font-bold">
                                                                        "Total"
                                                                    </th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                // Laki-laki row
                                                                <tr class="border-b">
                                                                    <td class="p-2 font-medium border">
                                                                        "Laki-laki (L)"
                                                                    </td>
                                                                    {items_for_body_l
                                                                        .iter()
                                                                        .map(|r| {
                                                                            view! {
                                                                                <td class="p-2 text-center border">
                                                                                    {r.jumlah_laki.to_string()}
                                                                                </td>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
                                                                    <td class="p-2 text-center border font-bold">
                                                                        {total_l.to_string()}
                                                                    </td>
                                                                </tr>
                                                                // Perempuan row
                                                                <tr class="border-b">
                                                                    <td class="p-2 font-medium border">
                                                                        "Perempuan (P)"
                                                                    </td>
                                                                    {items_for_body_p
                                                                        .iter()
                                                                        .map(|r| {
                                                                            view! {
                                                                                <td class="p-2 text-center border">
                                                                                    {r.jumlah_perempuan.to_string()}
                                                                                </td>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
                                                                    <td class="p-2 text-center border font-bold">
                                                                        {total_p.to_string()}
                                                                    </td>
                                                                </tr>
                                                                // Total row
                                                                <tr class=format!(
                                                                    "bg-{}-100 font-semibold",
                                                                    color_class,
                                                                )>
                                                                    <td class="p-2 font-bold border">"Jumlah"</td>
                                                                    {items_for_body_total
                                                                        .iter()
                                                                        .map(|r| {
                                                                            view! {
                                                                                <td class="p-2 text-center border font-bold">
                                                                                    {r.jumlah_total.to_string()}
                                                                                </td>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
                                                                    <td class="p-2 text-center border font-bold">
                                                                        {total_all.to_string()}
                                                                    </td>
                                                                </tr>
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}

/// Daftar Pegawai Tab — matching simpel_web-main cetakDaftarTemplateV.blade.php
///
/// Table columns: No, NIP, Nama, Jabatan, Golongan, Status (J/T), Gender, Busana Muslimah,
/// then ukuran columns (Baju, Celana, Sepatu)
#[component]
fn DaftarPegawaiTab(
    pengajuan_id: Option<String>,
    satker_id: Option<String>,
    jenis_pakaian_id: Option<String>,
    jenis_kelamin: Option<String>,
    jenis: Option<String>,
    eselon: Option<String>,
) -> impl IntoView {
    let (page, set_page) = signal(1);

    let data = LocalResource::new(move || {
        let query = LaporanQuery {
            pengajuan_id: pengajuan_id.clone(),
            satker_id: satker_id.clone(),
            jenis_pakaian_id: jenis_pakaian_id.clone(),
            jenis_kelamin: jenis_kelamin.clone(),
            eselon: eselon.clone(),
            jenis: jenis.clone(),
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
                                                <th class="p-3 font-semibold border-b">"Jabatan"</th>
                                                <th class="p-3 font-semibold border-b">"Golongan"</th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Status"
                                                </th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Gender"
                                                </th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Busana Muslimah"
                                                </th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Baju"
                                                </th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Celana"
                                                </th>
                                                <th class="p-3 font-semibold border-b text-center">
                                                    "Sepatu"
                                                </th>
                                            </tr>
                                        </thead>
                                        <tbody class="text-gray-700 text-sm">
                                            <For
                                                each=move || {
                                                    response.data.clone().into_iter().enumerate()
                                                }
                                                key=|(_, item)| item.nip.clone()
                                                children=move |(idx, item): (
                                                    usize,
                                                    LaporanDaftarPegawai,
                                                )| {
                                                    let status_label = match item
                                                        .jenis
                                                        .as_deref()
                                                    {
                                                        Some("0") => "J",
                                                        Some("1") => "T",
                                                        _ => "-",
                                                    };
                                                    let hijab_label = match item.with_hijab {
                                                        Some(true) => "Y",
                                                        Some(false) => "T",
                                                        None => "-",
                                                    };
                                                    view! {
                                                        <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                            <td class="p-3">
                                                                {((page.get() - 1) * 20 + idx as i32 + 1)
                                                                    .to_string()}
                                                            </td>
                                                            <td class="p-3 font-mono text-sm">{item.nip}</td>
                                                            <td class="p-3 font-medium">{item.nama}</td>
                                                            <td class="p-3">
                                                                {item.jabatan.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center">
                                                                {item.gol_kd.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center">{status_label}</td>
                                                            <td class="p-3 text-center">
                                                                {item
                                                                    .jenis_kelamin
                                                                    .unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center">{hijab_label}</td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item
                                                                    .ukuran_baju
                                                                    .unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item
                                                                    .ukuran_celana
                                                                    .unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-3 text-center font-semibold">
                                                                {item
                                                                    .ukuran_sepatu
                                                                    .unwrap_or_else(|| "-".to_string())}
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
                                                prop:disabled=move || {
                                                    page.get() >= response.total_pages
                                                }
                                                on:click=move |_| set_page.update(|p| *p += 1)
                                            >
                                                "Selanjutnya"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
