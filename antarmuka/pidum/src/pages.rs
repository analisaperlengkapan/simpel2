use leptos::prelude::*;
use crate::api::*;
use crate::types::*;
use chrono::{DateTime, Utc, TimeZone};

// Re-use ActionButton from previous version (simplified)
#[component]
fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let class_str = class.unwrap_or_else(|| {
        "bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded transition duration-200".to_string()
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

// Simple stat card component
#[component]
fn StatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(into)] color: String,
) -> impl IntoView {
    let color_class = format!("text-{} p-3 rounded-full bg-{}-100", color, color);
    view! {
        <div class="bg-white p-6 rounded-lg shadow-sm border border-gray-100 hover:shadow-md transition-shadow">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-500">{title}</p>
                    <p class="text-3xl font-bold text-gray-900 mt-1">{value}</p>
                </div>
                <div class={color_class}>
                    <i class={format!("fas {} text-xl", icon)}></i>
                </div>
            </div>
        </div>
    }
}

/// PIDUM Dashboard Page
#[component]
pub fn PidumDashboard() -> impl IntoView {
    // In a real app, this would be a resource fetching stats from API
    let (stats, _set_stats) = signal(vec![
        (
            "Total Perkara".to_string(),
            "1,248".to_string(),
            "fa-folder".to_string(),
            "blue".to_string(),
        ),
        (
            "SPDP Masuk".to_string(),
            "45".to_string(),
            "fa-file-import".to_string(),
            "yellow".to_string(),
        ),
        (
            "Tahap Penuntutan".to_string(),
            "128".to_string(),
            "fa-gavel".to_string(),
            "red".to_string(),
        ),
        (
            "Eksekusi (P-48)".to_string(),
            "32".to_string(),
            "fa-check-double".to_string(),
            "green".to_string(),
        ),
    ]);

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900">"Dashboard PIDUM"</h1>
                    <p class="text-gray-600">"Sistem Informasi Perkara Tindak Pidana Umum"</p>
                </div>
                <div class="flex space-x-3">
                    <a href="/perkara/create" class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded transition duration-200">
                        "Input SPDP Baru"
                    </a>
                </div>
            </div>

            // Statistics cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {move || stats.get().into_iter().map(|(title, value, icon, color)| view! {
                    <StatCard
                        title=title
                        value=value
                        icon=icon
                        color=color
                    />
                }).collect::<Vec<_>>()}
            </div>

            // Recent Activity / Quick Lists
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-white rounded-lg shadow-sm border border-gray-100">
                    <div class="p-6 border-b border-gray-100 flex justify-between items-center">
                        <h3 class="text-lg font-semibold text-gray-900">"Perkara Terbaru"</h3>
                        <a href="/perkara" class="text-blue-600 hover:text-blue-800 text-sm font-medium">"Lihat Semua"</a>
                    </div>
                    <div class="p-6">
                        <div class="space-y-4">
                            // Placeholder list items
                             <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <div>
                                    <p class="font-medium text-gray-900">"PDM-123/JKT.SLT/02/2024"</p>
                                    <p class="text-sm text-gray-500">"Pencurian dengan Pemberatan"</p>
                                </div>
                                <span class="px-2 py-1 text-xs font-semibold rounded-full bg-yellow-100 text-yellow-800">"SPDP"</span>
                            </div>
                            <div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
                                <div>
                                    <p class="font-medium text-gray-900">"PDM-119/JKT.SLT/02/2024"</p>
                                    <p class="text-sm text-gray-500">"Penganiayaan"</p>
                                </div>
                                <span class="px-2 py-1 text-xs font-semibold rounded-full bg-blue-100 text-blue-800">"P-21"</span>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-sm border border-gray-100">
                     <div class="p-6 border-b border-gray-100">
                        <h3 class="text-lg font-semibold text-gray-900">"Jadwal Sidang Hari Ini"</h3>
                    </div>
                    <div class="p-6">
                         <div class="space-y-4">
                             <div class="flex items-start space-x-3 p-3 bg-gray-50 rounded-lg border-l-4 border-red-500">
                                <div class="flex-1">
                                    <p class="font-medium text-gray-900">"Sidang Pembacaan Dakwaan"</p>
                                    <p class="text-sm text-gray-500">"Terdakwa: Budi Santoso"</p>
                                    <p class="text-xs text-gray-400 mt-1">"09:00 WIB - Ruang Sidang Utama"</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Page: Daftar Perkara (List)
#[component]
pub fn DaftarPerkara() -> impl IntoView {
    // Using a local resource for non-Send futures (WASM)
    let perkara_resource = LocalResource::new(|| fetch_perkara_list());

    view! {
        <div class="space-y-6">
             <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Daftar Perkara"</h1>
                    <p class="text-gray-600">"Manajemen data perkara tindak pidana umum"</p>
                </div>
                <a href="/perkara/create" class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded transition duration-200 flex items-center">
                    <span class="mr-2">"+"</span> "Perkara Baru"
                </a>
            </div>

            <div class="bg-white rounded-lg shadow-sm border border-gray-100 overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"No. Perkara"</th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Kasus"</th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Tanggal"</th>
                                <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            <Suspense fallback=move || view! { <tr><td colspan="5" class="px-6 py-4 text-center">"Memuat data..."</td></tr> }>
                                {move || {
                                    perkara_resource.get().map(|res| {
                                        match res {
                                            Ok(list) => {
                                                if list.is_empty() {
                                                    view! { <tr><td colspan="5" class="px-6 py-4 text-center text-gray-500">"Belum ada data perkara."</td></tr> }.into_any()
                                                } else {
                                                    list.into_iter().map(|p| view! {
                                                        <tr class="hover:bg-gray-50 transition-colors">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-blue-600 hover:underline cursor-pointer">
                                                                {p.nomor_perkara}
                                                            </td>
                                                            <td class="px-6 py-4">
                                                                <div class="text-sm text-gray-900 font-medium">{p.judul}</div>
                                                                <div class="text-sm text-gray-500 truncate max-w-xs">{p.deskripsi.unwrap_or_default()}</div>
                                                            </td>
                                                             <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                                                {p.tanggal_kejadian.format("%d %b %Y").to_string()}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap">
                                                                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                                                                    {p.status}
                                                                </span>
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                <a href="#" class="text-indigo-600 hover:text-indigo-900 mr-3">"Detail"</a>
                                                            </td>
                                                        </tr>
                                                    }).collect_view().into_any()
                                                }
                                            }
                                            Err(e) => view! { <tr><td colspan="5" class="px-6 py-4 text-center text-red-500">{format!("Error: {}", e)}</td></tr> }.into_any()
                                        }
                                    })
                                }}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

/// Page: Input Perkara Baru (Form)
#[component]
pub fn InputPerkara() -> impl IntoView {
    let (nomor, set_nomor) = signal("".to_string());
    let (judul, set_judul) = signal("".to_string());
    let (deskripsi, set_deskripsi) = signal("".to_string());
    let (tanggal, set_tanggal) = signal("".to_string());
    let (status, set_status) = signal("SPDP".to_string()); // Default status

    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (is_submitting, set_is_submitting) = signal(false);

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting.set(true);
        set_error_msg.set(None);
        set_success_msg.set(None);

        // Parse date string (YYYY-MM-DD) to DateTime<Utc>
        // Appending time to make it a full datetime
        let date_str = format!("{}T00:00:00Z", tanggal.get());
        let tanggal_parsed = match DateTime::parse_from_rfc3339(&date_str) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => Utc::now(), // Fallback to now if parsing fails (better validation should be added)
        };

        let req = CreatePerkaraRequest {
            nomor_perkara: nomor.get(),
            judul: judul.get(),
            tanggal_kejadian: tanggal_parsed,
            status: status.get(),
            deskripsi: Some(deskripsi.get()),
        };

        leptos::task::spawn_local(async move {
            match create_perkara(req).await {
                Ok(_) => {
                    set_success_msg.set(Some("Perkara berhasil dibuat!".to_string()));
                    // Clear form or redirect
                    set_nomor.set("".to_string());
                    set_judul.set("".to_string());
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Gagal membuat perkara: {}", e)));
                }
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <div class="max-w-2xl mx-auto">
            <div class="bg-white rounded-lg shadow-sm border border-gray-100 p-8">
                <div class="mb-8 border-b border-gray-100 pb-4">
                    <h1 class="text-2xl font-bold text-gray-900">"Input Perkara Baru"</h1>
                    <p class="text-gray-600 mt-1">"Masukkan data SPDP atau perkara baru"</p>
                </div>

                {move || error_msg.get().map(|msg| view! {
                    <div class="bg-red-50 text-red-700 p-4 rounded-lg mb-6 border border-red-200">
                        {msg}
                    </div>
                })}

                {move || success_msg.get().map(|msg| view! {
                    <div class="bg-green-50 text-green-700 p-4 rounded-lg mb-6 border border-green-200">
                        {msg}
                    </div>
                })}

                <form on:submit=on_submit class="space-y-6">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Nomor Perkara / SPDP"</label>
                        <input
                            type="text"
                            required
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                            placeholder="Contoh: PDM-123/..."
                            on:input=move |ev| set_nomor.set(event_target_value(&ev))
                            prop:value=nomor
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Judul Kasus / Tersangka"</label>
                        <input
                            type="text"
                            required
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                            placeholder="Contoh: Pencurian a.n. Fulan"
                            on:input=move |ev| set_judul.set(event_target_value(&ev))
                            prop:value=judul
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">"Tanggal Kejadian"</label>
                            <input
                                type="date"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                                on:input=move |ev| set_tanggal.set(event_target_value(&ev))
                                prop:value=tanggal
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">"Status Awal"</label>
                            <select
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                                on:change=move |ev| set_status.set(event_target_value(&ev))
                                prop:value=status
                            >
                                <option value="SPDP">"SPDP (Surat Pemberitahuan)"</option>
                                <option value="P-18">"P-18 (Hasil Penyelidikan Belum Lengkap)"</option>
                                <option value="P-19">"P-19 (Pengembalian Berkas)"</option>
                                <option value="P-21">"P-21 (Berkas Lengkap)"</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">"Deskripsi Singkat"</label>
                        <textarea
                            rows="4"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-colors"
                            placeholder="Kronologi singkat atau catatan penting..."
                            on:input=move |ev| set_deskripsi.set(event_target_value(&ev))
                            prop:value=deskripsi
                        ></textarea>
                    </div>

                    <div class="pt-4 flex items-center justify-end space-x-4">
                        <a href="/perkara" class="px-6 py-2 border border-gray-300 text-gray-700 font-medium rounded-lg hover:bg-gray-50 transition-colors">
                            "Batal"
                        </a>
                        <button
                            type="submit"
                            disabled=move || is_submitting.get()
                            class="px-6 py-2 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                        >
                            {move || if is_submitting.get() { "Menyimpan..." } else { "Simpan Perkara" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
