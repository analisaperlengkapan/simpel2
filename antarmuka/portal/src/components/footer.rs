//! Footer component untuk Portal SIMPelv2
//!
//! Footer yang informatif dengan:
//! - Copyright dan legal information
//! - Quick links ke halaman penting
//! - Contact information
//! - Social media links

use leptos::prelude::*;

/// Footer utama portal
#[component]
pub fn Footer() -> impl IntoView {
    let current_year = 2025; // Atau gunakan js_sys::Date untuk dynamic

    view! {
        <footer class="bg-kejaksaan-primary text-white mt-auto">
            <div class="container mx-auto px-4 py-8">
                <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
                    // Brand Section
                    <div class="space-y-4">
                        <div class="flex items-center space-x-3">
                            <img
                                src="/assets/logo-kejaksaan-white.png"
                                alt="Kejaksaan RI"
                                class="h-8 w-auto"
                            />
                            <div>
                                <h3 class="font-bold">"SIMPelv2"</h3>
                                <p class="text-xs opacity-80">"Portal Utama"</p>
                            </div>
                        </div>
                        <p class="text-sm opacity-80">
                            "Sistem Informasi Manajemen Pengelolaan Barang Milik Negara
                            Kejaksaan Agung Republik Indonesia"
                        </p>
                    </div>

                    // Quick Links
                    <div class="space-y-4">
                        <h4 class="font-semibold">"Navigasi Cepat"</h4>
                        <nav class="flex flex-col space-y-2 text-sm">
                            <a href="/" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Beranda"
                            </a>
                            <a href="/about" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Tentang Sistem"
                            </a>
                            <a href="/help" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Bantuan"
                            </a>
                            <a href="/docs" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Dokumentasi"
                            </a>
                        </nav>
                    </div>

                    // Systems Links
                    <div class="space-y-4">
                        <h4 class="font-semibold">"Sistem Utama"</h4>
                        <nav class="flex flex-col space-y-2 text-sm">
                            <a href="/pidum" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Pidana Umum"
                            </a>
                            <a href="/pidsus" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Pidana Khusus"
                            </a>
                            <a href="/datun" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Perdata & TUN"
                            </a>
                            <a href="/intel" class="opacity-80 hover:opacity-100 transition-opacity">
                                "Intelijen"
                            </a>
                        </nav>
                    </div>

                    // Contact Info
                    <div class="space-y-4">
                        <h4 class="font-semibold">"Kontak"</h4>
                        <div class="text-sm space-y-2 opacity-80">
                            <p>"📧 support@kejaksaan.go.id"</p>
                            <p>"📞 (021) 7805000"</p>
                            <p>"🏢 Jl. Sultan Hasanudin No. 1"</p>
                            <p>"Jakarta Selatan 12560"</p>
                        </div>
                    </div>
                </div>

                // Bottom Bar
                <div class="border-t border-white/20 mt-8 pt-6 flex flex-col md:flex-row justify-between items-center">
                    <p class="text-sm opacity-80">
                        {format!("© {current_year} Kejaksaan Agung Republik Indonesia. All rights reserved.")}
                    </p>

                    <div class="flex items-center space-x-4 mt-4 md:mt-0">
                        <span class="text-xs opacity-60">"Powered by:"</span>
                        <div class="flex items-center space-x-2 text-xs opacity-80">
                            <span>"Rust"</span>
                            <span>"•"</span>
                            <span>"Leptos"</span>
                            <span>"•"</span>
                            <span>"WASM"</span>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    }
}
