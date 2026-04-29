use crate::api::{Asset, PaginatedResponse, fetch_assets};
use crate::components::pagination_controls::PaginationControls;
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{EYE, PACKAGE};

/// leptos-fetch query keyed by `(page, category)`.
async fn query_aset_page(key: (i32, Option<String>)) -> Option<PaginatedResponse<Asset>> {
    let (page, category) = key;
    fetch_assets(page, 20, category)
        .await
        .map_err(|e| {
            leptos::logging::error!("Failed to fetch assets: {:?}", e);
        })
        .ok()
}

#[component]
pub fn AsetList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (category, set_category) = signal(None::<String>);

    // Per-(page, category) leptos-fetch cache.
    let client: QueryClient = expect_context();
    let assets_resource =
        client.local_resource(query_aset_page, move || (page.get(), category.get()));

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Daftar Aset"</h2>
                <div class="flex gap-2">
                    <select
                        class="px-3 py-2 border rounded-lg bg-white text-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            if val.is_empty() {
                                set_category.set(None);
                            } else {
                                set_category.set(Some(val));
                            }
                            set_page.set(1); // Reset to page 1
                        }
                    >
                        <option value="">"Semua Kategori"</option>
                        <option value="Tanah">"Tanah"</option>
                        <option value="Gedung Bangunan">"Gedung Bangunan"</option>
                        <option value="Alat Besar">"Alat Besar"</option>
                        <option value="Angkutan Bermotor">"Angkutan Bermotor"</option>
                        <option value="Alat Persenjataan">"Alat Persenjataan"</option>
                        <option value="Bangunan Air">"Bangunan Air"</option>
                        <option value="Instalasi Jaringan">"Instalasi Jaringan"</option>
                        <option value="Jalan dan Jembatan">"Jalan dan Jembatan"</option>
                        <option value="KDP">"KDP"</option>
                        <option value="Khusus TIK">"Khusus TIK"</option>
                        <option value="Non TIK">"Non TIK"</option>
                        <option value="Rumah">"Rumah"</option>
                        <option value="Tak Berwujud">"Tak Berwujud"</option>
                        <option value="Tetap Lainnya">"Tetap Lainnya"</option>
                        <option value="Tetap Renovasi">"Tetap Renovasi"</option>
                    </select>
                </div>
            </div>

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    assets_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <span class="text-4xl mb-3 text-gray-300"><AppIcon icon=PACKAGE /></span>
                                    <p>"Belum ada data aset."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                            <th class="p-3 font-semibold border-b">"No Aset"</th>
                                            <th class="p-3 font-semibold border-b">"Kode Barang"</th>
                                            <th class="p-3 font-semibold border-b">"Nama Aset"</th>
                                            <th class="p-3 font-semibold border-b">"Kategori"</th>
                                            <th class="p-3 font-semibold border-b">"Kondisi"</th>
                                            <th class="p-3 font-semibold border-b">"Satker"</th>
                                            <th class="p-3 font-semibold border-b">"Nilai"</th>
                                            <th class="p-3 font-semibold border-b">"Aksi"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="text-gray-700 text-sm">
                                        <For
                                            each=move || response.data.clone()
                                            key=|aset| aset.id.clone()
                                            children=move |aset: Asset| {
                                                view! {
                                                    <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                        <td class="p-3 font-mono text-xs">{aset.no_aset}</td>
                                                        <td class="p-3 font-mono text-xs">{aset.kode_barang.unwrap_or("-".to_string())}</td>
                                                        <td class="p-3 font-medium">{aset.nama_aset.unwrap_or("-".to_string())}</td>
                                                        <td class="p-3">
                                                            <span class="px-2 py-1 rounded-full text-xs bg-blue-50 text-blue-600">
                                                                {aset.kategori_aset}
                                                            </span>
                                                        </td>
                                                        <td class="p-3">
                                                            {
                                                                let kond = aset.kondisi.unwrap_or("-".to_string());
                                                                let color_class = match kond.to_lowercase().as_str() {
                                                                    "baik" => "bg-green-100 text-green-700",
                                                                    "rusak ringan" => "bg-yellow-100 text-yellow-700",
                                                                    "rusak berat" => "bg-red-100 text-red-700",
                                                                    _ => "bg-gray-100 text-gray-700",
                                                                };
                                                                view! {
                                                                    <span class={format!("px-2 py-1 rounded-full text-xs capitalize {}", color_class)}>
                                                                        {kond}
                                                                    </span>
                                                                }
                                                            }
                                                        </td>
                                                        <td class="p-3">{aset.satker.unwrap_or("-".to_string())}</td>
                                                        <td class="p-3 text-right">
                                                            {
                                                                aset.nilai_perolehan
                                                                    .map(|n| format!("Rp {:.2}", n))
                                                                    .unwrap_or("-".to_string())
                                                            }
                                                        </td>
                                                        <td class="p-3">
                                                            <div class="flex gap-2">
                                                                <button class="text-blue-600 hover:text-blue-800" title="Detail">
                                                                    <AppIcon icon=EYE />
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>

                                // Pagination — pass `page` signal (reactive) so the
                                // prev/next button disabled state updates on click.
                                // `total_pages` is captured from the response snapshot;
                                // the component re-renders on each fetch so this stays
                                // in sync with the current data.
                                {
                                    let total_pages = response.total_pages;
                                    view! {
                                        <PaginationControls
                                            current_page=page
                                            total_pages=Signal::derive(move || total_pages)
                                            total_items=None
                                            on_prev=Callback::new(move |_| set_page.update(|p| *p -= 1))
                                            on_next=Callback::new(move |_| set_page.update(|p| *p += 1))
                                        />
                                    }
                                }
                            </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
