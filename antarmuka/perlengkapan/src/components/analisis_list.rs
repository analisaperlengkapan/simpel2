use crate::api::{AnalisisKebutuhan, fetch_analisis};
use crate::components::pagination_controls::PaginationControls;
use crate::routes;
use leptos::prelude::*;

#[component]
pub fn AnalisisList() -> impl IntoView {
    let (page, set_page) = signal(1);

    // Resource to fetch items when page changes
    let data_resource = LocalResource::new(move || {
        let p = page.get();
        async move {
            match fetch_analisis(p, 20).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch analisis: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Analisis Kebutuhan"</h2>
                <a
                    href=routes::path::ANALISIS_BUAT_LEGACY
                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors inline-flex items-center"
                >
                    <i class="fas fa-plus mr-2"></i>
                    "Buat Analisis Baru"
                </a>
            </div>

            <Suspense fallback=move || view! { <div class="text-center py-8">"Memuat data..."</div> }>
                {move || {
                    data_resource.get().flatten().map(|response| {
                        if response.data.is_empty() {
                            view! {
                                <div class="text-center py-12 text-gray-500">
                                    <i class="fas fa-chart-pie text-4xl mb-3 text-gray-300"></i>
                                    <p>"Belum ada data analisis kebutuhan."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="w-full text-left border-collapse">
                                    <thead>
                                        <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                            <th class="p-3 font-semibold border-b">"Judul"</th>
                                            <th class="p-3 font-semibold border-b">"Kategori"</th>
                                            <th class="p-3 font-semibold border-b">"Prioritas"</th>
                                            <th class="p-3 font-semibold border-b">"Estimasi Biaya"</th>
                                            <th class="p-3 font-semibold border-b">"Status"</th>
                                            <th class="p-3 font-semibold border-b">"Aksi"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="text-gray-700 text-sm">
                                        <For
                                            each=move || response.data.clone()
                                            key=|item| item.id.clone()
                                            children=move |item: AnalisisKebutuhan| {
                                                view! {
                                                    <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                        <td class="p-3 font-medium">{item.judul}</td>
                                                        <td class="p-3">
                                                            <span class="px-2 py-1 rounded-full text-xs bg-purple-50 text-purple-600">
                                                                {item.kategori}
                                                            </span>
                                                        </td>
                                                        <td class="p-3">
                                                            {
                                                                let prio = item.prioritas.to_lowercase();
                                                                let color = match prio.as_str() {
                                                                    "tinggi" => "text-red-600 bg-red-50",
                                                                    "sedang" => "text-yellow-600 bg-yellow-50",
                                                                    _ => "text-green-600 bg-green-50",
                                                                };
                                                                view! {
                                                                    <span class={format!("px-2 py-1 rounded-full text-xs capitalize {}", color)}>
                                                                        {item.prioritas}
                                                                    </span>
                                                                }
                                                            }
                                                        </td>
                                                        <td class="p-3">
                                                            {
                                                                item.estimasi_biaya
                                                                    .map(|n| format!("Rp {:.2}", n))
                                                                    .unwrap_or("-".to_string())
                                                            }
                                                        </td>
                                                        <td class="p-3">
                                                            <span class="px-2 py-1 rounded-full text-xs bg-gray-100 text-gray-700 capitalize">
                                                                {item.status}
                                                            </span>
                                                        </td>
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
