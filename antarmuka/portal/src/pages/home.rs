//! Home page - Landing page for public access

use crate::components::layout::AuthLayout;
use leptos::prelude::*;

/// Home page component - public landing page
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AuthLayout>
            <div class="max-w-6xl mx-auto">
                // ── Hero Section ──
                <section class="text-center py-8 sm:py-12">
                    <div class="inline-flex items-center justify-center w-24 h-24 sm:w-28 sm:h-28 bg-gradient-to-br from-red-600 to-red-700 rounded-full shadow-2xl mb-6">
                        <span class="text-5xl sm:text-6xl">"⚖️"</span>
                    </div>

                    <h1 class="text-4xl sm:text-5xl lg:text-6xl font-extrabold mb-4 bg-gradient-to-r from-red-700 via-red-600 to-red-500 bg-clip-text text-transparent leading-tight">
                        "Portal SIMPelv2"
                    </h1>
                    <p class="text-lg sm:text-xl font-semibold text-gray-700 dark:text-gray-200 mb-2">
                        "Sistem Informasi Manajemen Perlengkapan"
                    </p>
                    <p class="text-base text-gray-500 dark:text-gray-400 mb-2">
                        "Kejaksaan Republik Indonesia"
                    </p>
                    <p class="text-sm italic text-gray-400 dark:text-gray-500 mb-8">
                        "\"Demi Keadilan Berdasarkan Ketuhanan Yang Maha Esa\""
                    </p>

                    <div class="flex flex-col sm:flex-row items-center justify-center gap-3">
                        <a
                            href="/portal/login"
                            class="w-full sm:w-auto inline-flex items-center justify-center gap-2 px-8 py-3 text-base font-semibold text-white bg-red-600 hover:bg-red-700 rounded-xl transition-all duration-200 shadow-lg hover:shadow-xl focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
                        >
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
                            </svg>
                            "Masuk ke Sistem"
                        </a>
                        <a
                            href="#fitur"
                            class="w-full sm:w-auto inline-flex items-center justify-center px-8 py-3 text-base font-medium text-gray-600 dark:text-gray-300 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-xl hover:bg-gray-50 dark:hover:bg-gray-700 transition-all duration-200 shadow-md"
                        >
                            "Pelajari Lebih Lanjut"
                        </a>
                    </div>
                </section>

                // ── Features ──
                <section id="fitur" class="py-8 sm:py-12">
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
                        {[
                            ("🚀", "bg-blue-500", "Modern & Cepat", "WebAssembly + Rust untuk performa maksimal"),
                            ("🔒", "bg-emerald-500", "Aman & Terpercaya", "Enkripsi end-to-end, MFA, zero-trust"),
                            ("🎯", "bg-purple-500", "Mudah Digunakan", "Desain responsif untuk semua perangkat"),
                            ("📊", "bg-orange-500", "Terintegrasi", "Satu portal untuk seluruh layanan"),
                        ].iter().map(|(icon, color, title, desc)| view! {
                            <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-md hover:shadow-xl p-6 transition-all duration-300 border border-gray-100 dark:border-gray-700">
                                <div class=format!("w-14 h-14 {} rounded-xl flex items-center justify-center mb-4", color)>
                                    <span class="text-2xl">{*icon}</span>
                                </div>
                                <h3 class="text-lg font-bold text-gray-900 dark:text-white mb-2">{*title}</h3>
                                <p class="text-sm text-gray-600 dark:text-gray-400 leading-relaxed">{*desc}</p>
                            </div>
                        }).collect_view()}
                    </div>
                </section>

                // ── Statistics ──
                <section class="py-8 sm:py-12">
                    <div class="bg-gradient-to-r from-red-600 to-red-700 rounded-2xl shadow-xl p-8 sm:p-10 text-white">
                        <h2 class="text-2xl sm:text-3xl font-bold text-center mb-8">"SIMPelv2 dalam Angka"</h2>
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-6">
                            {[
                                ("9", "Sistem Terintegrasi"),
                                ("24/7", "Ketersediaan"),
                                ("99.9%", "Uptime"),
                                ("A+", "Skor Keamanan"),
                            ].iter().map(|(num, label)| view! {
                                <div class="text-center">
                                    <div class="text-3xl sm:text-4xl font-extrabold mb-1">{*num}</div>
                                    <div class="text-red-200 text-sm">{*label}</div>
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </section>

                // ── Modules ──
                <section class="py-8 sm:py-12">
                    <h2 class="text-2xl sm:text-3xl font-bold text-center text-gray-900 dark:text-white mb-3">
                        "Modul Aplikasi"
                    </h2>
                    <p class="text-center text-gray-500 dark:text-gray-400 mb-8 max-w-xl mx-auto text-sm">
                        "Akses berbagai sistem kejaksaan dalam satu portal terpadu"
                    </p>
                    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3 sm:gap-4">
                        {[
                            ("⚖️", "Tindak Pidana Umum"),
                            ("🔍", "Tindak Pidana Khusus"),
                            ("🎖️", "Pidana Militer"),
                            ("📜", "Perdata & TUN"),
                            ("🎓", "Badiklat"),
                            ("💰", "Pemulihan Aset"),
                            ("🕵️", "Intelijen"),
                            ("👁️", "Pengawasan"),
                            ("🌱", "Pembinaan"),
                        ].iter().map(|(icon, name)| view! {
                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-sm hover:shadow-lg p-4 sm:p-5 text-center transition-all duration-300 border border-gray-100 dark:border-gray-700 hover:border-red-200 dark:hover:border-red-800">
                                <div class="text-3xl mb-2">{*icon}</div>
                                <div class="text-xs sm:text-sm font-semibold text-gray-700 dark:text-gray-300">{*name}</div>
                            </div>
                        }).collect_view()}
                    </div>
                </section>
            </div>
        </AuthLayout>
    }
}
