use leptos::prelude::*;

// Simple button component
#[component]
fn ActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let class_str = class.unwrap_or_else(|| {
        "bg-cyan-500 hover:bg-cyan-700 text-white font-bold py-2 px-4 rounded".to_string()
    });

    view! {
        <button
            class={class_str}
            on:click=move |_| {
                web_sys::console::log_1(&format!("Action: {}", action).into());
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
    #[prop(optional)] description: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-white p-6 rounded-lg shadow">
            <h3 class="text-lg font-semibold text-gray-700">{title}</h3>
            <p class="text-3xl font-bold text-cyan-600">{value}</p>
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
                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-cyan-500 focus:border-transparent"
                placeholder={placeholder}
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = on_search {
                        callback.run(val);
                    }
                }
                prop:value=search_value
            />
        </div>
    }
}

/// PERLENGKAPAN Dashboard Page
#[component]
pub fn PerlengkapanDashboard() -> impl IntoView {
    let (stats, _set_stats) = signal(vec![
        (
            "Total Barang".to_string(),
            "1,245".to_string(),
            "fa-boxes".to_string(),
            "cyan".to_string(),
        ),
        (
            "Pengadaan Aktif".to_string(),
            "23".to_string(),
            "fa-shopping-cart".to_string(),
            "blue".to_string(),
        ),
        (
            "Perlu Pemeliharaan".to_string(),
            "87".to_string(),
            "fa-tools".to_string(),
            "orange".to_string(),
        ),
        (
            "Tingkat Availabilitas".to_string(),
            "94%".to_string(),
            "fa-check-circle".to_string(),
            "green".to_string(),
        ),
    ]);

    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold text-gray-900">"Dashboard PERLENGKAPAN"</h1>
                    <p class="text-gray-600">"Sistem Manajemen Perlengkapan"</p>
                </div>
                <div class="flex space-x-3">
                    <ActionButton
                        label="Tambah Barang".to_string()
                        action="new-item".to_string()
                    />
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                {move || stats.get().into_iter().map(|(title, value, _icon, _color)| view! {
                    <StatCard
                        title=title.to_string()
                        value=value.to_string()
                    />
                }).collect::<Vec<_>>()}
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-semibold mb-4">"Barang Terpopuler"</h3>
                    <div class="space-y-2">
                        <p class="text-sm text-gray-600">"Komputer Desktop - 89 unit"</p>
                        <p class="text-sm text-gray-600">"Kursi Kantor - 156 unit"</p>
                        <p class="text-sm text-gray-600">"Meja Kerja - 134 unit"</p>
                    </div>
                </div>
                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-semibold mb-4">"Status Pengadaan"</h3>
                    <div class="space-y-2">
                        <p class="text-sm text-gray-600">"23 pengadaan dalam proses"</p>
                        <p class="text-sm text-gray-600">"12 menunggu persetujuan"</p>
                        <p class="text-sm text-gray-600">"8 siap diterima"</p>
                    </div>
                </div>
                <div class="bg-white p-6 rounded-lg shadow border">
                    <h3 class="text-lg font-semibold mb-4">"Pemeliharaan"</h3>
                    <div class="space-y-2">
                        <p class="text-sm text-gray-600">"87 barang perlu pemeliharaan"</p>
                        <p class="text-sm text-gray-600">"15 dalam perbaikan"</p>
                        <p class="text-sm text-gray-600">"94% tingkat ketersediaan"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// PERLENGKAPAN Inventory Page
#[component]
pub fn PerlengkapanInventaris() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Inventaris Barang"</h1>
                    <p class="text-gray-600">"Kelola inventaris dan stok barang"</p>
                </div>
                <ActionButton
                    label="Tambah Barang".to_string()
                    action="add-item".to_string()
                />
            </div>
            <SearchBox placeholder="Cari barang...".to_string() />
        </div>
    }
}

/// PERLENGKAPAN Procurement Page
#[component]
pub fn PerlengkapanPengadaan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Pengadaan Barang"</h1>
                    <p class="text-gray-600">"Kelola proses pengadaan dan pembelian"</p>
                </div>
                <ActionButton
                    label="Pengadaan Baru".to_string()
                    action="add-procurement".to_string()
                />
            </div>
            <SearchBox placeholder="Cari pengadaan...".to_string() />
        </div>
    }
}

/// PERLENGKAPAN Maintenance Page
#[component]
pub fn PerlengkapanPemeliharaan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Pemeliharaan Barang"</h1>
                    <p class="text-gray-600">"Kelola pemeliharaan dan perbaikan barang"</p>
                </div>
                <ActionButton
                    label="Schedule Pemeliharaan".to_string()
                    action="schedule-maintenance".to_string()
                />
            </div>
            <SearchBox placeholder="Cari pemeliharaan...".to_string() />
        </div>
    }
}

/// PERLENGKAPAN Reports Page
#[component]
pub fn PerlengkapanLaporan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900">"Laporan Perlengkapan"</h1>
                    <p class="text-gray-600">"Laporan inventaris dan aktivitas perlengkapan"</p>
                </div>
                <ActionButton
                    label="Generate Laporan".to_string()
                    action="generate-report".to_string()
                />
            </div>
            <SearchBox placeholder="Cari laporan...".to_string() />
        </div>
    }
}
