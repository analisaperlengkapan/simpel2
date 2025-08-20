//! Pages untuk DATUN microfrontend

use leptos::prelude::*;
use leptos_router::components::*;
use leptos::callback::Callback;

// Simple button component
#[component]
fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let class_str = class.unwrap_or_else(|| {
        "bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded".to_string()
    });

    view! {
        <button
            class={class_str}
            on:click=move |_| {
                web_sys::console::log_1(&format!("Action: {}", action).into());
            }
        >
            {label}
        </button>
    }
}

// Simple stat card component
#[component]
fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(optional)] description: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-white p-6 rounded-lg shadow">
            <h3 class="text-lg font-semibold text-gray-700">{title}</h3>
            <p class="text-3xl font-bold text-blue-600">{value}</p>
            {description.map(|desc| view! { <p class="text-sm text-gray-500">{desc}</p> })}
        </div>
    }
}

// Simple search box component
#[component]
fn SearchBox(
    #[prop(into)] placeholder: String,
    #[prop(optional)] on_search: Option<Callback<String>>,
) -> impl IntoView {
    let (search_value, set_search_value) = signal("".to_string());

    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder={placeholder}
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = on_search {
                        callback.run(val);
                    }
                }
                prop:value=search_value
            />
        </div>
    }
}
use crate::types::*;

#[component]
pub fn DatunDashboard() -> impl IntoView {
    let (stats, _set_stats) = signal(vec![
        ("Total Pengawasan", "245", "fa-search", "blue"),
        ("Dalam Proses", "18", "fa-clock", "yellow"),
        ("Tindak Lanjut", "67", "fa-tasks", "orange"),
        ("Selesai Bulan Ini", "34", "fa-check-circle", "green"),
    ]);

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900">"Dashboard DATUN"</h1>
                    <p class="text-gray-600">"Pengawasan dan Tindak Lanjut Hasil Pengawasan"</p>
                </div>
                <div class="flex space-x-3">
                    <ActionButton
                        label="Pengawasan Baru".to_string()
                        action="new-pengawasan".to_string()
                    />
                </div>
            </div>

            // Statistics cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {move || stats.get().into_iter().map(|(title, value, _icon, _color)| view! {
                    <StatCard
                        title=title.to_string()
                        value=value.to_string()
                    />
                }).collect::<Vec<_>>()}
            </div>

            // Quick actions
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">"Pengawasan Terbaru"</h3>
                    <div class="space-y-2">
                        <div class="text-sm text-gray-600">"Pengawasan audit keuangan Q3"</div>
                        <div class="text-sm text-gray-600">"Review implementasi kebijakan baru"</div>
                        <div class="text-sm text-gray-600">"Monitoring compliance SOP"</div>
                    </div>
                </div>

                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">"Tindak Lanjut Pending"</h3>
                    <div class="space-y-2">
                        <div class="text-sm text-gray-600">"Perbaikan sistem dokumentasi"</div>
                        <div class="text-sm text-gray-600">"Update prosedur operasional"</div>
                        <div class="text-sm text-gray-600">"Training compliance team"</div>
                    </div>
                </div>

                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-medium text-gray-900 mb-4">"Status Terkini"</h3>
                    <div class="space-y-2">
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"On Track"</span>
                            <span class="text-sm font-medium text-green-600">"78%"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"Delayed"</span>
                            <span class="text-sm font-medium text-orange-600">"15%"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"At Risk"</span>
                            <span class="text-sm font-medium text-red-600">"7%"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DatunPengawasan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Pengawasan"</h1>
                    <p class="text-gray-600">"Kelola dan monitor kegiatan pengawasan"</p>
                </div>
                <ActionButton
                    label="Tambah Pengawasan".to_string()
                    action="add-pengawasan".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari kegiatan pengawasan...".to_string()
                on_search=Callback::new(|_query| {})
            />

            <div class="bg-white rounded-lg shadow border p-6">
                <h3 class="text-lg font-medium mb-4">"Daftar Pengawasan"</h3>
                <div class="space-y-4">
                    <div class="border-b pb-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h4 class="font-medium">"Audit Keuangan Q3 2024"</h4>
                                <p class="text-sm text-gray-600">"Review laporan keuangan triwulan ketiga"</p>
                                <p class="text-xs text-gray-500">"Mulai: 15 Jan 2024 | Target: 28 Jan 2024"</p>
                            </div>
                            <span class="px-3 py-1 rounded-full text-xs bg-yellow-100 text-yellow-800">"Dalam Proses"</span>
                        </div>
                    </div>

                    <div class="border-b pb-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h4 class="font-medium">"Monitoring Compliance SOP"</h4>
                                <p class="text-sm text-gray-600">"Evaluasi kepatuhan terhadap SOP terbaru"</p>
                                <p class="text-xs text-gray-500">"Mulai: 10 Jan 2024 | Target: 25 Jan 2024"</p>
                            </div>
                            <span class="px-3 py-1 rounded-full text-xs bg-green-100 text-green-800">"Selesai"</span>
                        </div>
                    </div>

                    <div class="border-b pb-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h4 class="font-medium">"Review Implementasi Kebijakan"</h4>
                                <p class="text-sm text-gray-600">"Analisis efektivitas kebijakan baru"</p>
                                <p class="text-xs text-gray-500">"Mulai: 8 Jan 2024 | Target: 22 Jan 2024"</p>
                            </div>
                            <span class="px-3 py-1 rounded-full text-xs bg-blue-100 text-blue-800">"Dijadwalkan"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DatunTindakLanjut() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Tindak Lanjut"</h1>
                    <p class="text-gray-600">"Kelola tindak lanjut hasil pengawasan"</p>
                </div>
                <ActionButton
                    label="Tambah Tindak Lanjut".to_string()
                    action="add-tindak-lanjut".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari tindak lanjut...".to_string()
                on_search=Callback::new(|_query| {})
            />

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow border p-6">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-medium">"Pending"</h3>
                        <span class="px-2 py-1 rounded-full text-xs bg-orange-100 text-orange-800">"18"</span>
                    </div>
                    <div class="space-y-3">
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Perbaikan Sistem Dokumentasi"</h4>
                            <p class="text-xs text-gray-600">"Update sistem untuk compliance baru"</p>
                            <p class="text-xs text-gray-500 mt-1">"Deadline: 30 Jan 2024"</p>
                        </div>
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Training Compliance Team"</h4>
                            <p class="text-xs text-gray-600">"Pelatihan untuk tim compliance"</p>
                            <p class="text-xs text-gray-500 mt-1">"Deadline: 2 Feb 2024"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-medium">"Dalam Proses"</h3>
                        <span class="px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800">"12"</span>
                    </div>
                    <div class="space-y-3">
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Update Prosedur Operasional"</h4>
                            <p class="text-xs text-gray-600">"Revisi SOP berdasarkan temuan audit"</p>
                            <p class="text-xs text-gray-500 mt-1">"Progress: 60%"</p>
                        </div>
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Implementasi Control Baru"</h4>
                            <p class="text-xs text-gray-600">"Deploy sistem kontrol internal"</p>
                            <p class="text-xs text-gray-500 mt-1">"Progress: 35%"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-medium">"Selesai"</h3>
                        <span class="px-2 py-1 rounded-full text-xs bg-green-100 text-green-800">"34"</span>
                    </div>
                    <div class="space-y-3">
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Audit Keuangan Q2"</h4>
                            <p class="text-xs text-gray-600">"Tindak lanjut hasil audit Q2 selesai"</p>
                            <p class="text-xs text-gray-500 mt-1">"Selesai: 15 Jan 2024"</p>
                        </div>
                        <div class="p-3 border rounded">
                            <h4 class="font-medium text-sm">"Review Policy Compliance"</h4>
                            <p class="text-xs text-gray-600">"Update kebijakan compliance selesai"</p>
                            <p class="text-xs text-gray-500 mt-1">"Selesai: 12 Jan 2024"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DatunLaporan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Laporan"</h1>
                    <p class="text-gray-600">"Laporan dan analisis pengawasan"</p>
                </div>
                <ActionButton
                    label="Generate Laporan".to_string()
                    action="generate-report".to_string()
                />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-blue-100 text-blue-600 mr-4">
                            <i class="fas fa-chart-line"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Bulanan"</h3>
                            <p class="text-sm text-gray-600">"Ringkasan aktivitas bulanan"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-green-100 text-green-600 mr-4">
                            <i class="fas fa-chart-pie"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Kinerja"</h3>
                            <p class="text-sm text-gray-600">"Analisis kinerja pengawasan"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-purple-100 text-purple-600 mr-4">
                            <i class="fas fa-chart-bar"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Compliance"</h3>
                            <p class="text-sm text-gray-600">"Status kepatuhan dan compliance"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-orange-100 text-orange-600 mr-4">
                            <i class="fas fa-exclamation-triangle"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Risk"</h3>
                            <p class="text-sm text-gray-600">"Identifikasi dan analisis risiko"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-red-100 text-red-600 mr-4">
                            <i class="fas fa-clipboard-check"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Audit"</h3>
                            <p class="text-sm text-gray-600">"Hasil dan temuan audit"</p>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-teal-100 text-teal-600 mr-4">
                            <i class="fas fa-tasks"></i>
                        </div>
                        <div>
                            <h3 class="text-lg font-medium">"Laporan Tindak Lanjut"</h3>
                            <p class="text-sm text-gray-600">"Progress tindak lanjut"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DatunSettings() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-gray-900">"Pengaturan DATUN"</h1>
                <p class="text-gray-600">"Konfigurasi sistem pengawasan dan tindak lanjut"</p>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-white rounded-lg shadow border p-6">
                    <h3 class="text-lg font-medium mb-4">"Pengaturan Umum"</h3>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">"Periode Laporan Default"</label>
                            <select class="w-full border border-gray-300 rounded-md px-3 py-2">
                                <option>"Bulanan"</option>
                                <option>"Triwulanan"</option>
                                <option>"Semester"</option>
                                <option>"Tahunan"</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">"Auto Reminder"</label>
                            <div class="flex items-center">
                                <input type="checkbox" class="mr-2"/>
                                <span class="text-sm">"Aktifkan pengingat otomatis untuk tindak lanjut"</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6">
                    <h3 class="text-lg font-medium mb-4">"Notifikasi"</h3>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <span class="text-sm">"Email untuk pengawasan baru"</span>
                            <input type="checkbox" checked/>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm">"Reminder deadline tindak lanjut"</span>
                            <input type="checkbox" checked/>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm">"Laporan completion status"</span>
                            <input type="checkbox"/>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
