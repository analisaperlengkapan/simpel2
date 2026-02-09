//! Kebutuhan BMN Satker Detail Component
//!
//! Displays satker-specific view with goods list, workflow, and analysis.

use crate::api::{
    AnalisisKelayakanResponse, CreateKebutuhanBmnBarangRequest, KebutuhanBmnStatus,
    PengajuanKebutuhanBmnAktivitas, PengajuanKebutuhanBmnBarang, SatkerWithBarangResponse,
    UpdateBarangApprovalRequest, WorkflowTransitionRequest, create_kebutuhan_bmn_barang,
    delete_kebutuhan_bmn_barang, fetch_satker_aktivitas, fetch_satker_analisis,
    fetch_satker_with_barang, transition_satker_status, update_kebutuhan_bmn_barang,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

#[component]
pub fn KebutuhanBmnSatkerDetail() -> impl IntoView {
    let params = use_params_map();
    let satker_id = Memo::new(move |_| params.read().get("satker_id").clone().unwrap_or_default());

    // Data state
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);
    let (satker_data, set_satker_data) = signal::<Option<SatkerWithBarangResponse>>(None);
    let (aktivitas, set_aktivitas) = signal::<Vec<PengajuanKebutuhanBmnAktivitas>>(vec![]);
    let (analisis, set_analisis) = signal::<Option<AnalisisKelayakanResponse>>(None);

    // UI state
    let (active_tab, set_active_tab) = signal("barang");
    let (show_add_barang, set_show_add_barang) = signal(false);
    let (submitting, set_submitting) = signal(false);

    // New barang form state
    let (new_nama, set_new_nama) = signal(String::new());
    let (new_kode_barang, set_new_kode_barang) = signal::<Option<String>>(None);
    let (new_jumlah, set_new_jumlah) = signal(1i32);
    let (new_satuan, set_new_satuan) = signal("Unit".to_string());
    let (new_alasan, set_new_alasan) = signal::<Option<String>>(None);

    // Load satker data
    let load_data = move |sid: String| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match fetch_satker_with_barang(&sid).await {
                Ok(response) => {
                    set_satker_data.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memuat data: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // Load aktivitas
    let load_aktivitas = move |sid: String| {
        spawn_local(async move {
            if let Ok(response) = fetch_satker_aktivitas(&sid).await {
                set_aktivitas.set(response.data);
            }
        });
    };

    // Load analisis
    let load_analisis = move |sid: String| {
        spawn_local(async move {
            if let Ok(response) = fetch_satker_analisis(&sid).await {
                set_analisis.set(Some(response.data));
            }
        });
    };

    // Initial load
    Effect::new(move || {
        let sid = satker_id.get();
        if !sid.is_empty() {
            load_data(sid.clone());
            load_aktivitas(sid.clone());
            load_analisis(sid);
        }
    });

    // Handle add barang
    let handle_add_barang = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let sid = satker_id.get();
        let request = CreateKebutuhanBmnBarangRequest {
            nama: new_nama.get(),
            kode_barang: new_kode_barang.get(),
            jumlah: new_jumlah.get(),
            satuan: new_satuan.get(),
            alasan: new_alasan.get(),
            keterangan: None,
            file_pendukung: vec![],
        };

        spawn_local(async move {
            match create_kebutuhan_bmn_barang(&sid, request).await {
                Ok(_) => {
                    // Reset form
                    set_new_nama.set(String::new());
                    set_new_kode_barang.set(None);
                    set_new_jumlah.set(1);
                    set_new_alasan.set(None);
                    set_show_add_barang.set(false);
                    // Reload data
                    load_data(sid);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menambah barang: {:?}", e)));
                }
            }
            set_submitting.set(false);
        });
    };

    // Handle delete barang
    let handle_delete_barang = move |barang_id: String| {
        let sid = satker_id.get();
        spawn_local(async move {
            match delete_kebutuhan_bmn_barang(&barang_id).await {
                Ok(_) => {
                    load_data(sid);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menghapus barang: {:?}", e)));
                }
            }
        });
    };

    // Handle transition
    let handle_transition = move |target_status: i32| {
        let sid = satker_id.get();
        spawn_local(async move {
            let request = WorkflowTransitionRequest {
                target_status,
                komentar: None,
            };
            match transition_satker_status(&sid, request).await {
                Ok(_) => {
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal transisi: {:?}", e)));
                }
            }
        });
    };

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            // Back link
            <div class="mb-6">
                <button
                    onclick="history.back()"
                    class="text-blue-600 hover:text-blue-800 inline-flex items-center"
                >
                    <i class="fas fa-arrow-left mr-2"></i>
                    "Kembali"
                </button>
            </div>

            // Loading
            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-3"></div>
                    <p class="text-gray-500">"Memuat data..."</p>
                </div>
            </Show>

            // Error
            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
                    <i class="fas fa-exclamation-circle mr-2"></i>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Content
            <Show when=move || !loading.get() && satker_data.get().is_some()>
                {move || {
                    if let Some(data) = satker_data.get() {
                        let satker = data.satker.clone();
                        let barang_list = data.barang_list.clone();
                        let has_barang = !barang_list.is_empty();
                        let barang_list_store = StoredValue::new(barang_list);
                        let status = KebutuhanBmnStatus::from_code(satker.status_kode);
                        let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-gray-100 text-gray-800");
                        let status_label = status.map(|s| s.label()).unwrap_or("Unknown");

                        view! {
                            <div class="space-y-6">
                                // Header
                                <div class="flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4">
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-800">
                                            {satker.nm_satker.clone().unwrap_or_else(|| satker.ms_satker_id.clone())}
                                        </h2>
                                        <p class="text-gray-500 mt-1">
                                            "Kode Satker: " <span class="font-mono">{satker.ms_satker_id.clone()}</span>
                                        </p>
                                    </div>
                                    <span class=format!("px-3 py-1.5 rounded-full text-sm font-medium {}", badge_class)>
                                        {status_label}
                                    </span>
                                </div>

                                // Stats cards
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    <div class="bg-blue-50 rounded-lg p-4">
                                        <div class="text-blue-600 text-sm font-medium">"Total Barang"</div>
                                        <div class="text-2xl font-bold text-gray-800 mt-1">{data.total_barang}</div>
                                    </div>
                                    <div class="bg-purple-50 rounded-lg p-4">
                                        <div class="text-purple-600 text-sm font-medium">"Total Jumlah Diminta"</div>
                                        <div class="text-2xl font-bold text-gray-800 mt-1">{data.total_jumlah}</div>
                                    </div>
                                    <div class="bg-green-50 rounded-lg p-4">
                                        <div class="text-green-600 text-sm font-medium">"Prioritas"</div>
                                        <div class="text-2xl font-bold text-gray-800 mt-1">{satker.prioritas}</div>
                                    </div>
                                </div>

                                // Tabs
                                <div class="border-b border-gray-200">
                                    <nav class="flex gap-4">
                                        <button
                                            class=move || format!(
                                                "py-2 px-4 border-b-2 font-medium text-sm transition-colors {}",
                                                if active_tab.get() == "barang" { "border-blue-600 text-blue-600" } else { "border-transparent text-gray-500 hover:text-gray-700" }
                                            )
                                            on:click=move |_| set_active_tab.set("barang")
                                        >
                                            <i class="fas fa-boxes mr-2"></i>
                                            "Daftar Barang"
                                        </button>
                                        <button
                                            class=move || format!(
                                                "py-2 px-4 border-b-2 font-medium text-sm transition-colors {}",
                                                if active_tab.get() == "analisis" { "border-blue-600 text-blue-600" } else { "border-transparent text-gray-500 hover:text-gray-700" }
                                            )
                                            on:click=move |_| set_active_tab.set("analisis")
                                        >
                                            <i class="fas fa-chart-bar mr-2"></i>
                                            "Analisis Kelayakan"
                                        </button>
                                        <button
                                            class=move || format!(
                                                "py-2 px-4 border-b-2 font-medium text-sm transition-colors {}",
                                                if active_tab.get() == "aktivitas" { "border-blue-600 text-blue-600" } else { "border-transparent text-gray-500 hover:text-gray-700" }
                                            )
                                            on:click=move |_| set_active_tab.set("aktivitas")
                                        >
                                            <i class="fas fa-history mr-2"></i>
                                            "Riwayat Aktivitas"
                                        </button>
                                    </nav>
                                </div>

                                // Tab content: Barang
                                <Show when=move || active_tab.get() == "barang">
                                    <div>
                                        <div class="flex justify-between items-center mb-4">
                                            <h3 class="text-lg font-semibold">"Daftar Barang"</h3>
                                            <button
                                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                                on:click=move |_| set_show_add_barang.set(true)
                                            >
                                                <i class="fas fa-plus mr-2"></i>
                                                "Tambah Barang"
                                            </button>
                                        </div>

                                        <Show
                                            when=move || has_barang
                                            fallback=|| view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <i class="fas fa-box-open text-4xl mb-3 text-gray-300"></i>
                                                    <p>"Belum ada barang yang ditambahkan"</p>
                                                </div>
                                            }
                                        >
                                            <div class="overflow-x-auto">
                                                <table class="w-full text-left border-collapse">
                                                    <thead>
                                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                            <th class="p-3 font-semibold border-b">"Nama Barang"</th>
                                                            <th class="p-3 font-semibold border-b">"Kode"</th>
                                                            <th class="p-3 font-semibold border-b text-center">"Jumlah"</th>
                                                            <th class="p-3 font-semibold border-b text-center">"Disetujui"</th>
                                                            <th class="p-3 font-semibold border-b text-center">"Prioritas"</th>
                                                            <th class="p-3 font-semibold border-b">"Alasan"</th>
                                                            <th class="p-3 font-semibold border-b text-center">"Aksi"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="text-gray-700 text-sm">
                                                        <For
                                                            each=move || barang_list_store.get_value().into_iter()
                                                            key=|b| b.id.clone()
                                                            children=move |barang: PengajuanKebutuhanBmnBarang| {
                                                                let barang_id = barang.id.clone();
                                                                view! {
                                                                    <tr class="hover:bg-gray-50 border-b last:border-0">
                                                                        <td class="p-3 font-medium">{barang.nama.clone()}</td>
                                                                        <td class="p-3 font-mono text-xs">{barang.kode_barang.clone().unwrap_or("-".into())}</td>
                                                                        <td class="p-3 text-center">{format!("{} {}", barang.jumlah, barang.satuan)}</td>
                                                                        <td class="p-3 text-center text-green-600 font-medium">{barang.jml_setuju}</td>
                                                                        <td class="p-3 text-center">
                                                                            <span class="bg-blue-100 text-blue-800 px-2 py-0.5 rounded text-xs">
                                                                                {barang.prioritas}
                                                                            </span>
                                                                        </td>
                                                                        <td class="p-3 text-gray-600 max-w-xs truncate">{barang.alasan.clone().unwrap_or("-".into())}</td>
                                                                        <td class="p-3 text-center">
                                                                            <button
                                                                                class="text-red-600 hover:text-red-800"
                                                                                on:click=move |_| handle_delete_barang(barang_id.clone())
                                                                            >
                                                                                <i class="fas fa-trash"></i>
                                                                            </button>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </div>
                                        </Show>
                                    </div>
                                </Show>

                                // Tab content: Analisis
                                <Show when=move || active_tab.get() == "analisis">
                                    <div>
                                        {move || {
                                            analisis.get().map(|a| {
                                                view! {
                                                    <div class="space-y-4">
                                                        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                                                            <div class="bg-blue-50 rounded-lg p-4">
                                                                <div class="text-blue-600 text-sm">"Total Diminta"</div>
                                                                <div class="text-2xl font-bold">{a.summary.total_diminta}</div>
                                                            </div>
                                                            <div class="bg-green-50 rounded-lg p-4">
                                                                <div class="text-green-600 text-sm">"Total Existing"</div>
                                                                <div class="text-2xl font-bold">{a.summary.total_existing}</div>
                                                            </div>
                                                            <div class="bg-orange-50 rounded-lg p-4">
                                                                <div class="text-orange-600 text-sm">"Gap Kebutuhan"</div>
                                                                <div class="text-2xl font-bold">{a.summary.total_gap}</div>
                                                            </div>
                                                            <div class="bg-purple-50 rounded-lg p-4">
                                                                <div class="text-purple-600 text-sm">"% Kelayakan"</div>
                                                                <div class="text-2xl font-bold">{format!("{:.1}%", a.summary.kelayakan_persen)}</div>
                                                            </div>
                                                        </div>

                                                        // Gap analysis table with existing assets from SIMAN
                                                        <div class="mt-6">
                                                            <h4 class="font-semibold text-gray-800 mb-3 flex items-center gap-2">
                                                                <i class="fas fa-chart-bar text-blue-500"></i>
                                                                "Detail Analisis per Barang"
                                                            </h4>
                                                            <div class="overflow-x-auto">
                                                                <table class="w-full text-left border-collapse">
                                                                    <thead>
                                                                        <tr class="bg-gray-50 text-gray-600 text-xs uppercase">
                                                                            <th class="p-3 font-semibold border-b">"Nama Barang"</th>
                                                                            <th class="p-3 font-semibold border-b text-center">"Diminta"</th>
                                                                            <th class="p-3 font-semibold border-b text-center">"Existing (SIMAN)"</th>
                                                                            <th class="p-3 font-semibold border-b text-center">"Gap"</th>
                                                                            <th class="p-3 font-semibold border-b">"Rekomendasi"</th>
                                                                        </tr>
                                                                    </thead>
                                                                    <tbody class="text-gray-700 text-sm">
                                                                        <For
                                                                            each=move || a.barang_list.clone()
                                                                            key=|b| b.barang.id.clone()
                                                                            children=move |item| {
                                                                let item_store = StoredValue::new(item.clone());
                                                                                let gap_class = if item.gap <= 0 {
                                                                                    "text-green-600"
                                                                                } else if item.gap < item.barang.jumlah / 2 {
                                                                                    "text-yellow-600"
                                                                                } else {
                                                                                    "text-red-600"
                                                                                };
                                                                                let existing_count = item.existing_assets.len();

                                                                                view! {
                                                                                    <tr class="hover:bg-gray-50 border-b last:border-0">
                                                                                        <td class="p-3">
                                                                                            <div class="font-medium">{item.barang.nama.clone()}</div>
                                                            {
                                                                (existing_count > 0).then(|| {
                                                                    view! {
                                                                        <div class="text-xs text-gray-400 mt-1">
                                                                            <i class="fas fa-database mr-1"></i>
                                                                            {format!("{} aset ditemukan di SIMAN", existing_count)}
                                                                        </div>
                                                                    }
                                                                })
                                                            }
                                                                                        </td>
                                                                                        <td class="p-3 text-center font-medium">{item.barang.jumlah}</td>
                                                                                        <td class="p-3 text-center">
                                                                                            <span class="text-blue-600 font-medium">{existing_count as i32}</span>
                                                                                        </td>
                                                                                        <td class=format!("p-3 text-center font-bold {}", gap_class)>
                                                                                            {if item.gap > 0 { format!("+{}", item.gap) } else { item.gap.to_string() }}
                                                                                        </td>
                                                                                        <td class="p-3 text-sm">{item.recommendation.clone()}</td>
                                                                                    </tr>
                                                                                    // Show existing assets if any
                                                                    <Show when=move || !item_store.with_value(|i| i.existing_assets.is_empty())>
                                                                                        <tr class="bg-blue-50">
                                                                                            <td colspan="5" class="p-2">
                                                                                                <details class="cursor-pointer">
                                                                                                    <summary class="text-xs text-blue-600 font-medium">
                                                                                                        <i class="fas fa-list-ul mr-1"></i>
                                                                                                        "Lihat aset existing dari SIMAN"
                                                                                                    </summary>
                                                                                                    <div class="mt-2 space-y-1 max-h-40 overflow-y-auto">
                                                                                                        <For
                                                                                            each=move || item_store.get_value().existing_assets
                                                                                                            key=|ea| ea.no_aset.clone()
                                                                                                            children=move |ea| {
                                                                                                                let kondisi_class = match ea.kondisi.as_str() {
                                                                                                                    "Baik" => "bg-green-100 text-green-800",
                                                                                                                    "Rusak Ringan" => "bg-yellow-100 text-yellow-800",
                                                                                                                    _ => "bg-red-100 text-red-800"
                                                                                                                };
                                                                                                                view! {
                                                                                                                    <div class="flex justify-between items-center text-xs bg-white rounded px-2 py-1">
                                                                                                                        <div>
                                                                                                                            <span class="font-mono text-gray-500">{ea.no_aset.clone()}</span>
                                                                                                                            " - "
                                                                                                                            <span class="font-medium">{ea.nama_aset.clone()}</span>
                                                                                                                        </div>
                                                                                                                        <span class=format!("px-2 py-0.5 rounded text-xs {}", kondisi_class)>
                                                                                                                            {ea.kondisi.clone()}
                                                                                                                        </span>
                                                                                                                    </div>
                                                                                                                }
                                                                                                            }
                                                                                                        />
                                                                                                    </div>
                                                                                                </details>
                                                                                            </td>
                                                                                        </tr>
                                                                                    </Show>
                                                                                }
                                                                            }
                                                                        />
                                                                    </tbody>
                                                                </table>
                                                            </div>
                                                        </div>

                                                        // SIMAN integration note
                                                        <div class="mt-4 p-3 bg-blue-50 rounded-lg text-sm text-blue-700">
                                                            <i class="fas fa-info-circle mr-2"></i>
                                                            "Data aset existing diambil dari SIMAN (Sistem Informasi Manajemen Aset Negara). "
                                                            "Gap dihitung berdasarkan jumlah yang diminta dikurangi jumlah aset sejenis yang sudah ada."
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            }).unwrap_or_else(|| view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <p>"Data analisis belum tersedia"</p>
                                                </div>
                                            }.into_any())
                                        }}
                                    </div>
                                </Show>

                                // Tab content: Aktivitas
                                <Show when=move || active_tab.get() == "aktivitas">
                                    <div>
                                        <Show
                                            when=move || !aktivitas.get().is_empty()
                                            fallback=|| view! {
                                                <div class="text-center py-8 text-gray-500">
                                                    <p>"Belum ada aktivitas"</p>
                                                </div>
                                            }
                                        >
                                            <div class="space-y-3">
                                                <For
                                                    each=move || aktivitas.get()
                                                    key=|a| a.id.clone()
                                                    children=move |akt| {
                                                        let to_status = KebutuhanBmnStatus::from_code(akt.to_status_kode);
                                                        view! {
                                                            <div class="flex items-start gap-4 p-4 bg-gray-50 rounded-lg">
                                                                <div class="w-10 h-10 rounded-full bg-blue-100 flex items-center justify-center">
                                                                    <i class="fas fa-arrow-right text-blue-600"></i>
                                                                </div>
                                                                <div class="flex-1">
                                                                    <div class="font-medium text-gray-800">{akt.aksi.clone()}</div>
                                                                    <div class="text-sm text-gray-500 mt-1">
                                                                        "Ke status: " {to_status.map(|s| s.label()).unwrap_or("Unknown")}
                                                                    </div>
                                                                    {
                                                                        let has_komentar = akt.komentar.is_some();
                                                                        view! {
                                                                            <Show when=move || has_komentar>
                                                                                <div class="text-sm text-gray-600 mt-2 italic">
                                                                                    "\""{ akt.komentar.clone().unwrap_or_default() }"\""
                                                                                </div>
                                                                            </Show>
                                                                        }
                                                                    }
                                                                    <div class="text-xs text-gray-400 mt-2">
                                                                        {format!("{} - {}", akt.nama.clone().unwrap_or("System".into()), akt.created_at)}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </Show>
                                    </div>
                                </Show>
                            </div>
                        }.into_any()
                    } else {
                         view! {
                            <div class="text-center py-8 text-gray-500">
                                <p>"Data analisis belum tersedia"</p>
                            </div>
                        }.into_any()
                    }
                }}
            </Show>

            // Add barang modal
            <Show when=move || show_add_barang.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white rounded-xl p-6 max-w-lg w-full mx-4 shadow-xl">
                        <h3 class="text-lg font-bold text-gray-800 mb-4">"Tambah Barang"</h3>
                        <form on:submit=handle_add_barang class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Barang *"</label>
                                <input
                                    type="text"
                                    required
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                    on:input=move |ev| set_new_nama.set(event_target_value(&ev))
                                    prop:value=move || new_nama.get()
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">"Kode Barang"</label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        set_new_kode_barang.set(if v.is_empty() { None } else { Some(v) });
                                    }
                                />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Jumlah *"</label>
                                    <input
                                        type="number"
                                        min="1"
                                        required
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse() {
                                                set_new_jumlah.set(v);
                                            }
                                        }
                                        prop:value=move || new_jumlah.get()
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Satuan"</label>
                                    <input
                                        type="text"
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                        on:input=move |ev| set_new_satuan.set(event_target_value(&ev))
                                        prop:value=move || new_satuan.get()
                                    />
                                </div>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">"Alasan/Justifikasi"</label>
                                <textarea
                                    rows="2"
                                    class="w-full px-4 py-2 border border-gray-300 rounded-lg"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        set_new_alasan.set(if v.is_empty() { None } else { Some(v) });
                                    }
                                ></textarea>
                            </div>
                            <div class="flex justify-end gap-3 pt-4 border-t">
                                <button
                                    type="button"
                                    class="px-4 py-2 border border-gray-300 rounded-lg"
                                    on:click=move |_| set_show_add_barang.set(false)
                                >
                                    "Batal"
                                </button>
                                <button
                                    type="submit"
                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                    disabled=move || submitting.get()
                                >
                                    {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </div>
    }
}
