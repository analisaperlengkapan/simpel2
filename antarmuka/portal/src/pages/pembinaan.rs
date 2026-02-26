//! Pembinaan sub-menu page - Shows Keuangan, Perencanaan, Perlengkapan options

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use leptos::prelude::*;

/// Represents a pembinaan module with metadata
#[derive(Clone, Debug)]
pub struct PembinaanModule {
    /// Unique identifier for the module
    pub id: String,
    /// Display name of the module
    pub name: String,
    /// Description of the module's purpose
    pub description: String,
    /// Icon emoji for visual representation
    pub icon: String,
    /// URL to launch the module
    pub url: String,
    /// CSS color class for styling
    pub color: &'static str,
}

#[component]
pub fn PembinaanPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    let modules = vec![
        PembinaanModule {
            id: "keuangan".to_string(),
            name: "Keuangan".to_string(),
            description: "Sistem Pembinaan Keuangan dan Anggaran".to_string(),
            icon: "💵".to_string(),
            url: "http://localhost:8091".to_string(),
            color: "from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700",
        },
        PembinaanModule {
            id: "perencanaan".to_string(),
            name: "Perencanaan".to_string(),
            description: "Sistem Perencanaan dan Program".to_string(),
            icon: "📊".to_string(),
            url: "http://localhost:8092".to_string(),
            color: "from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700",
        },
        PembinaanModule {
            id: "perlengkapan".to_string(),
            name: "Perlengkapan".to_string(),
            description: "Sistem Perlengkapan dan Aset".to_string(),
            icon: "📦".to_string(),
            url: "http://localhost:8093".to_string(),
            color: "from-orange-500 to-orange-600 hover:from-orange-600 hover:to-orange-700",
        },
    ];

    view! {
        <MainLayout user_session=user_session on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Page Header
                <div class="mb-8">
                    <nav class="flex items-center space-x-2 text-sm text-gray-600 dark:text-gray-400 mb-4">
                        <a href="/portal/apps" class="hover:text-red-600 dark:hover:text-red-400 transition-colors">
                            "← Kembali ke Aplikasi"
                        </a>
                    </nav>
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2 flex items-center">
                        <span class="text-5xl mr-4">"🌱"</span>
                        "Pembinaan"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Pilih modul pembinaan yang ingin diakses"
                    </p>
                </div>

                // Modules Grid
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {modules.into_iter().map(|module| {
                        let module_clone = module.clone();
                        view! {
                            <a
                                href={module.url.clone()}
                                target="_blank"
                                rel="noopener noreferrer"
                                class={format!("group relative bg-gradient-to-br {} text-white rounded-xl shadow-lg p-8 transition-all duration-300 transform hover:scale-105 hover:shadow-2xl", module_clone.color)}
                            >
                                // Icon
                                <div class="text-6xl mb-4">
                                    {module.icon}
                                </div>

                                // Title & Description
                                <h3 class="text-2xl font-bold mb-3">
                                    {module.name}
                                </h3>
                                <p class="text-sm text-white/90 leading-relaxed">
                                    {module.description}
                                </p>

                                // External Link Icon
                                <div class="absolute top-4 right-4 opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/>
                                    </svg>
                                </div>

                                // Coming Soon Badge (if needed)
                                <div class="absolute bottom-4 right-4">
                                    <span class="bg-black/40 text-white text-xs font-semibold px-3 py-1 rounded-full">
                                        "Microfrontend"
                                    </span>
                                </div>
                            </a>
                        }
                    }).collect_view()}
                </div>

                // Info Section
                <div class="mt-12 bg-gradient-to-r from-pink-50 to-pink-100 dark:from-pink-900/20 dark:to-pink-800/20 rounded-xl p-6">
                    <div class="flex items-start space-x-4">
                        <div class="text-3xl">"ℹ️"</div>
                        <div class="flex-1">
                            <h3 class="text-lg font-bold text-gray-900 dark:text-white mb-2">
                                "Tentang Pembinaan"
                            </h3>
                            <p class="text-gray-700 dark:text-gray-300 mb-3">
                                "Sistem Pembinaan SIMPEL terdiri dari tiga modul utama yang saling terintegrasi:"
                            </p>
                            <ul class="space-y-2 text-sm text-gray-600 dark:text-gray-400">
                                <li class="flex items-start">
                                    <span class="text-emerald-600 dark:text-emerald-400 mr-2">"💵"</span>
                                    <span><strong class="text-gray-900 dark:text-white">"Keuangan:"</strong>" Mengelola anggaran, pelaporan keuangan, dan pertanggungjawaban"</span>
                                </li>
                                <li class="flex items-start">
                                    <span class="text-blue-600 dark:text-blue-400 mr-2">"📊"</span>
                                    <span><strong class="text-gray-900 dark:text-white">"Perencanaan:"</strong>" Menyusun rencana strategis, program kerja, dan monitoring kinerja"</span>
                                </li>
                                <li class="flex items-start">
                                    <span class="text-orange-600 dark:text-orange-400 mr-2">"📦"</span>
                                    <span><strong class="text-gray-900 dark:text-white">"Perlengkapan:"</strong>" Mengelola aset, inventaris, dan pengadaan barang"</span>
                                </li>
                            </ul>
                        </div>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
