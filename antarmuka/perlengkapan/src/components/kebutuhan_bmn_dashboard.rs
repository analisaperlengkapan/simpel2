//! Kebutuhan BMN Dashboard Component
//!
//! Displays overview statistics and quick actions for BMN needs analysis.

use crate::api::{KebutuhanBmnDashboardStats, KebutuhanBmnStatus, fetch_kebutuhan_bmn_dashboard};
use leptos::prelude::*;

#[component]
pub fn KebutuhanBmnDashboard() -> impl IntoView {
    // Fetch dashboard stats
    let stats_resource = LocalResource::new(|| async move {
        match fetch_kebutuhan_bmn_dashboard().await {
            Ok(response) => Some(response.data),
            Err(e) => {
                leptos::logging::error!("Failed to fetch dashboard: {:?}", e);
                None
            }
        }
    });

    view! {
        <div class="space-y-6">
            // Header
            <div class="flex flex-col lg:flex-row items-start lg:items-center justify-between gap-4">
                <div>
                    <h2 class="text-2xl font-bold text-gray-800">"Dashboard Analisis Kebutuhan BMN"</h2>
                    <p class="text-gray-500 mt-1">"Ringkasan data dan statistik pengajuan kebutuhan BMN"</p>
                </div>
                <a
                    href="/dashboard/kebutuhan-bmn/baru"
                    class="px-5 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center shadow-sm"
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Buat Pengajuan Baru"
                </a>
            </div>

            <Suspense fallback=move || view! {
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                    <For
                        each=|| 0..4
                        key=|i| *i
                        children=|_| view! {
                            <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100 animate-pulse">
                                <div class="h-4 bg-gray-200 rounded w-1/2 mb-3"></div>
                                <div class="h-8 bg-gray-200 rounded w-1/3"></div>
                            </div>
                        }
                    />
                </div>
            }>
                {move || {
                    stats_resource.get().flatten().map(|stats| {
                        let stats_store = StoredValue::new(stats.clone());
                        view! {
                            <div class="space-y-6">
                                // Main stats cards
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                                    // Total Pengajuan
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-gray-500">"Total Pengajuan"</p>
                                                <p class="text-3xl font-bold text-gray-800 mt-1">{stats.total_pengajuan}</p>
                                            </div>
                                            <div class="w-12 h-12 rounded-full bg-blue-100 flex items-center justify-center">
                                                <i class="fas fa-file-alt text-blue-600 text-xl"></i>
                                            </div>
                                        </div>
                                    </div>

                                    // Draft
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-gray-500">"Draft"</p>
                                                <p class="text-3xl font-bold text-gray-800 mt-1">{stats.pengajuan_draft}</p>
                                            </div>
                                            <div class="w-12 h-12 rounded-full bg-gray-100 flex items-center justify-center">
                                                <i class="fas fa-edit text-gray-600 text-xl"></i>
                                            </div>
                                        </div>
                                    </div>

                                    // In Progress
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-gray-500">"Dalam Proses"</p>
                                                <p class="text-3xl font-bold text-yellow-600 mt-1">{stats.pengajuan_in_progress}</p>
                                            </div>
                                            <div class="w-12 h-12 rounded-full bg-yellow-100 flex items-center justify-center">
                                                <i class="fas fa-spinner text-yellow-600 text-xl"></i>
                                            </div>
                                        </div>
                                    </div>

                                    // Completed
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-gray-500">"Selesai"</p>
                                                <p class="text-3xl font-bold text-green-600 mt-1">{stats.pengajuan_completed}</p>
                                            </div>
                                            <div class="w-12 h-12 rounded-full bg-green-100 flex items-center justify-center">
                                                <i class="fas fa-check-circle text-green-600 text-xl"></i>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // Secondary stats
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    // Total Satker
                                    <div class="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white shadow-lg">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-blue-100">"Total Satker Terlibat"</p>
                                                <p class="text-4xl font-bold mt-2">{stats.total_satker_terlibat}</p>
                                            </div>
                                            <i class="fas fa-building text-4xl text-blue-200"></i>
                                        </div>
                                    </div>

                                    // Barang Diminta
                                    <div class="bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl p-6 text-white shadow-lg">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-purple-100">"Total Barang Diminta"</p>
                                                <p class="text-4xl font-bold mt-2">{stats.total_barang_diminta}</p>
                                            </div>
                                            <i class="fas fa-boxes text-4xl text-purple-200"></i>
                                        </div>
                                    </div>

                                    // Barang Disetujui
                                    <div class="bg-gradient-to-br from-green-500 to-green-600 rounded-xl p-6 text-white shadow-lg">
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <p class="text-sm font-medium text-green-100">"Total Barang Disetujui"</p>
                                                <p class="text-4xl font-bold mt-2">{stats.total_barang_disetujui}</p>
                                            </div>
                                            <i class="fas fa-check-double text-4xl text-green-200"></i>
                                        </div>
                                    </div>
                                </div>

                                // Charts area
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    // By Year
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                            <i class="fas fa-calendar-alt mr-2 text-blue-600"></i>
                                            "Pengajuan per Tahun"
                                        </h3>
                                        <Show
                                            when=move || !stats.by_tahun.is_empty()
                                            fallback=|| view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <p>"Belum ada data"</p>
                                                </div>
                                            }
                                        >
                                            <div class="space-y-3">
                                                <For
                                                    each=move || stats_store.get_value().by_tahun
                                                    key=|s| s.tahun
                                                    children=move |item| {
                                                        let max_val = stats_store.get_value().by_tahun.iter().map(|s| s.total).max().unwrap_or(1);
                                                        let width_pct = (item.total as f64 / max_val as f64 * 100.0) as i32;
                                                        view! {
                                                            <div class="flex items-center gap-3">
                                                                <span class="w-16 text-sm font-medium text-gray-600">{item.tahun}</span>
                                                                <div class="flex-1 bg-gray-100 rounded-full h-6 overflow-hidden">
                                                                    <div
                                                                        class="bg-blue-500 h-full rounded-full transition-all duration-500"
                                                                        style=format!("width: {}%", width_pct)
                                                                    ></div>
                                                                </div>
                                                                <span class="w-12 text-right text-sm font-semibold text-gray-800">{item.total}</span>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </Show>
                                    </div>

                                    // By Status
                                    <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                        <h3 class="text-lg font-semibold text-gray-800 mb-4">
                                            <i class="fas fa-chart-pie mr-2 text-purple-600"></i>
                                            "Pengajuan per Status"
                                        </h3>
                                        <Show
                                            when=move || !stats.by_status.is_empty()
                                            fallback=|| view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <p>"Belum ada data"</p>
                                                </div>
                                            }
                                        >
                                            <div class="space-y-3">
                                                <For
                                                    each=move || stats_store.get_value().by_status
                                                    key=|s| s.status_kode
                                                    children=move |item| {
                                                        let status = KebutuhanBmnStatus::from_code(item.status_kode);
                                                        let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-gray-100 text-gray-800");
                                                        view! {
                                                            <div class="flex items-center justify-between p-3 rounded-lg bg-gray-50">
                                                                <span class=format!("px-2 py-1 rounded-full text-xs font-medium {}", badge_class)>
                                                                    {item.status_nama.clone()}
                                                                </span>
                                                                <span class="text-lg font-bold text-gray-800">{item.total}</span>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </Show>
                                    </div>
                                </div>

                                // Quick links
                                <div class="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
                                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"Akses Cepat"</h3>
                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                                        <a
                                            href="/dashboard/kebutuhan-bmn"
                                            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:border-blue-500 hover:bg-blue-50 transition-colors"
                                        >
                                            <i class="fas fa-list text-2xl text-blue-600 mb-2"></i>
                                            <span class="text-sm font-medium text-gray-700">"Daftar Pengajuan"</span>
                                        </a>
                                        <a
                                            href="/dashboard/kebutuhan-bmn/baru"
                                            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:border-green-500 hover:bg-green-50 transition-colors"
                                        >
                                            <i class="fas fa-plus-circle text-2xl text-green-600 mb-2"></i>
                                            <span class="text-sm font-medium text-gray-700">"Pengajuan Baru"</span>
                                        </a>
                                        <a
                                            href="/dashboard/analisis"
                                            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:border-purple-500 hover:bg-purple-50 transition-colors"
                                        >
                                            <i class="fas fa-chart-bar text-2xl text-purple-600 mb-2"></i>
                                            <span class="text-sm font-medium text-gray-700">"Laporan Analisis"</span>
                                        </a>
                                        <a
                                            href="/dashboard/aset"
                                            class="flex flex-col items-center p-4 rounded-lg border border-gray-200 hover:border-orange-500 hover:bg-orange-50 transition-colors"
                                        >
                                            <i class="fas fa-box-open text-2xl text-orange-600 mb-2"></i>
                                            <span class="text-sm font-medium text-gray-700">"Bank Aset"</span>
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
