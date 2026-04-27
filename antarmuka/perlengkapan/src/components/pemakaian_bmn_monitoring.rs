//! # Pemakaian BMN Monitoring Dashboard
//!
//! Monitoring dashboard for BMN usage statistics.
//! Requirements: REQ-P011, REQ-P012, REQ-P013

use crate::api::{
    BmnUsageStats, IzinPemakaianBmn, PegawaiUsageStats, fetch_bmn_usage_history,
    fetch_expiring_permits, fetch_pegawai_usage_history,
};
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{MAGNIFYING_GLASS, SPINNER, WARNING};

#[component]
pub fn PemakaianBmnMonitoring() -> impl IntoView {
    // State for search
    let (search_type, set_search_type) = signal("bmn".to_string());
    let (search_query, set_search_query) = signal("".to_string());
    let (search_result, set_search_result) = signal(None::<SearchResult>);
    let (searching, set_searching) = signal(false);

    // Resource for expiring permits
    let expiring_permits = LocalResource::new(|| async move {
        match fetch_expiring_permits(30).await {
            Ok(response) => Some(response.data),
            Err(_) => None,
        }
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
                <h2 class="text-2xl font-bold text-gray-800">"Monitoring Pemakaian BMN"</h2>
                <p class="text-sm text-gray-600 mt-1">"Pantau penggunaan dan riwayat pemakaian BMN"</p>
            </div>

            // Expiring Permits Alert
            <div class="bg-white rounded-lg shadow-sm border border-gray-100 p-6">
                <h3 class="text-lg font-semibold text-gray-800 mb-4 flex items-center gap-2">
                    <span class="text-yellow-500"><AppIcon icon=WARNING /></span>
                    "Izin yang Akan Berakhir (30 Hari)"
                </h3>
                <Suspense fallback=move || view! {
                    <div class="text-center py-4">
                        <span class="fa-spin text-gray-400"><AppIcon icon=SPINNER /></span>
                    </div>
                }>
                    {move || {
                        expiring_permits.get().flatten().map(|permits: Vec<IzinPemakaianBmn>| {
                            if permits.is_empty() {
                                view! {
                                    <p class="text-gray-600 text-center py-4">"Tidak ada izin yang akan berakhir dalam 30 hari"</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="w-full">
                                            <thead class="bg-gray-50 border-b">
                                                <tr>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Nomor Izin"</th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Pemohon"</th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"BMN"</th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Berakhir"</th>
                                                    <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Aksi"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y">
                                                <For
                                                    each=move || permits.clone()
                                                    key=|p| p.id.clone()
                                                    children=move |permit| {
                                                        view! {
                                                            <tr class="hover:bg-gray-50">
                                                                <td class="px-4 py-3 text-sm">{permit.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                                <td class="px-4 py-3 text-sm">{permit.pegawai_nama.clone()}</td>
                                                                <td class="px-4 py-3 text-sm">{permit.bmn_nama_barang.clone()}</td>
                                                                <td class="px-4 py-3 text-sm">{permit.tanggal_selesai.clone()}</td>
                                                                <td class="px-4 py-3 text-sm">
                                                                    <a
                                                                        href={format!("/perlengkapan/pemakaian-bmn/{}", permit.id)}
                                                                        class="text-blue-600 hover:text-blue-800"
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
                                }.into_any()
                            }
                        }).unwrap_or_else(|| view! {
                            <p class="text-gray-600 text-center py-4">"Gagal memuat data"</p>
                        }.into_any())
                    }}
                </Suspense>
            </div>

            // Search Section
            <div class="bg-white rounded-lg shadow-sm border border-gray-100 p-6">
                <h3 class="text-lg font-semibold text-gray-800 mb-4">"Cari Riwayat Pemakaian"</h3>
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
                        placeholder={move || if search_type.get() == "bmn" { "Masukkan NUP BMN" } else { "Masukkan NIP Pegawai" }}
                        prop:value=move || search_query.get()
                        on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    />
                    <button
                        class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                        on:click=handle_search
                        prop:disabled=move || searching.get() || search_query.get().is_empty()
                    >
                        <Show when=move || searching.get() fallback=|| view! { <AppIcon icon=MAGNIFYING_GLASS /> }>
                            <span class="fa-spin"><AppIcon icon=SPINNER /></span>
                        </Show>
                    </button>
                </div>

                // Search Results
                <Show when=move || search_result.get().is_some()>
                    {move || {
                        match search_result.get().unwrap() {
                            SearchResult::Bmn(stats) => view! {
                                <div class="space-y-4">
                                    <div class="grid grid-cols-4 gap-4">
                                        <div class="bg-blue-50 p-4 rounded-lg">
                                            <p class="text-sm text-blue-700">"Total Izin"</p>
                                            <p class="text-2xl font-bold text-blue-900">{stats.total_permits}</p>
                                        </div>
                                        <div class="bg-green-50 p-4 rounded-lg">
                                            <p class="text-sm text-green-700">"Izin Aktif"</p>
                                            <p class="text-2xl font-bold text-green-900">{stats.active_permits}</p>
                                        </div>
                                        <div class="bg-purple-50 p-4 rounded-lg">
                                            <p class="text-sm text-purple-700">"Total Hari Digunakan"</p>
                                            <p class="text-2xl font-bold text-purple-900">{stats.total_days_used}</p>
                                        </div>
                                        <div class="bg-orange-50 p-4 rounded-lg">
                                            <p class="text-sm text-orange-700">"Pemegang Saat Ini"</p>
                                            <p class="text-lg font-bold text-orange-900">{stats.current_holder.clone().unwrap_or_else(|| "-".to_string())}</p>
                                        </div>
                                    </div>

                                    <div>
                                        <h4 class="font-semibold text-gray-800 mb-2">"Riwayat Pemakaian"</h4>
                                        <div class="overflow-x-auto">
                                            <table class="w-full">
                                                <thead class="bg-gray-50 border-b">
                                                    <tr>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Nomor Izin"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Periode"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Status"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Dibuat"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y">
                                                    <For
                                                        each=move || stats.permit_history.clone()
                                                        key=|h| h.id.clone()
                                                        children=move |history| {
                                                            view! {
                                                                <tr class="hover:bg-gray-50">
                                                                    <td class="px-4 py-3 text-sm">{history.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.tanggal_mulai.clone()} " - " {history.tanggal_selesai.clone()}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.status.clone()}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.created_at.clone()}</td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                            SearchResult::Pegawai(stats) => view! {
                                <div class="space-y-4">
                                    <div class="grid grid-cols-3 gap-4">
                                        <div class="bg-blue-50 p-4 rounded-lg">
                                            <p class="text-sm text-blue-700">"Total Izin"</p>
                                            <p class="text-2xl font-bold text-blue-900">{stats.total_permits}</p>
                                        </div>
                                        <div class="bg-green-50 p-4 rounded-lg">
                                            <p class="text-sm text-green-700">"Izin Aktif"</p>
                                            <p class="text-2xl font-bold text-green-900">{stats.active_permits}</p>
                                        </div>
                                        <div class="bg-purple-50 p-4 rounded-lg">
                                            <p class="text-sm text-purple-700">"Pegawai"</p>
                                            <p class="text-lg font-bold text-purple-900">{stats.pegawai_nama.clone()}</p>
                                        </div>
                                    </div>

                                    <div>
                                        <h4 class="font-semibold text-gray-800 mb-2">"Riwayat Pemakaian"</h4>
                                        <div class="overflow-x-auto">
                                            <table class="w-full">
                                                <thead class="bg-gray-50 border-b">
                                                    <tr>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Nomor Izin"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Periode"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Status"</th>
                                                        <th class="px-4 py-2 text-left text-xs font-semibold text-gray-600">"Dibuat"</th>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y">
                                                    <For
                                                        each=move || stats.permit_history.clone()
                                                        key=|h| h.id.clone()
                                                        children=move |history| {
                                                            view! {
                                                                <tr class="hover:bg-gray-50">
                                                                    <td class="px-4 py-3 text-sm">{history.nomor_izin.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.tanggal_mulai.clone()} " - " {history.tanggal_selesai.clone()}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.status.clone()}</td>
                                                                    <td class="px-4 py-3 text-sm">{history.created_at.clone()}</td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </div>
                            }.into_any(),
                            SearchResult::Error(msg) => view! {
                                <div class="p-4 bg-red-50 text-red-700 rounded-lg border border-red-100">
                                    {msg}
                                </div>
                            }.into_any(),
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
