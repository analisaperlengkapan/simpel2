//! SIMAN Asset Search Component
//!
//! Provides a searchable interface for finding existing BMN/assets
//! from SIMAN (Sistem Informasi Manajemen Aset Negara) integration.

use crate::api::{SimanAsset, search_siman_assets};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Props for SimanAssetSearch component
#[derive(Clone, Default)]
pub struct SimanSearchConfig {
    pub default_kategori: Option<String>,
    pub on_select: Option<Callback<SimanAsset>>,
    pub show_details: bool,
}

/// SIMAN Asset Search Component
///
/// Allows users to search for existing assets from SIMAN inventory.
/// Can be used standalone or embedded in other components like
/// feasibility analysis forms.
#[component]
pub fn SimanAssetSearch(
    #[prop(optional)] default_kategori: Option<String>,
    #[prop(optional)] on_select: Option<Callback<SimanAsset>>,
    #[prop(default = true)] show_details: bool,
) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());
    let (kategori, set_kategori) = signal(default_kategori.clone());
    let (loading, set_loading) = signal(false);
    let (results, set_results) = signal::<Vec<SimanAsset>>(vec![]);
    let (error, set_error) = signal::<Option<String>>(None);
    let (selected_asset, set_selected_asset) = signal::<Option<SimanAsset>>(None);

    // Debounced search
    let do_search = move || {
        let term = search_term.get();
        if term.len() < 2 {
            set_results.set(vec![]);
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let kat = kategori.get();

        spawn_local(async move {
            let kat_ref = kat.as_deref();
            match search_siman_assets(&term, kat_ref, Some(20)).await {
                Ok(response) => {
                    set_results.set(response.data);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal mencari: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // Handle asset selection
    let handle_select = move |asset: SimanAsset| {
        set_selected_asset.set(Some(asset.clone()));
        if let Some(callback) = &on_select {
            callback.run(asset);
        }
    };

    // Asset categories
    let kategori_options = vec![
        ("", "Semua Kategori"),
        ("Khusus TIK", "Peralatan TIK"),
        ("Non TIK", "Peralatan Non-TIK"),
        ("Angkutan Bermotor", "Kendaraan"),
        ("Gedung Bangunan", "Gedung & Bangunan"),
        ("Tanah", "Tanah"),
        ("Tetap Lainnya", "Aset Tetap Lainnya"),
    ];

    view! {
        <div class="bg-white rounded-xl shadow-lg p-6">
            <h3 class="text-lg font-bold text-gray-800 mb-4 flex items-center gap-2">
                <i class="fas fa-database text-blue-500"></i>
                "Cari Aset Eksisting (SIMAN)"
            </h3>

            // Search controls
            <div class="flex flex-col md:flex-row gap-4 mb-6">
                <div class="flex-1">
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Nama Barang / Aset"
                    </label>
                    <div class="relative">
                        <input
                            type="text"
                            class="w-full px-4 py-2 pl-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder="Ketik minimal 2 karakter..."
                            prop:value=move || search_term.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev);
                                set_search_term.set(val);
                            }
                            on:keyup=move |ev| {
                                if event_target_value(&ev).len() >= 2 {
                                    do_search();
                                }
                            }
                        />
                        <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                            <i class="fas fa-search"></i>
                        </span>
                    </div>
                </div>

                <div class="w-full md:w-48">
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Kategori"
                    </label>
                    <select
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_kategori.set(if val.is_empty() { None } else { Some(val) });
                        }
                    >
                        {kategori_options.iter().map(|(value, label)| {
                            view! {
                                <option value=*value>{*label}</option>
                            }
                        }).collect_view()}
                    </select>
                </div>

                <div class="self-end">
                    <button
                        class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                        disabled=move || loading.get() || search_term.get().len() < 2
                        on:click=move |_| do_search()
                    >
                        {move || if loading.get() {
                            view! { <i class="fas fa-spinner fa-spin mr-2"></i>"Mencari..." }.into_any()
                        } else {
                            view! { <i class="fas fa-search mr-2"></i>"Cari" }.into_any()
                        }}
                    </button>
                </div>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
                    <i class="fas fa-exclamation-circle mr-2"></i>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Loading state
            <Show when=move || loading.get()>
                <div class="flex justify-center py-8">
                    <div class="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600"></div>
                </div>
            </Show>

            // Results table
            <Show when=move || !loading.get() && !results.get().is_empty()>
                <div class="overflow-x-auto">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                <th class="p-3 font-semibold border-b">"No. Aset"</th>
                                <th class="p-3 font-semibold border-b">"Nama Aset"</th>
                                <th class="p-3 font-semibold border-b">"Kategori"</th>
                                <th class="p-3 font-semibold border-b text-center">"Kondisi"</th>
                                <Show when=move || show_details>
                                    <th class="p-3 font-semibold border-b">"Lokasi"</th>
                                    <th class="p-3 font-semibold border-b text-right">"Nilai Perolehan"</th>
                                </Show>
                                <Show when=move || on_select.is_some()>
                                    <th class="p-3 font-semibold border-b text-center">"Aksi"</th>
                                </Show>
                            </tr>
                        </thead>
                        <tbody class="text-gray-700 text-sm">
                            <For
                                each=move || results.get()
                                key=|a| a.no_aset.clone()
                                children=move |asset| {
                                    let asset_clone = asset.clone();
                                    let asset_for_select = asset.clone();
                                    let kondisi_class = match asset.kondisi.as_str() {
                                        "Baik" => "bg-green-100 text-green-800",
                                        "Rusak Ringan" => "bg-yellow-100 text-yellow-800",
                                        "Rusak Berat" => "bg-red-100 text-red-800",
                                        _ => "bg-gray-100 text-gray-800",
                                    };

                                    view! {
                                        <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                            <td class="p-3 font-mono text-xs">{asset.no_aset.clone()}</td>
                                            <td class="p-3 font-medium">{asset.nama_aset.clone()}</td>
                                            <td class="p-3 text-gray-600">{asset.kategori.clone()}</td>
                                            <td class="p-3 text-center">
                                                <span class=format!("px-2 py-1 rounded-full text-xs font-medium {}", kondisi_class)>
                                                    {asset.kondisi.clone()}
                                                </span>
                                            </td>
                                            <Show when=move || show_details>
                                                <td class="p-3 text-gray-600">{asset_clone.lokasi.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                <td class="p-3 text-right font-mono">
                                                    {asset_clone.nilai_perolehan.map(|v| format!("Rp {:.0}", v)).unwrap_or_else(|| "-".to_string())}
                                                </td>
                                            </Show>
                                            <Show when=move || on_select.is_some()>
                                                <td class="p-3 text-center">
                                                    <button
                                                        class="px-3 py-1 bg-blue-600 text-white text-xs rounded hover:bg-blue-700 transition-colors"
                                                        on:click={
                                                            let asset_sel = asset_for_select.clone();
                                                            move |_| handle_select(asset_sel.clone())
                                                        }
                                                    >
                                                        "Pilih"
                                                    </button>
                                                </td>
                                            </Show>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>

                <div class="mt-4 text-sm text-gray-500">
                    "Ditemukan " <strong>{move || results.get().len()}</strong> " aset"
                </div>
            </Show>

            // No results
            <Show when=move || {
                let is_not_loading = !loading.get();
                let is_empty = results.get().is_empty();
                let has_search = search_term.get().len() >= 2;
                is_not_loading && is_empty && has_search
            }>
                <div class="text-center py-8 text-gray-500">
                    <i class="fas fa-box-open text-4xl mb-4"></i>
                    <p>"Tidak ditemukan aset yang sesuai"</p>
                    <p class="text-sm mt-2">"Coba kata kunci lain atau ubah kategori"</p>
                </div>
            </Show>

            // Selected asset preview
            <Show when=move || selected_asset.get().is_some()>
                {move || selected_asset.get().map(|asset| view! {
                    <div class="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                        <h4 class="font-semibold text-blue-800 mb-2">
                            <i class="fas fa-check-circle mr-2"></i>
                            "Aset Terpilih"
                        </h4>
                        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div>
                                <span class="text-gray-500">"No. Aset:"</span>
                                <p class="font-mono font-medium">{asset.no_aset.clone()}</p>
                            </div>
                            <div>
                                <span class="text-gray-500">"Nama:"</span>
                                <p class="font-medium">{asset.nama_aset.clone()}</p>
                            </div>
                            <div>
                                <span class="text-gray-500">"Kondisi:"</span>
                                <p class="font-medium">{asset.kondisi.clone()}</p>
                            </div>
                            <div>
                                <span class="text-gray-500">"Kategori:"</span>
                                <p class="font-medium">{asset.kategori.clone()}</p>
                            </div>
                        </div>
                    </div>
                })}
            </Show>
        </div>
    }
}

/// SIMAN Asset Summary Card
///
/// Shows a summary of existing assets for a satker from SIMAN.
#[component]
pub fn SimanAssetSummaryCard(#[prop(into)] satker_id: String) -> impl IntoView {
    let satker_id_clone = satker_id.clone();
    let (loading, set_loading) = signal(true);
    let (summary, set_summary) = signal::<Option<crate::api::SatkerAssetSummary>>(None);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load summary on mount
    Effect::new(move || {
        let id = satker_id_clone.clone();
        spawn_local(async move {
            match crate::api::fetch_siman_satker_summary(&id).await {
                Ok(response) => {
                    set_summary.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memuat ringkasan SIMAN: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    view! {
        <div class="bg-gradient-to-br from-blue-50 to-indigo-50 rounded-xl p-6 border border-blue-200">
            <h4 class="text-lg font-bold text-blue-800 mb-4 flex items-center gap-2">
                <i class="fas fa-chart-pie"></i>
                "Ringkasan Aset (SIMAN)"
            </h4>

            <Show when=move || loading.get()>
                <div class="flex justify-center py-4">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div class="text-red-600 text-sm">
                    <i class="fas fa-exclamation-triangle mr-2"></i>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <Show when=move || !loading.get() && summary.get().is_some()>
                {move || summary.get().map(|s| {
                    let by_category_list = StoredValue::new(s.by_category.clone());
                    view! {
                    <div class="space-y-4">
                        // Total stats
                        <div class="grid grid-cols-2 gap-4">
                            <div class="bg-white rounded-lg p-4 text-center shadow-sm">
                                <p class="text-3xl font-bold text-blue-600">{s.total_assets}</p>
                                <p class="text-sm text-gray-500">"Total Aset"</p>
                            </div>
                            <div class="bg-white rounded-lg p-4 text-center shadow-sm">
                                <p class="text-lg font-bold text-green-600">
                                    {format!("Rp {:.0}", s.total_value)}
                                </p>
                                <p class="text-sm text-gray-500">"Total Nilai"</p>
                            </div>
                        </div>

                        // By category
                        <Show when=move || !s.by_category.is_empty()>
                            <div>
                                <h5 class="font-semibold text-gray-700 mb-2">"Per Kategori"</h5>
                                <div class="space-y-2">
                                    <For
                                        each=move || by_category_list.get_value()
                                        key=|c| c.category.clone()
                                        children=|cat| {
                                            view! {
                                                <div class="flex justify-between items-center text-sm bg-white rounded px-3 py-2">
                                                    <span class="text-gray-600">{cat.category}</span>
                                                    <span class="font-semibold text-gray-800">{cat.count}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        </Show>
                    </div>
                }})}
            </Show>
        </div>
    }
}
