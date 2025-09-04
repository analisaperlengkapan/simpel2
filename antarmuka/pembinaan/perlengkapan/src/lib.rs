use leptos::prelude::*;
use leptos::{either::Either, mount::mount_to_body};
use wasm_bindgen::prelude::*;
use web_sys::SubmitEvent;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Public function that main.rs will call
pub fn start_app() {
    console_log!("Starting perlengkapan app");
    mount_to_body(App)
}

#[component]
pub fn App() -> impl IntoView {
    console_log!("Rendering App component");

    let (current_view, set_current_view) = signal("login".to_string());
    let (is_authenticated, set_is_authenticated) = signal(false);

    let login_handler = move || {
        console_log!("Login button clicked");
        set_is_authenticated.set(true);
        set_current_view.set("dashboard".to_string());
    };

    let logout_handler = move || {
        console_log!("Logout clicked");
        set_is_authenticated.set(false);
        set_current_view.set("login".to_string());
    };

    let nav_handler = {
        let set_current_view = set_current_view.clone();
        move |view: String| {
            console_log!("Navigation to: {}", &view);
            set_current_view.set(view);
        }
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-blue-50 via-white to-green-50">
            {move || {
                if is_authenticated.get() {
                    Either::Left(view! {
                        <DashboardView
                            current_view=current_view
                            on_logout=logout_handler
                            on_navigate=nav_handler
                        />
                    })
                } else {
                    Either::Right(view! {
                        <LoginView on_login=login_handler />
                    })
                }
            }}
        </div>
    }
}

#[component]
fn LoginView<F>(on_login: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <div class="min-h-screen flex items-center justify-center relative overflow-hidden">
            // Background decorative elements
            <div class="absolute inset-0 bg-gradient-to-br from-blue-900 via-blue-800 to-green-800"></div>
            <div class="absolute top-10 left-10 w-32 h-32 bg-white/10 rounded-full blur-xl"></div>
            <div class="absolute bottom-10 right-10 w-48 h-48 bg-green-400/20 rounded-full blur-2xl"></div>
            <div class="absolute top-1/2 left-1/4 w-24 h-24 bg-blue-300/15 rounded-full blur-lg"></div>

            <div class="relative z-10 bg-white/95 backdrop-blur-lg rounded-3xl shadow-2xl p-12 w-full max-w-md border border-white/20">
                <div class="text-center mb-10">
                    <div class="inline-flex items-center justify-center w-20 h-20 bg-gradient-to-r from-blue-600 to-green-600 rounded-2xl mb-6 shadow-lg">
                        <i class="fas fa-shield-alt text-white text-2xl"></i>
                    </div>
                    <h1 class="text-3xl font-bold bg-gradient-to-r from-blue-800 to-green-800 bg-clip-text text-transparent mb-2">
                        "SIMPEL"
                    </h1>
                    <p class="text-gray-600 font-medium">"Sistem Manajemen Perlengkapan"</p>
                    <div class="w-24 h-1 bg-gradient-to-r from-blue-500 to-green-500 mx-auto mt-4 rounded-full"></div>
                </div>

                <form class="space-y-6" on:submit=move |e: SubmitEvent| {
                    e.prevent_default();
                    on_login();
                }>
                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-3">"Username"</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                <i class="fas fa-user text-gray-400"></i>
                            </div>
                            <input
                                type="text"
                                class="w-full pl-12 pr-4 py-4 border border-gray-200 rounded-xl focus:ring-4 focus:ring-blue-500/20 focus:border-blue-500 transition-all duration-300 bg-white/80 backdrop-blur-sm"
                                placeholder="Masukkan username"
                            />
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-semibold text-gray-700 mb-3">"Password"</label>
                        <div class="relative">
                            <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
                                <i class="fas fa-lock text-gray-400"></i>
                            </div>
                            <input
                                type="password"
                                class="w-full pl-12 pr-4 py-4 border border-gray-200 rounded-xl focus:ring-4 focus:ring-blue-500/20 focus:border-blue-500 transition-all duration-300 bg-white/80 backdrop-blur-sm"
                                placeholder="Masukkan password"
                            />
                        </div>
                    </div>

                    <button
                        type="submit"
                        class="w-full bg-gradient-to-r from-blue-600 to-green-600 text-white py-4 rounded-xl font-semibold shadow-lg hover:shadow-xl transform hover:scale-[1.02] transition-all duration-300 focus:ring-4 focus:ring-blue-500/30"
                    >
                        <i class="fas fa-sign-in-alt mr-2"></i>
                        "Masuk ke SIMPEL"
                    </button>
                </form>

                <div class="mt-8 text-center">
                    <p class="text-xs text-gray-500">"© 2024 SIMPEL - Sistem Manajemen Perlengkapan"</p>
                    <div class="flex justify-center space-x-4 mt-4">
                        <div class="flex items-center text-xs text-gray-400">
                            <i class="fas fa-shield-check mr-1"></i>
                            "Secure"
                        </div>
                        <div class="flex items-center text-xs text-gray-400">
                            <i class="fas fa-lock mr-1"></i>
                            "Encrypted"
                        </div>
                        <div class="flex items-center text-xs text-gray-400">
                            <i class="fas fa-check-circle mr-1"></i>
                            "Verified"
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DashboardView<F, G>(
    current_view: ReadSignal<String>,
    on_logout: F,
    on_navigate: G,
) -> impl IntoView
where
    F: Fn() + 'static,
    G: Fn(String) + Clone + 'static,
{
    view! {
        <div class="min-h-screen bg-gradient-to-br from-gray-50 via-blue-50/30 to-green-50/30">
            <TopBar on_logout=on_logout />
            <div class="flex">
                <Sidebar current_view=current_view on_navigate=on_navigate />
                <MainContent current_view=current_view />
            </div>
        </div>
    }
}

#[component]
fn TopBar<F>(on_logout: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <nav class="bg-white/90 backdrop-blur-lg shadow-lg border-b border-gray-200/50 sticky top-0 z-50">
            <div class="px-6 py-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-6">
                        <div class="flex items-center space-x-3">
                            <div class="w-10 h-10 bg-gradient-to-r from-blue-600 to-green-600 rounded-xl flex items-center justify-center shadow-lg">
                                <i class="fas fa-shield-alt text-white text-lg"></i>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold bg-gradient-to-r from-blue-800 to-green-800 bg-clip-text text-transparent">
                                    "SIMPEL"
                                </h1>
                                <p class="text-xs text-gray-500 font-medium">"Perlengkapan"</p>
                            </div>
                        </div>

                        <div class="hidden md:flex items-center space-x-2 text-sm text-gray-600">
                            <i class="fas fa-home text-blue-500"></i>
                            <span>"Dashboard"</span>
                            <i class="fas fa-chevron-right text-xs text-gray-400"></i>
                            <span class="text-green-600 font-medium">"Perlengkapan"</span>
                        </div>
                    </div>

                    <div class="flex items-center space-x-4">
                        <button class="relative p-2 text-gray-600 hover:text-blue-600 hover:bg-blue-50 rounded-xl transition-all duration-200">
                            <i class="fas fa-bell text-lg"></i>
                            <span class="absolute -top-1 -right-1 w-5 h-5 bg-red-500 text-white text-xs rounded-full flex items-center justify-center">
                                "3"
                            </span>
                        </button>

                        <div class="flex items-center space-x-3 px-4 py-2 bg-gradient-to-r from-blue-50 to-green-50 rounded-xl border border-blue-200/50">
                            <div class="w-8 h-8 bg-gradient-to-r from-blue-500 to-green-500 rounded-lg flex items-center justify-center">
                                <i class="fas fa-user text-white text-sm"></i>
                            </div>
                            <div class="hidden sm:block">
                                <p class="text-sm font-semibold text-gray-800">"Admin User"</p>
                                <p class="text-xs text-gray-500">"Perlengkapan"</p>
                            </div>
                        </div>

                        <button
                            on:click=move |_| on_logout()
                            class="px-4 py-2 bg-gradient-to-r from-red-500 to-red-600 text-white rounded-xl hover:shadow-lg transform hover:scale-105 transition-all duration-200 flex items-center space-x-2"
                        >
                            <i class="fas fa-sign-out-alt"></i>
                            <span class="hidden sm:inline">"Keluar"</span>
                        </button>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn Sidebar<F>(current_view: ReadSignal<String>, on_navigate: F) -> impl IntoView
where
    F: Fn(String) + Clone + 'static,
{
    let menu_items = vec![
        (
            "dashboard",
            "Dashboard",
            "fas fa-tachometer-alt",
            "from-blue-500 to-blue-600",
        ),
        (
            "perencanaan",
            "Perencanaan",
            "fas fa-calendar-alt",
            "from-purple-500 to-purple-600",
        ),
        (
            "pengadaan",
            "Pengadaan",
            "fas fa-shopping-cart",
            "from-green-500 to-green-600",
        ),
        (
            "penerimaan",
            "Penerimaan",
            "fas fa-clipboard-check",
            "from-teal-500 to-teal-600",
        ),
        (
            "penyimpanan",
            "Penyimpanan",
            "fas fa-warehouse",
            "from-yellow-500 to-yellow-600",
        ),
        (
            "distribusi",
            "Distribusi",
            "fas fa-truck",
            "from-orange-500 to-orange-600",
        ),
        (
            "inventarisasi",
            "Inventarisasi",
            "fas fa-list-alt",
            "from-indigo-500 to-indigo-600",
        ),
        (
            "maintenance",
            "Pemeliharaan",
            "fas fa-tools",
            "from-red-500 to-red-600",
        ),
        (
            "pemusnahan",
            "Pemusnahan",
            "fas fa-trash-alt",
            "from-gray-500 to-gray-600",
        ),
        (
            "laporan",
            "Laporan",
            "fas fa-chart-bar",
            "from-pink-500 to-pink-600",
        ),
        (
            "audit",
            "Audit",
            "fas fa-search",
            "from-cyan-500 to-cyan-600",
        ),
        (
            "settings",
            "Pengaturan",
            "fas fa-cog",
            "from-violet-500 to-violet-600",
        ),
    ];

    view! {
        <aside class="w-72 bg-white/80 backdrop-blur-lg shadow-xl border-r border-gray-200/50 min-h-screen">
            <div class="p-6">
                <div class="mb-8">
                    <div class="bg-gradient-to-r from-blue-500/10 to-green-500/10 rounded-2xl p-4 border border-blue-200/30">
                        <h2 class="text-lg font-bold text-gray-800 mb-1">"Menu Perlengkapan"</h2>
                        <p class="text-sm text-gray-600">"Sistem Manajemen Aset"</p>
                    </div>
                </div>

                <nav class="space-y-2">
                    {menu_items.into_iter().map(|(view, label, icon, gradient)| {
                        let nav_handler = on_navigate.clone();
                        let view_str = view.to_string();
                        let is_active = move || current_view.get() == view;

                        view! {
                            <button
                                on:click=move |_| nav_handler(view_str.clone())
                                class=move || format!(
                                    "w-full flex items-center space-x-4 px-4 py-3 rounded-xl transition-all duration-300 group {}",
                                    if is_active() {
                                        format!("bg-gradient-to-r {} text-white shadow-lg transform scale-[1.02]", gradient)
                                    } else {
                                        "text-gray-700 hover:bg-gradient-to-r hover:from-gray-50 hover:to-blue-50 hover:shadow-md hover:scale-[1.01]".to_string()
                                    }
                                )
                            >
                                <div class=move || format!(
                                    "w-10 h-10 rounded-lg flex items-center justify-center transition-all duration-300 {}",
                                    if is_active() {
                                        "bg-white/20 shadow-lg".to_string()
                                    } else {
                                        format!("bg-gradient-to-r {} text-white group-hover:scale-110", gradient)
                                    }
                                )>
                                    <i class=format!("{} text-sm", icon)></i>
                                </div>
                                <div class="flex-1 text-left">
                                    <span class="font-medium text-sm">{label}</span>
                                </div>
                                <i class=move || format!(
                                    "fas fa-chevron-right text-xs transition-transform duration-300 {}",
                                    if is_active() { "rotate-90" } else { "group-hover:translate-x-1" }
                                )></i>
                            </button>
                        }
                    }).collect_view()}
                </nav>

                <div class="mt-8 p-4 bg-gradient-to-r from-blue-50 to-green-50 rounded-2xl border border-blue-200/50">
                    <div class="flex items-center space-x-3 mb-3">
                        <div class="w-8 h-8 bg-gradient-to-r from-blue-500 to-green-500 rounded-lg flex items-center justify-center">
                            <i class="fas fa-info-circle text-white text-xs"></i>
                        </div>
                        <span class="text-sm font-semibold text-gray-800">"Status Sistem"</span>
                    </div>
                    <div class="space-y-2 text-xs">
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Server"</span>
                            <div class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                                <span class="text-green-600 font-medium">"Online"</span>
                            </div>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Database"</span>
                            <div class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                                <span class="text-green-600 font-medium">"Connected"</span>
                            </div>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-gray-600">"Sync"</span>
                            <div class="flex items-center space-x-1">
                                <div class="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
                                <span class="text-blue-600 font-medium">"Active"</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </aside>
    }
}

#[component]
fn MainContent(current_view: ReadSignal<String>) -> impl IntoView {
    view! {
        <main class="flex-1 p-8">
            {move || {
                match current_view.get().as_str() {
                    "dashboard" => view! { <DashboardContent /> }.into_any(),
                    "perencanaan" => view! { <PerencanaanContent /> }.into_any(),
                    "pengadaan" => view! { <PengadaanContent /> }.into_any(),
                    "penerimaan" => view! { <PenerimaanContent /> }.into_any(),
                    "penyimpanan" => view! { <PenyimpananContent /> }.into_any(),
                    "distribusi" => view! { <DistribusiContent /> }.into_any(),
                    "inventarisasi" => view! { <InventarisasiContent /> }.into_any(),
                    "maintenance" => view! { <MaintenanceContent /> }.into_any(),
                    "pemusnahan" => view! { <PemusnahanContent /> }.into_any(),
                    "laporan" => view! { <LaporanContent /> }.into_any(),
                    "audit" => view! { <AuditContent /> }.into_any(),
                    "settings" => view! { <SettingsContent /> }.into_any(),
                    _ => view! { <DashboardContent /> }.into_any(),
                }
            }}
        </main>
    }
}

#[component]
fn DashboardContent() -> impl IntoView {
    view! {
        <div class="space-y-8">
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold bg-gradient-to-r from-blue-800 to-green-800 bg-clip-text text-transparent">
                        "Dashboard Perlengkapan"
                    </h1>
                    <p class="text-gray-600 mt-2">"Ringkasan dan statistik sistem manajemen perlengkapan"</p>
                </div>
                <div class="flex space-x-4">
                    <button class="px-6 py-3 bg-gradient-to-r from-blue-500 to-blue-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200">
                        <i class="fas fa-download mr-2"></i>
                        "Export Data"
                    </button>
                    <button class="px-6 py-3 bg-gradient-to-r from-green-500 to-green-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200">
                        <i class="fas fa-sync mr-2"></i>
                        "Refresh"
                    </button>
                </div>
            </div>

            // Stats Cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="bg-gradient-to-br from-blue-500 to-blue-600 rounded-2xl p-6 text-white shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-blue-100 text-sm font-medium">"Total Aset"</p>
                            <p class="text-3xl font-bold">"2,547"</p>
                            <p class="text-blue-100 text-xs mt-1">"↗ +12% dari bulan lalu"</p>
                        </div>
                        <div class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center">
                            <i class="fas fa-boxes text-2xl"></i>
                        </div>
                    </div>
                </div>

                <div class="bg-gradient-to-br from-green-500 to-green-600 rounded-2xl p-6 text-white shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-green-100 text-sm font-medium">"Nilai Total"</p>
                            <p class="text-3xl font-bold">"15.2B"</p>
                            <p class="text-green-100 text-xs mt-1">"↗ +8% dari bulan lalu"</p>
                        </div>
                        <div class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center">
                            <i class="fas fa-money-bill-wave text-2xl"></i>
                        </div>
                    </div>
                </div>

                <div class="bg-gradient-to-br from-purple-500 to-purple-600 rounded-2xl p-6 text-white shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-purple-100 text-sm font-medium">"Pengadaan Aktif"</p>
                            <p class="text-3xl font-bold">"24"</p>
                            <p class="text-purple-100 text-xs mt-1">"↗ +3 dari minggu lalu"</p>
                        </div>
                        <div class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center">
                            <i class="fas fa-shopping-cart text-2xl"></i>
                        </div>
                    </div>
                </div>

                <div class="bg-gradient-to-br from-orange-500 to-orange-600 rounded-2xl p-6 text-white shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                    <div class="flex items-center justify-between">
                        <div>
                            <p class="text-orange-100 text-sm font-medium">"Maintenance"</p>
                            <p class="text-3xl font-bold">"8"</p>
                            <p class="text-orange-100 text-xs mt-1">"↘ -2 dari minggu lalu"</p>
                        </div>
                        <div class="w-16 h-16 bg-white/20 rounded-2xl flex items-center justify-center">
                            <i class="fas fa-tools text-2xl"></i>
                        </div>
                    </div>
                </div>
            </div>

            // Recent Activities
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
                <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-6 border border-gray-200/50">
                    <div class="flex items-center justify-between mb-6">
                        <h3 class="text-xl font-bold text-gray-800">"Aktivitas Terbaru"</h3>
                        <button class="text-blue-600 hover:text-blue-700 text-sm font-medium">
                            "Lihat Semua"
                            <i class="fas fa-arrow-right ml-1"></i>
                        </button>
                    </div>
                    <div class="space-y-4">
                        <div class="flex items-start space-x-4 p-4 bg-blue-50/50 rounded-xl border border-blue-200/30">
                            <div class="w-10 h-10 bg-gradient-to-r from-blue-500 to-blue-600 rounded-xl flex items-center justify-center flex-shrink-0">
                                <i class="fas fa-plus text-white text-sm"></i>
                            </div>
                            <div class="flex-1">
                                <p class="text-sm font-semibold text-gray-800">"Penambahan Aset Baru"</p>
                                <p class="text-xs text-gray-600 mt-1">"Laptop Dell Latitude ditambahkan ke inventori"</p>
                                <p class="text-xs text-blue-600 mt-2">"5 menit yang lalu"</p>
                            </div>
                        </div>
                        <div class="flex items-start space-x-4 p-4 bg-green-50/50 rounded-xl border border-green-200/30">
                            <div class="w-10 h-10 bg-gradient-to-r from-green-500 to-green-600 rounded-xl flex items-center justify-center flex-shrink-0">
                                <i class="fas fa-check text-white text-sm"></i>
                            </div>
                            <div class="flex-1">
                                <p class="text-sm font-semibold text-gray-800">"Pengadaan Disetujui"</p>
                                <p class="text-xs text-gray-600 mt-1">"Pengadaan printer HP LaserJet telah disetujui"</p>
                                <p class="text-xs text-green-600 mt-2">"1 jam yang lalu"</p>
                            </div>
                        </div>
                        <div class="flex items-start space-x-4 p-4 bg-yellow-50/50 rounded-xl border border-yellow-200/30">
                            <div class="w-10 h-10 bg-gradient-to-r from-yellow-500 to-yellow-600 rounded-xl flex items-center justify-center flex-shrink-0">
                                <i class="fas fa-exclamation-triangle text-white text-sm"></i>
                            </div>
                            <div class="flex-1">
                                <p class="text-sm font-semibold text-gray-800">"Peringatan Maintenance"</p>
                                <p class="text-xs text-gray-600 mt-1">"AC Unit-5 memerlukan maintenance rutin"</p>
                                <p class="text-xs text-yellow-600 mt-2">"3 jam yang lalu"</p>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-6 border border-gray-200/50">
                    <div class="flex items-center justify-between mb-6">
                        <h3 class="text-xl font-bold text-gray-800">"Quick Actions"</h3>
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <button class="p-4 bg-gradient-to-br from-blue-500 to-blue-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                            <i class="fas fa-plus text-2xl mb-2"></i>
                            <p class="text-sm font-semibold">"Tambah Aset"</p>
                        </button>
                        <button class="p-4 bg-gradient-to-br from-green-500 to-green-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                            <i class="fas fa-shopping-cart text-2xl mb-2"></i>
                            <p class="text-sm font-semibold">"Buat Pengadaan"</p>
                        </button>
                        <button class="p-4 bg-gradient-to-br from-purple-500 to-purple-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                            <i class="fas fa-chart-bar text-2xl mb-2"></i>
                            <p class="text-sm font-semibold">"Lihat Laporan"</p>
                        </button>
                        <button class="p-4 bg-gradient-to-br from-orange-500 to-orange-600 text-white rounded-xl shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-300">
                            <i class="fas fa-tools text-2xl mb-2"></i>
                            <p class="text-sm font-semibold">"Schedule Maintenance"</p>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// Placeholder components for other views
#[component]
fn PerencanaanContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Perencanaan Perlengkapan"</h2>
            <p class="text-gray-600">"Modul perencanaan untuk kebutuhan perlengkapan akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn PengadaanContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Pengadaan Barang"</h2>
            <p class="text-gray-600">"Modul pengadaan barang dan jasa akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn PenerimaanContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Penerimaan Barang"</h2>
            <p class="text-gray-600">"Modul penerimaan dan verifikasi barang akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn PenyimpananContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Penyimpanan & Gudang"</h2>
            <p class="text-gray-600">"Modul manajemen penyimpanan dan gudang akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn DistribusiContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Distribusi Barang"</h2>
            <p class="text-gray-600">"Modul distribusi dan penyaluran barang akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn InventarisasiContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Inventarisasi Aset"</h2>
            <p class="text-gray-600">"Modul inventarisasi dan pencatatan aset akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn MaintenanceContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Pemeliharaan Aset"</h2>
            <p class="text-gray-600">"Modul pemeliharaan dan perawatan aset akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn PemusnahanContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Pemusnahan Aset"</h2>
            <p class="text-gray-600">"Modul pemusnahan dan penghapusan aset akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn LaporanContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Laporan & Analitik"</h2>
            <p class="text-gray-600">"Modul laporan dan analitik akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn AuditContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Audit & Compliance"</h2>
            <p class="text-gray-600">"Modul audit dan kepatuhan akan segera hadir..."</p>
        </div>
    }
}

#[component]
fn SettingsContent() -> impl IntoView {
    view! {
        <div class="bg-white/80 backdrop-blur-lg rounded-2xl shadow-xl p-8">
            <h2 class="text-2xl font-bold text-gray-800 mb-4">"Pengaturan Sistem"</h2>
            <p class="text-gray-600">"Modul pengaturan dan konfigurasi sistem akan segera hadir..."</p>
        </div>
    }
}
