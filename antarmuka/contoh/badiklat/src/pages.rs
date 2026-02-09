//! BADIKLAT Pages
//!
//! Halaman-halaman untuk modul pendidikan dan pelatihan

use leptos::callback::Callback;
use leptos::prelude::*;

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
            <p class="text-3xl font-bold text-green-600">{value}</p>
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
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-green-500 focus:border-transparent"
                placeholder={placeholder}
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = on_search {
                        callback.run(val);
                    }
                }
                value=move || search_value.get()
            />
        </div>
    }
}

// Simple breadcrumb component
#[component]
fn Breadcrumb(#[prop(into)] items: Vec<String>) -> impl IntoView {
    view! {
        <nav class="flex mb-4">
            <ol class="flex items-center space-x-2">
                {items.clone().into_iter().enumerate().map(|(i, item)| {
                    let is_last = i == items.len() - 1;
                    view! {
                        <li>
                            <span class={if is_last { "text-gray-900 font-medium" } else { "text-gray-500" }}>{item.clone()}</span>
                            {!is_last}
                                .then(|| view! { <i class="fas fa-chevron-right mx-2 text-gray-400"></i> })
                        </li>
                    }
                }).collect_view()}
            </ol>
        </nav>
    }
}

/// BADIKLAT Dashboard Page
#[component]
pub fn BadiklatDashboard() -> impl IntoView {
    view! {
        <div class="space-y-8">
            <div>
                <Breadcrumb items=vec!["BADIKLAT".to_string(), "Dashboard".to_string()] />
                <h1 class="text-3xl font-bold text-gray-900 mt-2">"Dashboard BADIKLAT"</h1>
                <p class="text-gray-600">"Selamat datang di sistem pendidikan dan pelatihan"</p>
            </div>

            // Statistics cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatCard
                    title="Total Pelatihan"
                    value="45"
                    description="Program aktif".to_string()
                />
                <StatCard
                    title="Peserta Aktif"
                    value="234"
                    description="Peserta terdaftar".to_string()
                />
                <StatCard
                    title="Instruktur"
                    value="18"
                    description="Instruktur aktif".to_string()
                />
                <StatCard
                    title="Tingkat Kelulusan"
                    value="87%"
                    description="Rata-rata kelulusan".to_string()
                />
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                    <h2 class="text-xl font-semibold text-gray-900 mb-4">
                        <i class="fas fa-clock mr-2"></i>
                        "Pelatihan Terbaru"
                    </h2>
                    <div class="space-y-4">
                        <div class="flex items-center space-x-4 p-3 bg-gray-50 rounded-lg">
                            <div class="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center">
                                <i class="fas fa-chalkboard-teacher text-green-600"></i>
                            </div>
                            <div class="flex-1">
                                <h3 class="font-medium text-gray-900">"Kepemimpinan Dasar"</h3>
                                <p class="text-sm text-gray-500">"25 Agustus 2025"</p>
                            </div>
                            <span class="text-sm font-medium text-green-600">"24/30"</span>
                        </div>
                        <div class="flex items-center space-x-4 p-3 bg-gray-50 rounded-lg">
                            <div class="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center">
                                <i class="fas fa-laptop text-green-600"></i>
                            </div>
                            <div class="flex-1">
                                <h3 class="font-medium text-gray-900">"Sistem Digital"</h3>
                                <p class="text-sm text-gray-500">"28 Agustus 2025"</p>
                            </div>
                            <span class="text-sm font-medium text-green-600">"18/25"</span>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
                    <h2 class="text-xl font-semibold text-gray-900 mb-4">
                        <i class="fas fa-chart-line mr-2"></i>
                        "Statistik Bulanan"
                    </h2>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Pelatihan Selesai"</span>
                            <span class="font-semibold text-gray-900">"8"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Peserta Lulus"</span>
                            <span class="font-semibold text-gray-900">"156"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Sertifikat Diterbitkan"</span>
                            <span class="font-semibold text-gray-900">"142"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Rating Rata-rata"</span>
                            <span class="font-semibold text-gray-900">"4.8/5.0"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn PelatihanPage() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());

    view! {
        <div class="space-y-6">
            <div>
                <Breadcrumb items=vec!["BADIKLAT".to_string(), "Pelatihan".to_string()] />
                <h1 class="text-3xl font-bold text-gray-900">"Manajemen Pelatihan"</h1>
                <p class="text-gray-600">"Kelola program pelatihan dan workshop"</p>
            </div>

            <div class="flex flex-col md:flex-row gap-4">
                <div class="flex-1">
                    <SearchBox
                        placeholder="Cari pelatihan..."
                        on_search=Callback::new(move |query| set_search_query.set(query))
                    />
                </div>
                <button class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700">
                    <i class="fas fa-plus mr-2"></i>
                    "Tambah Pelatihan"
                </button>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-lg shadow border border-gray-200">
                    <div class="flex items-start justify-between mb-4">
                        <div class="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
                            <i class="fas fa-chalkboard-teacher text-green-600"></i>
                        </div>
                        <span class="px-2 py-1 bg-green-100 text-green-800 text-xs font-medium rounded-full">"Aktif"</span>
                    </div>
                    <h3 class="text-lg font-semibold text-gray-900 mb-2">"Kepemimpinan Dasar"</h3>
                    <p class="text-gray-600 mb-4">"Program pelatihan kepemimpinan untuk pegawai tingkat menengah"</p>
                    <div class="space-y-2 text-sm text-gray-500">
                        <div class="flex items-center">
                            <i class="fas fa-calendar-alt w-4 mr-2"></i>
                            <span>"25 - 29 Agustus 2025"</span>
                        </div>
                        <div class="flex items-center">
                            <i class="fas fa-users w-4 mr-2"></i>
                            <span>"24/30 peserta"</span>
                        </div>
                        <div class="flex items-center">
                            <i class="fas fa-user-tie w-4 mr-2"></i>
                            <span>"Dr. Ahmad Rahman"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn PesertaPage() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());

    view! {
        <div class="space-y-6">
            <div>
                <Breadcrumb items=vec!["BADIKLAT".to_string(), "Peserta".to_string()] />
                <h1 class="text-3xl font-bold text-gray-900">"Manajemen Peserta"</h1>
                <p class="text-gray-600">"Kelola data peserta pelatihan"</p>
            </div>

            <SearchBox
                placeholder="Cari peserta berdasarkan nama atau NIP..."
                on_search=Callback::new(move |query| set_search_query.set(query))
            />

            <div class="bg-white rounded-lg shadow">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold text-gray-900">"Data Peserta"</h2>
                </div>
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Nama"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"NIP"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Jabatan"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">"Ahmad Fauzi"</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">"196801011234567890"</td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">"Jaksa Madya"</td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">"Aktif"</span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                    <button class="text-indigo-600 hover:text-indigo-900 mr-4">"Edit"</button>
                                    <button class="text-red-600 hover:text-red-900">"Hapus"</button>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn InstrukturPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <Breadcrumb items=vec!["BADIKLAT".to_string(), "Instruktur".to_string()] />
            <h1 class="text-3xl font-bold text-gray-900">"Manajemen Instruktur"</h1>
            <p class="text-gray-600">"Kelola data instruktur dan pengajar"</p>

            <div class="bg-white p-8 rounded-lg shadow">
                <div class="text-center">
                    <i class="fas fa-user-tie text-6xl text-gray-400 mb-4"></i>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">"Halaman Instruktur"</h3>
                    <p class="text-gray-500">"Fitur manajemen instruktur sedang dalam pengembangan"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn LaporanPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <Breadcrumb items=vec!["BADIKLAT".to_string(), "Laporan".to_string()] />
            <h1 class="text-3xl font-bold text-gray-900">"Laporan Pelatihan"</h1>
            <p class="text-gray-600">"Generate dan unduh laporan pelatihan"</p>

            <div class="bg-white p-8 rounded-lg shadow">
                <div class="text-center">
                    <i class="fas fa-chart-bar text-6xl text-gray-400 mb-4"></i>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">"Halaman Laporan"</h3>
                    <p class="text-gray-500">"Fitur laporan sedang dalam pengembangan"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SertifikatPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <Breadcrumb items=vec!["BADIKLAT".to_string(), "Sertifikat".to_string()] />
            <h1 class="text-3xl font-bold text-gray-900">"Manajemen Sertifikat"</h1>
            <p class="text-gray-600">"Generate dan kelola sertifikat pelatihan"</p>

            <div class="bg-white p-8 rounded-lg shadow">
                <div class="text-center">
                    <i class="fas fa-certificate text-6xl text-gray-400 mb-4"></i>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">"Halaman Sertifikat"</h3>
                    <p class="text-gray-500">"Fitur sertifikat sedang dalam pengembangan"</p>
                </div>
            </div>
        </div>
    }
}
