//! # Pemakaian BMN Monitoring Dashboard
//!
//! Monitoring dashboard for BMN usage statistics.
//! Requirements: REQ-P011, REQ-P012, REQ-P013

use crate::api::pemakaian_bmn::{
    MonitoringSummaryCards, PemakaianBmnMonitoringPage, fetch_monitoring_summary,
    fetch_pemakaian_monitoring,
};
use crate::api::{
    BmnUsageStats, IzinPemakaianBmn, PegawaiUsageStats, fetch_bmn_usage_history,
    fetch_expiring_permits, fetch_pegawai_usage_history,
};
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{MAGNIFYING_GLASS, SPINNER, WARNING};

/// Phase 6 pilot — leptos-fetch query for the expiring-permits widget.
///
/// Defined as a free `async fn` (instead of an inline closure inside
/// `LocalResource::new`) so the `QueryClient` can build a `QueryScope`
/// from it. The single tunable is the lookahead window in days,
/// which doubles as the cache key — switching to 60 days here will
/// dedupe with any other call that asks for 60 days while leaving
/// the 30-day cache entry untouched.
async fn query_expiring_permits(days: i32) -> Option<Vec<IzinPemakaianBmn>> {
    fetch_expiring_permits(days)
        .await
        .ok()
        .map(|response| response.data)
}

/// Read-only ringkasan tiga kartu headline (Fase 2.6): sedang dipakai /
/// tidak dipakai / akan expired.
///
/// Tanpa argumen satker: batas visibilitas diturunkan backend dari klaim
/// pemanggil, jadi frontend tidak perlu — dan tidak boleh — memintanya. Kunci
/// unit = satu entri cache global.
async fn query_monitoring_summary(_: ()) -> Option<MonitoringSummaryCards> {
    fetch_monitoring_summary(None, None)
        .await
        .ok()
        .map(|response| response.data)
}

/// Daftar pemakaian BMN yang boleh dilihat pemanggil. Kuncinya `(halaman,
/// pencarian)` sehingga ganti halaman/kata kunci memakai entri cache sendiri.
async fn query_pemakaian_monitoring(key: (i64, String)) -> Option<PemakaianBmnMonitoringPage> {
    let (page, search) = key;
    fetch_pemakaian_monitoring(page, 20, Some(search))
        .await
        .ok()
        .map(|response| response.data)
}

#[component]
pub fn PemakaianBmnMonitoring() -> impl IntoView {
    // State for search
    let (search_type, set_search_type) = signal("bmn".to_string());
    let (search_query, set_search_query) = signal("".to_string());
    let (search_result, set_search_result) = signal(None::<SearchResult>);
    let (searching, set_searching) = signal(false);

    // Phase 6 pilot — keyed cache + dedup for expiring permits via
    // leptos-fetch. The keyer signal returns the lookahead window
    // (30 days). Multiple components asking for the same window
    // share a single in-flight request and a single cached result.
    // Daftar pemakaian: halaman + kata kunci pencarian.
    let (pemakaian_page, set_pemakaian_page) = signal(1i64);
    let (pemakaian_search, set_pemakaian_search) = signal(String::new());

    let client: QueryClient = expect_context();
    let expiring_permits = client.local_resource(query_expiring_permits, || 30);
    let monitoring_summary = client.local_resource(query_monitoring_summary, || ());
    let pemakaian_rows = client.local_resource(query_pemakaian_monitoring, move || {
        (pemakaian_page.get(), pemakaian_search.get())
    });

    // Handle search
    let handle_search = move |_| {
        let query = search_query.get();
        if query.is_empty() {
            return;
        }

        set_searching.set(true);
        set_search_result.set(None);

        let search_type_val = search_type.get();

        leptos::task::spawn_local(async move {
            if search_type_val == "bmn" {
                match fetch_bmn_usage_history(&query).await {
                    Ok(response) => {
                        set_search_result.set(Some(SearchResult::Bmn(response.data)));
                    }
                    Err(_) => {
                        set_search_result.set(Some(SearchResult::Error(
                            "Data tidak ditemukan".to_string(),
                        )));
                    }
                }
            } else {
                match fetch_pegawai_usage_history(&query).await {
                    Ok(response) => {
                        set_search_result.set(Some(SearchResult::Pegawai(response.data)));
                    }
                    Err(_) => {
                        set_search_result.set(Some(SearchResult::Error(
                            "Data tidak ditemukan".to_string(),
                        )));
                    }
                }
            }
            set_searching.set(false);
        });
    };

    view! {
        <div class="p-6 space-y-6">
            <div>
                <h2 class="text-2xl font-bold text-slate-100">"Monitoring Pemakaian BMN"</h2>
                <p class="text-sm text-slate-300 mt-1">
                    "Pantau penggunaan dan riwayat pemakaian BMN"
                </p>
            </div>

            // Kartu ringkasan headline (Fase 2.6, read-only):
            // sedang dipakai / tidak dipakai / akan expired (30 hari)
            <Suspense fallback=move || {
                view! {
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        {(0..3)
                            .map(|_| {
                                view! {
                                    <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6 animate-pulse h-24"></div>
                                }
                            })
                            .collect_view()}
                    </div>
                }
            }>
                {move || {
                    let s = monitoring_summary.get().flatten();
                    let sedang = s
                        .as_ref()
                        .map(|c| c.sedang_dipakai.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let (tidak, tidak_sub) = match s.as_ref().and_then(|c| c.tidak_dipakai) {
                        Some(v) => (v.to_string(), "BMN BAIK tanpa izin aktif".to_string()),
                        None => ("—".to_string(), "Data SIMAN tidak tersedia".to_string()),
                    };
                    let expired = s
                        .as_ref()
                        .map(|c| c.akan_expired_30d.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                                <p class="text-sm text-slate-300">"BMN Sedang Dipakai"</p>
                                <p class="text-3xl font-bold text-emerald-600 mt-1">{sedang}</p>
                                <p class="text-xs text-slate-400 mt-1">"Izin pemakaian aktif"</p>
                            </div>
                            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                                <p class="text-sm text-slate-300">"BMN Tidak Dipakai"</p>
                                <p class="text-3xl font-bold text-info-400 mt-1">{tidak}</p>
                                <p class="text-xs text-slate-400 mt-1">{tidak_sub}</p>
                            </div>
                            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                                <p class="text-sm text-slate-300">"Akan Expired (30 hari)"</p>
                                <p class="text-3xl font-bold text-yellow-600 mt-1">{expired}</p>
                                <p class="text-xs text-slate-400 mt-1">
                                    "Izin berakhir ≤ 30 hari"
                                </p>
                            </div>
                        </div>
                    }
                }}
            </Suspense>

            // Daftar pemakaian BMN — batas per-role diturunkan backend dari
            // klaim: pusat semua, wilayah sebatas wilayahnya, satker sebatas
            // satkernya. Frontend tidak mengirim satker apa pun.
            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                <div class="flex flex-wrap items-center justify-between gap-3 mb-4">
                    <h3 class="text-lg font-semibold text-slate-100">
                        "Daftar Pemakaian BMN"
                    </h3>
                    <input
                        type="text"
                        class="px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none w-full sm:w-80"
                        placeholder="Cari nama barang / NUP / pegawai"
                        prop:value=move || pemakaian_search.get()
                        on:input=move |ev| {
                            set_pemakaian_page.set(1);
                            set_pemakaian_search.set(event_target_value(&ev));
                        }
                    />
                </div>
                <Suspense fallback=move || {
                    view! {
                        <div class="text-center py-4">
                            <span class="fa-spin text-slate-500">
                                <AppIcon icon=SPINNER />
                            </span>
                        </div>
                    }
                }>
                    {move || {
                        match pemakaian_rows.get().flatten() {
                            None => {
                                view! {
                                    <p class="text-slate-300 text-center py-4">"Gagal memuat data"</p>
                                }
                                    .into_any()
                            }
                            Some(page) if page.data.is_empty() => {
                                view! {
                                    <p class="text-slate-300 text-center py-4">
                                        "Tidak ada pemakaian BMN pada cakupan Anda"
                                    </p>
                                }
                                    .into_any()
                            }
                            Some(page) => {
                                let total = page.total;
                                let current = page.page;
                                let total_pages = page.total_pages;
                                let rows = page.data.clone();
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="w-full" data-testid="tabel-pemakaian-bmn">
                                            <thead class="bg-white/[0.02] border-b">
                                                <tr>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Satker"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Nama Barang"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "NUP"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Pegawai Pemakai"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Jangka Waktu"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Status"
                                                    </th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                        "Aksi"
                                                    </th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y">
                                                <For
                                                    each=move || rows.clone()
                                                    key=|r| r.id.clone()
                                                    children=move |row| {
                                                        let satker = row
                                                            .satker_nama
                                                            .clone()
                                                            .or_else(|| row.satker_code.clone())
                                                            .unwrap_or_else(|| "-".to_string());
                                                        let satker_code = row.satker_code.clone();
                                                        // merk/tipe = label bebas operator SIMAN,
                                                        // ditaruh di baris kedua supaya tidak
                                                        // tertukar dengan nama resmi barang.
                                                        let merk_tipe = row.merk_tipe.clone();
                                                        let jabatan = row.pegawai_jabatan.clone();
                                                        let sisa = row.sisa_hari;
                                                        let sisa_label = if sisa < 0 {
                                                            format!("lewat {} hari", -sisa)
                                                        } else {
                                                            format!("sisa {sisa} hari")
                                                        };
                                                        let sisa_class = if sisa < 0 {
                                                            "text-xs text-red-600"
                                                        } else if sisa <= 30 {
                                                            "text-xs text-yellow-600"
                                                        } else {
                                                            "text-xs text-slate-400"
                                                        };
                                                        view! {
                                                            <tr class="hover:bg-white/[0.02]">
                                                                <td class="px-4 py-3 text-sm">
                                                                    <div>{satker}</div>
                                                                    {satker_code
                                                                        .map(|c| {
                                                                            view! {
                                                                                <div class="text-xs text-slate-400">{c}</div>
                                                                            }
                                                                        })}
                                                                </td>
                                                                <td class="px-4 py-3 text-sm">
                                                                    <div>{row.nama_barang.clone()}</div>
                                                                    <div class="text-xs text-slate-400">
                                                                        {row.kode_barang.clone()}
                                                                    </div>
                                                                    {merk_tipe
                                                                        .map(|m| {
                                                                            view! {
                                                                                <div class="text-xs text-slate-500">{m}</div>
                                                                            }
                                                                        })}
                                                                </td>
                                                                <td class="px-4 py-3 text-sm">{row.nup.clone()}</td>
                                                                <td class="px-4 py-3 text-sm">
                                                                    <div>{row.pegawai_nama.clone()}</div>
                                                                    <div class="text-xs text-slate-400">
                                                                        {row.pegawai_nip.clone()}
                                                                    </div>
                                                                    {jabatan
                                                                        .map(|j| {
                                                                            view! {
                                                                                <div class="text-xs text-slate-500">{j}</div>
                                                                            }
                                                                        })}
                                                                </td>
                                                                <td class="px-4 py-3 text-sm">
                                                                    <div>
                                                                        {format!(
                                                                            "{} s.d. {}",
                                                                            row.tanggal_mulai,
                                                                            row.tanggal_selesai,
                                                                        )}
                                                                    </div>
                                                                    <div class="text-xs text-slate-400">
                                                                        {format!("{} hari", row.durasi_hari)}
                                                                    </div>
                                                                    <div class=sisa_class>{sisa_label}</div>
                                                                </td>
                                                                <td class="px-4 py-3 text-sm">{row.status.clone()}</td>
                                                                <td class="px-4 py-3 text-sm">
                                                                    <a
                                                                        href=crate::routes::url::pemakaian_detail(&row.id)
                                                                        class="text-info-400 hover:text-blue-800"
                                                                    >
                                                                        "Detail"
                                                                    </a>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                    <div class="flex items-center justify-between mt-4">
                                        <p class="text-sm text-slate-300">
                                            {format!("{total} pemakaian — halaman {current} dari {}", total_pages.max(1))}
                                        </p>
                                        <div class="flex gap-2">
                                            <button
                                                class="px-3 py-1 border rounded disabled:opacity-40"
                                                prop:disabled=move || current <= 1
                                                on:click=move |_| set_pemakaian_page.update(|p| *p = (*p - 1).max(1))
                                            >
                                                "Sebelumnya"
                                            </button>
                                            <button
                                                class="px-3 py-1 border rounded disabled:opacity-40"
                                                prop:disabled=move || { current >= total_pages }
                                                on:click=move |_| set_pemakaian_page.update(|p| *p += 1)
                                            >
                                                "Berikutnya"
                                            </button>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>

            // Expiring Permits Alert
            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                <h3 class="text-lg font-semibold text-slate-100 mb-4 flex items-center gap-2">
                    <span class="text-yellow-500">
                        <AppIcon icon=WARNING />
                    </span>
                    "Izin yang Akan Berakhir (30 Hari)"
                </h3>
                <Suspense fallback=move || {
                    view! {
                        <div class="text-center py-4">
                            <span class="fa-spin text-slate-500">
                                <AppIcon icon=SPINNER />
                            </span>
                        </div>
                    }
                }>
                    {move || {
                        expiring_permits
                            .get()
                            .flatten()
                            .map(|permits: Vec<IzinPemakaianBmn>| {
                                if permits.is_empty() {
                                    view! {
                                        <p class="text-slate-300 text-center py-4">
                                            "Tidak ada izin yang akan berakhir dalam 30 hari"
                                        </p>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <div class="overflow-x-auto">
                                            <table class="w-full">
                                                <thead class="bg-white/[0.02] border-b">
                                                    <tr>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                            "Nomor Izin"
                                                        </th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                            "Pemohon"
                                                        </th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                            "BMN"
                                                        </th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                            "Berakhir"
                                                        </th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                            "Aksi"
                                                        </th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y">
                                                    <For
                                                        each=move || permits.clone()
                                                        key=|p| p.id.clone()
                                                        children=move |permit| {
                                                            view! {
                                                                <tr class="hover:bg-white/[0.02]">
                                                                    <td class="px-4 py-3 text-sm">
                                                                        {permit
                                                                            .nomor_izin
                                                                            .clone()
                                                                            .unwrap_or_else(|| "-".to_string())}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm">
                                                                        {permit.pegawai_nama.clone()}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm">
                                                                        {permit.bmn_nama_barang.clone()}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm">
                                                                        {permit.tanggal_selesai.clone()}
                                                                    </td>
                                                                    <td class="px-4 py-3 text-sm">
                                                                        <a
                                                                            href=crate::routes::url::pemakaian_detail(&permit.id)
                                                                            class="text-info-400 hover:text-blue-800"
                                                                        >
                                                                            "Detail"
                                                                        </a>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    }
                                        .into_any()
                                }
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <p class="text-slate-300 text-center py-4">
                                        "Gagal memuat data"
                                    </p>
                                }
                                    .into_any()
                            })
                    }}
                </Suspense>
            </div>

            // Search Section
            <div class="rounded-lg border border-white/[0.06] bg-white/[0.04]/[0.04] p-6">
                <h3 class="text-lg font-semibold text-slate-100 mb-4">"Cari Riwayat Pemakaian"</h3>
                <div class="flex gap-4 mb-6">
                    <select
                        class="px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                        prop:value=move || search_type.get()
                        on:change=move |ev| set_search_type.set(event_target_value(&ev))
                    >
                        <option value="bmn">"Berdasarkan NUP BMN"</option>
                        <option value="pegawai">"Berdasarkan NIP Pegawai"</option>
                    </select>
                    <input
                        type="text"
                        class="flex-1 px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                        placeholder=move || {
                            if search_type.get() == "bmn" {
                                "Masukkan NUP BMN"
                            } else {
                                "Masukkan NIP Pegawai"
                            }
                        }
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <button
                        class="px-6 py-2 bg-info-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                        on:click=handle_search
                        prop:disabled=move || searching.get() || search_query.get().is_empty()
                    >
                        <Show
                            when=move || searching.get()
                            fallback=|| view! { <AppIcon icon=MAGNIFYING_GLASS /> }
                        >
                            <span class="fa-spin">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                    </button>
                </div>

                // Search Results
                <Show when=move || {
                    search_result.get().is_some()
                }>
                    {move || {
                        match search_result.get().unwrap() {
                            SearchResult::Bmn(stats) => {
                                view! {
                                    <div class="space-y-4">
                                        <div class="grid grid-cols-4 gap-4">
                                            <div class="bg-blue-50 p-4 rounded-lg">
                                                <p class="text-sm text-blue-700">"Total Izin"</p>
                                                <p class="text-2xl font-bold text-blue-900">
                                                    {stats.total_permits}
                                                </p>
                                            </div>
                                            <div class="bg-green-50 p-4 rounded-lg">
                                                <p class="text-sm text-green-700">"Izin Aktif"</p>
                                                <p class="text-2xl font-bold text-green-900">
                                                    {stats.active_permits}
                                                </p>
                                            </div>
                                            <div class="bg-purple-50 p-4 rounded-lg">
                                                <p class="text-sm text-purple-700">
                                                    "Total Hari Digunakan"
                                                </p>
                                                <p class="text-2xl font-bold text-purple-900">
                                                    {stats.total_days_used}
                                                </p>
                                            </div>
                                            <div class="bg-orange-50 p-4 rounded-lg">
                                                <p class="text-sm text-orange-700">"Pemegang Saat Ini"</p>
                                                <p class="text-lg font-bold text-orange-900">
                                                    {stats
                                                        .current_holder
                                                        .clone()
                                                        .unwrap_or_else(|| "-".to_string())}
                                                </p>
                                            </div>
                                        </div>

                                        <div>
                                            <h4 class="font-semibold text-slate-100 mb-2">
                                                "Riwayat Pemakaian"
                                            </h4>
                                            <div class="overflow-x-auto">
                                                <table class="w-full">
                                                    <thead class="bg-white/[0.02] border-b">
                                                        <tr>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Nomor Izin"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Periode"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Status"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Dibuat"
                                                            </th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y">
                                                        <For
                                                            each=move || stats.permit_history.clone()
                                                            key=|h| h.id.clone()
                                                            children=move |history| {
                                                                view! {
                                                                    <tr class="hover:bg-white/[0.02]">
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history
                                                                                .nomor_izin
                                                                                .clone()
                                                                                .unwrap_or_else(|| "-".to_string())}
                                                                        </td>
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history.tanggal_mulai.clone()} " - "
                                                                            {history.tanggal_selesai.clone()}
                                                                        </td>
                                                                        <td class="px-4 py-3 text-sm">{history.status.clone()}</td>
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history.created_at.clone()}
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                            SearchResult::Pegawai(stats) => {
                                view! {
                                    <div class="space-y-4">
                                        <div class="grid grid-cols-3 gap-4">
                                            <div class="bg-blue-50 p-4 rounded-lg">
                                                <p class="text-sm text-blue-700">"Total Izin"</p>
                                                <p class="text-2xl font-bold text-blue-900">
                                                    {stats.total_permits}
                                                </p>
                                            </div>
                                            <div class="bg-green-50 p-4 rounded-lg">
                                                <p class="text-sm text-green-700">"Izin Aktif"</p>
                                                <p class="text-2xl font-bold text-green-900">
                                                    {stats.active_permits}
                                                </p>
                                            </div>
                                            <div class="bg-purple-50 p-4 rounded-lg">
                                                <p class="text-sm text-purple-700">"Pegawai"</p>
                                                <p class="text-lg font-bold text-purple-900">
                                                    {stats.pegawai_nama.clone()}
                                                </p>
                                            </div>
                                        </div>

                                        <div>
                                            <h4 class="font-semibold text-slate-100 mb-2">
                                                "Riwayat Pemakaian"
                                            </h4>
                                            <div class="overflow-x-auto">
                                                <table class="w-full">
                                                    <thead class="bg-white/[0.02] border-b">
                                                        <tr>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Nomor Izin"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Periode"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Status"
                                                            </th>
                                                            <th class="px-4 py-2 text-left text-xs font-semibold text-slate-300">
                                                                "Dibuat"
                                                            </th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y">
                                                        <For
                                                            each=move || stats.permit_history.clone()
                                                            key=|h| h.id.clone()
                                                            children=move |history| {
                                                                view! {
                                                                    <tr class="hover:bg-white/[0.02]">
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history
                                                                                .nomor_izin
                                                                                .clone()
                                                                                .unwrap_or_else(|| "-".to_string())}
                                                                        </td>
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history.tanggal_mulai.clone()} " - "
                                                                            {history.tanggal_selesai.clone()}
                                                                        </td>
                                                                        <td class="px-4 py-3 text-sm">{history.status.clone()}</td>
                                                                        <td class="px-4 py-3 text-sm">
                                                                            {history.created_at.clone()}
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                            SearchResult::Error(msg) => {
                                view! {
                                    <div class="p-4 bg-red-50 text-red-700 rounded-lg border border-red-100">
                                        {msg}
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}

#[derive(Clone)]
enum SearchResult {
    Bmn(BmnUsageStats),
    Pegawai(PegawaiUsageStats),
    Error(String),
}
