use leptos::prelude::*;
use crate::api::{fetch_assets, Aset};

#[component]
pub fn AsetList() -> impl IntoView {
    let (page, set_page) = signal(1);

    // Resource to fetch assets when page changes
    let assets_resource = LocalResource::new(move || {
        let p = page.get();
        async move {
            match fetch_assets(p, 20).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch assets: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Daftar Aset"</h2>
                <div class="flex gap-2">
                    <button class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                        <i class="fas fa-plus mr-2"></i> "Tambah Aset"
                    </button>
                    <button class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors">
                         <i class="fas fa-filter mr-2"></i> "Filter"
                    </button>
                </div>
            </div>

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    assets_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-box-open text-4xl mb-3 text-gray-300"></i>
                                    <p>"Belum ada data aset."</p>
                                    <button class="mt-4 text-blue-600 hover:text-blue-800 text-sm font-medium">
                                        "Tambah Aset Baru"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                            <th class="p-3 font-semibold border-b">"Kode BMN"</th>
                                            <th class="p-3 font-semibold border-b">"Nama Aset"</th>
                                            <th class="p-3 font-semibold border-b">"Kategori"</th>
                                            <th class="p-3 font-semibold border-b">"Merk"</th>
                                            <th class="p-3 font-semibold border-b">"Kondisi"</th>
                                            <th class="p-3 font-semibold border-b">"Lokasi"</th>
                                            <th class="p-3 font-semibold border-b">"Nilai"</th>
                                            <th class="p-3 font-semibold border-b">"Aksi"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="text-gray-700 text-sm">
                                        <For
                                            each=move || response.data.clone()
                                            key=|aset| aset.id.clone()
                                            children=move |aset: Aset| {
                                                view! {
                                                    <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                        <td class="p-3 font-mono text-xs">{aset.kode_bmn}</td>
                                                        <td class="p-3 font-medium">{aset.nama}</td>
                                                        <td class="p-3">
                                                            <span class="px-2 py-1 rounded-full text-xs bg-blue-50 text-blue-600">
                                                                {aset.kategori}
                                                            </span>
                                                        </td>
                                                        <td class="p-3">{aset.merk.unwrap_or("-".to_string())}</td>
                                                        <td class="p-3">
                                                            {
                                                                let color_class = match aset.kondisi.to_lowercase().as_str() {
                                                                    "baik" => "bg-green-100 text-green-700",
                                                                    "rusak ringan" => "bg-yellow-100 text-yellow-700",
                                                                    "rusak berat" => "bg-red-100 text-red-700",
                                                                    _ => "bg-gray-100 text-gray-700",
                                                                };
                                                                view! {
                                                                    <span class={format!("px-2 py-1 rounded-full text-xs capitalize {}", color_class)}>
                                                                        {aset.kondisi}
                                                                    </span>
                                                                }
                                                            }
                                                        </td>
                                                        <td class="p-3">{aset.lokasi}</td>
                                                        <td class="p-3 text-right">
                                                            {
                                                                aset.nilai_perolehan
                                                                    .map(|n| format!("Rp {:.2}", n))
                                                                    .unwrap_or("-".to_string())
                                                            }
                                                        </td>
                                                        <td class="p-3">
                                                            <div class="flex gap-2">
                                                                <button class="text-blue-600 hover:text-blue-800" title="Edit">
                                                                    <i class="fas fa-edit"></i>
                                                                </button>
                                                                <button class="text-red-600 hover:text-red-800" title="Hapus">
                                                                    <i class="fas fa-trash"></i>
                                                                </button>
                                                            </div>
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    </tbody>
                                </table>

                                // Pagination
                                <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                    <div class="text-sm text-gray-500">
                                        "Menampilkan halaman " <span class="font-medium">{response.page}</span> " dari " <span class="font-medium">{response.total_pages}</span>
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || page.get() <= 1
                                            on:click=move |_| set_page.update(|p| *p -= 1)
                                        >
                                            "Sebelumnya"
                                        </button>
                                        <button
                                            class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                            prop:disabled=move || page.get() >= response.total_pages
                                            on:click=move |_| set_page.update(|p| *p += 1)
                                        >
                                            "Selanjutnya"
                                        </button>
                                    </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
