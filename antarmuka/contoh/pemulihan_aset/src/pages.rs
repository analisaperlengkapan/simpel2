use crate::api::*;
use crate::types::*;
use leptos::prelude::*;

// Re-using components from original file
#[component]
fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let class_str = class.unwrap_or_else(|| {
        "bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 rounded".to_string()
    });

    view! {
        <button
            class={class_str}
            on:click=move |_| {
                web_sys::console::log_1(&format!("Action: {action}").into());
            }
        >
            {label}
        </button>
    }
}

#[component]
fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: Signal<String>,
    #[prop(optional)] description: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-white p-6 rounded-lg shadow">
            <h3 class="text-lg font-semibold text-gray-700">{title}</h3>
            <p class="text-3xl font-bold text-red-600">{value}</p>
            {description.map(|desc| view! { <p class="text-sm text-gray-500">{desc}</p> })}
        </div>
    }
}

#[component]
fn SearchBox(
    #[prop(into)] placeholder: String,
    #[prop(optional)] on_search: Option<leptos::callback::Callback<String>>,
) -> impl IntoView {
    let (search_value, set_search_value) = signal("".to_string());

    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-red-500 focus:border-transparent"
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

/// PEMULIHAN ASET Dashboard Page
#[component]
pub fn PemulihanAsetDashboard() -> impl IntoView {
    let (cases, set_cases) = signal::<Option<Result<Vec<Case>, String>>>(None);

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            let res = fetch_cases().await;
            set_cases.set(Some(res));
        });
    });

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900">"Dashboard PEMULIHAN ASET"</h1>
                    <p class="text-gray-600">"Sistem Pemulihan Aset Negara"</p>
                </div>
                <div class="flex space-x-3">
                    <ActionButton
                        label="Identifikasi Aset".to_string()
                        action="new-identification".to_string()
                    />
                </div>
            </div>

            // Statistics cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                 <StatCard
                        title="Total Cases".to_string()
                        value=move || {
                            match cases.get() {
                                Some(Ok(c)) => c.len().to_string(),
                                Some(Err(_)) => "Error".to_string(),
                                None => "...".to_string(),
                            }
                        }
                    />
                <StatCard
                    title="Aset Teridentifikasi".to_string()
                    value=Signal::derive(move || "127".to_string())
                    description="Total aset dalam database".to_string()
                />
                <StatCard
                    title="Dalam Proses".to_string()
                    value=Signal::derive(move || "34".to_string())
                    description="Sedang berjalan".to_string()
                />
                 <StatCard
                    title="Berhasil Dipulihkan".to_string()
                    value=Signal::derive(move || "89".to_string())
                    description="Kasus selesai".to_string()
                />
            </div>

             <div class="bg-white p-6 rounded-lg shadow border">
                <h3 class="text-lg font-semibold mb-4">"Recent Cases"</h3>
                {move || match cases.get() {
                    Some(Ok(c)) => {
                            c.into_iter().map(|item| view! {
                            <div class="border-b py-2">
                                <p class="font-bold">{item.title}</p>
                                <p class="text-sm text-gray-600">{item.status}</p>
                            </div>
                        }).collect_view().into_any()
                    },
                    Some(Err(e)) => view! { <p class="text-red-500">{format!("Error: {}", e)}</p> }.into_any(),
                    None => view! { <p>"Loading..."</p> }.into_any()
                }}
            </div>
        </div>
    }
}

/// PEMULIHAN ASET Identification Page
#[component]
pub fn PemulihanAsetIdentifikasi() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Identifikasi Aset"</h1>
                    <p class="text-gray-600">"Identifikasi dan evaluasi aset negara yang bermasalah"</p>
                </div>
                <ActionButton
                    label="Identifikasi Baru".to_string()
                    action="add-identification".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari aset...".to_string()
            />

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <h3 class="font-semibold text-lg mb-2">"Tanah Negara Jakarta"</h3>
                    <p class="text-gray-600 text-sm mb-4">"Tanah seluas 2.5 Ha di Jakarta Selatan dengan masalah sertifikat"</p>
                    <div class="flex justify-between text-sm text-gray-500">
                        <span>"Nilai: Rp 2.1M"</span>
                        <span class="bg-red-100 text-red-800 px-2 py-1 rounded">"Prioritas Tinggi"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// PEMULIHAN ASET Execution Page
#[component]
pub fn PemulihanAsetEksekusi() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Eksekusi Pemulihan"</h1>
                    <p class="text-gray-600">"Jalankan proses pemulihan aset negara"</p>
                </div>
                <ActionButton
                    label="Eksekusi Baru".to_string()
                    action="add-execution".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari eksekusi...".to_string()
            />

            <div class="bg-white rounded-lg shadow">
                <div class="p-6">
                    <table class="w-full">
                        <thead>
                            <tr class="border-b">
                                <th class="text-left py-3 px-4">"Aset"</th>
                                <th class="text-left py-3 px-4">"Metode"</th>
                                <th class="text-left py-3 px-4">"Status"</th>
                                <th class="text-left py-3 px-4">"Progress"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr class="border-b hover:bg-gray-50">
                                <td class="py-3 px-4">"Tanah Negara Jakarta"</td>
                                <td class="py-3 px-4">"Lelang"</td>
                                <td class="py-3 px-4">
                                    <span class="bg-yellow-100 text-yellow-800 px-2 py-1 rounded text-sm">"Proses"</span>
                                </td>
                                <td class="py-3 px-4">"65%"</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

/// PEMULIHAN ASET Monitoring Page
#[component]
pub fn PemulihanAsetPemantauan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Pemantauan Pemulihan"</h1>
                    <p class="text-gray-600">"Monitor progress dan hasil pemulihan aset"</p>
                </div>
                <ActionButton
                    label="Refresh Data".to_string()
                    action="refresh-monitoring".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari pemantauan...".to_string()
            />

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="bg-white rounded-lg shadow border p-6">
                    <h3 class="font-semibold text-lg mb-4">"Progress Keseluruhan"</h3>
                    <div class="space-y-3">
                        <div class="flex justify-between items-center">
                            <span class="text-sm text-gray-600">"Identifikasi"</span>
                            <span class="text-sm font-medium">"127/150 (85%)"</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-sm text-gray-600">"Eksekusi"</span>
                            <span class="text-sm font-medium">"89/127 (70%)"</span>
                        </div>
                        <div class="flex justify-between items-center">
                            <span class="text-sm text-gray-600">"Selesai"</span>
                            <span class="text-sm font-medium text-green-600">"89 aset"</span>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6">
                    <h3 class="font-semibold text-lg mb-4">"Nilai Pemulihan"</h3>
                    <div class="space-y-2">
                        <p class="text-sm text-gray-600">"Target: Rp 15.2M"</p>
                        <p class="text-sm text-gray-600">"Tercapai: Rp 12.5M"</p>
                        <p class="text-sm text-gray-600">"Persentase: 82.2%"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// PEMULIHAN ASET Reports Page
#[component]
pub fn PemulihanAsetLaporan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Laporan Pemulihan Aset"</h1>
                    <p class="text-gray-600">"Laporan dan dokumentasi hasil pemulihan aset"</p>
                </div>
                <ActionButton
                    label="Generate Laporan".to_string()
                    action="generate-report".to_string()
                />
            </div>

            <SearchBox
                placeholder="Cari laporan...".to_string()
            />

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <h3 class="font-semibold text-lg mb-2">"Laporan Bulanan"</h3>
                    <p class="text-gray-600 text-sm mb-4">"Ringkasan pemulihan aset bulan ini"</p>
                    <div class="text-sm text-gray-500">
                        <span>"Updated: Hari ini"</span>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <h3 class="font-semibold text-lg mb-2">"Analisis Kinerja"</h3>
                    <p class="text-gray-600 text-sm mb-4">"Evaluasi efektivitas proses pemulihan"</p>
                    <div class="text-sm text-gray-500">
                        <span>"Updated: 2 hari lalu"</span>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow border p-6 hover:shadow-lg transition-shadow cursor-pointer">
                    <h3 class="font-semibold text-lg mb-2">"Laporan Tahunan"</h3>
                    <p class="text-gray-600 text-sm mb-4">"Kompilasi hasil pemulihan aset tahun 2023"</p>
                    <div class="text-sm text-gray-500">
                        <span>"Updated: Minggu lalu"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
