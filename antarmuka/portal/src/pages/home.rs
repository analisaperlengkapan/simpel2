//! Home page - Landing page for public access

use crate::components::layout::AuthLayout;
use leptos::prelude::*;
use lib_ui::{
    components::Button,
    core::types::{ButtonSize, ButtonVariant},
};

/// Home page component - public landing page
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AuthLayout>
            <div class="w-full max-w-7xl mx-auto px-4">
                // Hero Section - Modern & Engaging
                <div class="text-center mb-20 animate-fade-in">
                    // Logo with gradient background
                    <div class="relative inline-block mb-8">
                        <div class="absolute inset-0 bg-gradient-to-r from-red-600 via-red-500 to-orange-500 rounded-full blur-2xl opacity-30 animate-pulse"></div>
                        <div class="relative inline-flex items-center justify-center w-32 h-32 bg-gradient-to-br from-red-600 to-red-700 rounded-full shadow-2xl">
                            <span class="text-6xl">"⚖️"</span>
                        </div>
                    </div>

                    <h1 class="text-6xl md:text-7xl font-extrabold mb-6 bg-gradient-to-r from-red-600 via-red-500 to-orange-500 bg-clip-text text-transparent">
                        "Portal SIMPelv2"
                    </h1>
                    <p class="text-2xl md:text-3xl font-semibold text-gray-800 dark:text-gray-200 mb-4">
                        "Sistem Informasi Manajemen Pengelolaan"
                    </p>
                    <p class="text-xl text-gray-600 dark:text-gray-400 mb-8 max-w-3xl mx-auto">
                        "Barang Milik Negara - Kejaksaan Agung Republik Indonesia"
                    </p>
                    <p class="text-sm italic text-gray-500 dark:text-gray-500 mb-10">
                        "\"Demi Keadilan Berdasarkan Ketuhanan Yang Maha Esa\""
                    </p>

                    <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
                        <a href="/login" class="w-full sm:w-auto">
                            <Button variant=ButtonVariant::Primary size=ButtonSize::Large>
                                <span class="flex items-center gap-2">
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
                                    </svg>
                                    "Masuk ke Sistem"
                                </span>
                            </Button>
                        </a>
                        <a href="#features" class="w-full sm:w-auto">
                            <button class="w-full px-8 py-3 text-base font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 border-2 border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-200 shadow-md hover:shadow-lg">
                                "Pelajari Lebih Lanjut"
                            </button>
                        </a>
                    </div>
                </div>

                // Features Grid - Enhanced with icons and animations
                <div id="features" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-16">
                    <div class="group bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-8 hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-2 border border-gray-100 dark:border-gray-700">
                        <div class="w-16 h-16 bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300">
                            <span class="text-3xl">"🚀"</span>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">
                            "Modern & Cepat"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400 leading-relaxed">
                            "Dibangun dengan teknologi WebAssembly dan Rust untuk performa maksimal"
                        </p>
                    </div>

                    <div class="group bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-8 hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-2 border border-gray-100 dark:border-gray-700">
                        <div class="w-16 h-16 bg-gradient-to-br from-green-500 to-green-600 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300">
                            <span class="text-3xl">"🔒"</span>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">
                            "Aman & Terpercaya"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400 leading-relaxed">
                            "Keamanan tingkat enterprise dengan enkripsi end-to-end dan MFA"
                        </p>
                    </div>

                    <div class="group bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-8 hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-2 border border-gray-100 dark:border-gray-700">
                        <div class="w-16 h-16 bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300">
                            <span class="text-3xl">"🎯"</span>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">
                            "Mudah Digunakan"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400 leading-relaxed">
                            "Interface intuitif dengan desain responsif untuk semua perangkat"
                        </p>
                    </div>

                    <div class="group bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-8 hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-2 border border-gray-100 dark:border-gray-700">
                        <div class="w-16 h-16 bg-gradient-to-br from-orange-500 to-orange-600 rounded-xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300">
                            <span class="text-3xl">"📊"</span>
                        </div>
                        <h3 class="text-xl font-bold text-gray-900 dark:text-white mb-3">
                            "Terintegrasi"
                        </h3>
                        <p class="text-gray-600 dark:text-gray-400 leading-relaxed">
                            "Satu portal untuk semua layanan kejaksaan yang terintegrasi"
                        </p>
                    </div>
                </div>

                // Statistics Section
                <div class="bg-gradient-to-r from-red-600 to-red-700 rounded-3xl shadow-2xl p-12 mb-16 text-white">
                    <h2 class="text-3xl font-bold text-center mb-12">"SIMPelv2 dalam Angka"</h2>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-8">
                        <div class="text-center">
                            <div class="text-5xl font-extrabold mb-2">"9"</div>
                            <div class="text-red-100">"Sistem Terintegrasi"</div>
                        </div>
                        <div class="text-center">
                            <div class="text-5xl font-extrabold mb-2">"24/7"</div>
                            <div class="text-red-100">"Ketersediaan"</div>
                        </div>
                        <div class="text-center">
                            <div class="text-5xl font-extrabold mb-2">"99.9%"</div>
                            <div class="text-red-100">"Uptime"</div>
                        </div>
                        <div class="text-center">
                            <div class="text-5xl font-extrabold mb-2">"A+"</div>
                            <div class="text-red-100">"Skor Keamanan"</div>
                        </div>
                    </div>
                </div>

                // Modules Overview
                <div class="mb-16">
                    <h2 class="text-3xl font-bold text-center text-gray-900 dark:text-white mb-4">
                        "Modul Aplikasi"
                    </h2>
                    <p class="text-center text-gray-600 dark:text-gray-400 mb-12 max-w-2xl mx-auto">
                        "Akses berbagai sistem kejaksaan dalam satu portal terpadu"
                    </p>
                    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
                        {[
                            ("⚖️", "Tindak Pidana Umum"),
                            ("🔍", "Tindak Pidana Khusus"),
                            ("🎖️", "Pidana Militer"),
                            ("📜", "Perdata dan Tata Usaha Negara"),
                            ("🎓", "Badan Pendidikan dan Pelatihan"),
                            ("💰", "Pemulihan Aset"),
                            ("🕵️", "Intelijen"),
                            ("👁️", "Pengawasan"),
                            ("🌱", "Pembinaan"),
                        ].iter().map(|(icon, name)| view! {
                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-md p-6 text-center hover:shadow-xl transition-all duration-300 transform hover:scale-105 border border-gray-100 dark:border-gray-700">
                                <div class="text-4xl mb-3">{*icon}</div>
                                <div class="text-sm font-semibold text-gray-700 dark:text-gray-300">{*name}</div>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </div>
        </AuthLayout>
    }
}
