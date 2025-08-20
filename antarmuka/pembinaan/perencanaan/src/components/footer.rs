//! Footer Component for Pembinaan Perencanaan Microfrontend
//!
//! Footer dengan informasi Kejaksaan RI dan copyright

use leptos::prelude::*;

/// Footer untuk Pembinaan Perencanaan
#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-kejaksaan-text text-white">
            <div class="container mx-auto px-4 py-8">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    // Informasi Organisasi
                    <div>
                        <h3 class="text-lg font-semibold mb-4">"Kejaksaan Agung RI"</h3>
                        <p class="text-gray-300 text-sm leading-relaxed">
                            "Sistem Informasi Manajemen Perkara dan Layanan (SIMPelv2) - Pembinaan dan Pengembangan Perencanaan Strategis"
                        </p>
                    </div>

                    // Links Terkait
                    <div>
                        <h3 class="text-lg font-semibold mb-4">"Tautan Penting"</h3>
                        <ul class="space-y-2 text-sm">
                            <li>
                                <a href="https://kejaksaan.go.id" target="_blank" class="text-gray-300 hover:text-white transition-colors">
                                    "Portal Utama Kejaksaan"
                                </a>
                            </li>
                            <li>
                                <a href="/help" class="text-gray-300 hover:text-white transition-colors">
                                    "Bantuan & Panduan"
                                </a>
                            </li>
                            <li>
                                <a href="/contact" class="text-gray-300 hover:text-white transition-colors">
                                    "Kontak Support"
                                </a>
                            </li>
                        </ul>
                    </div>

                    // Kontak
                    <div>
                        <h3 class="text-lg font-semibold mb-4">"Kontak"</h3>
                        <div class="text-sm text-gray-300 space-y-2">
                            <p>"📧 support@kejaksaan.go.id"</p>
                            <p>"📞 (021) 7221337"</p>
                            <p>"🏢 Jl. Sultan Hasanuddin No. 1, Jakarta Selatan"</p>
                        </div>
                    </div>
                </div>

                <div class="border-t border-gray-600 mt-8 pt-6">
                    <div class="flex flex-col md:flex-row justify-between items-center">
                        <p class="text-sm text-gray-300">
                            "© 2024 Kejaksaan Agung Republik Indonesia. Hak Cipta Dilindungi."
                        </p>
                        <p class="text-sm text-gray-300 mt-4 md:mt-0">
                            "SIMPelv2 - Perencanaan Strategis v1.0"
                        </p>
                    </div>
                </div>
            </div>
        </footer>
    }
}
