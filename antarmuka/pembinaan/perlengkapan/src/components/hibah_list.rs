use crate::api::{Hibah, fetch_hibah};
use leptos::prelude::*;

#[component]
pub fn HibahList() -> impl IntoView {
    let (page, set_page) = signal(1);

    // Resource to fetch items when page changes
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        async move {
            match fetch_hibah(p, 20).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch hibah: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Daftar Hibah BMN"</h2>
                <a
                    href="/dashboard/pengelolaan/hibah/baru"
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Catat Hibah"
                </a>
            </div>

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-clipboard-list text-4xl mb-3 text-gray-300"></i>
                                    <p>"Belum ada data hibah."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                            <th class="p-3 font-semibold border-b">"Pemberi"</th>
                                            <th class="p-3 font-semibold border-b">"Penerima"</th>
                                            <th class="p-3 font-semibold border-b">"Tanggal"</th>
                                            <th class="p-3 font-semibold border-b">"Keterangan"</th>
                                            <th class="p-3 font-semibold border-b">"Aksi"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="text-gray-700 text-sm">
                                        <For
                                            each=move || response.data.clone()
                                            key=|item| item.id.clone()
                                            children=move |item: Hibah| {
                                                view! {
                                                    <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                        <td class="p-3 font-medium">{item.pemberi}</td>
                                                        <td class="p-3">{item.penerima}</td>
                                                        <td class="p-3">{item.tanggal_hibah.to_string()}</td>
                                                        <td class="p-3">{item.keterangan.unwrap_or("-".to_string())}</td>
                                                        <td class="p-3">
                                                            <div class="flex gap-2">
                                                                <button class="text-blue-600 hover:text-blue-800" title="Detail">
                                                                    <i class="fas fa-eye"></i>
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
