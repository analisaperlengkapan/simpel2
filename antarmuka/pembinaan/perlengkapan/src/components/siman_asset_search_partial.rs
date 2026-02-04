/// SIMAN Asset Summary Card
#[component]
pub fn SimanAssetSummaryCard(#[prop(into)] satker_id: String) -> impl IntoView {
    let satker_id_clone = satker_id.clone();
    let (loading, set_loading) = signal(true);
    let (summary, set_summary) = signal::<Option<crate::api::SatkerAssetSummary>>(None);
    let (error, set_error) = signal::<Option<String>>(None);

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

            {move || summary.get().map(|s| {
                let by_category = store_value(s.by_category.clone());
                let total_assets = s.total_assets;
                let total_value = s.total_value;

                view! {
                    <div class="space-y-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div class="bg-white rounded-lg p-4 text-center shadow-sm">
                                <p class="text-3xl font-bold text-blue-600">{total_assets}</p>
                                <p class="text-sm text-gray-500">"Total Aset"</p>
                            </div>
                            <div class="bg-white rounded-lg p-4 text-center shadow-sm">
                                <p class="text-lg font-bold text-green-600">
                                    {format!("Rp {:.0}", total_value)}
                                </p>
                                <p class="text-sm text-gray-500">"Total Nilai"</p>
                            </div>
                        </div>

                        <Show when=move || !by_category.with_value(|v| v.is_empty())>
                            <div>
                                <h5 class="font-semibold text-gray-700 mb-2">"Per Kategori"</h5>
                                <div class="space-y-2">
                                    <For
                                        each=move || by_category.get_value()
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
                }
            })}
        </div>
    }
}
