use leptos::prelude::*;

#[component]
pub fn DatunFooter() -> impl IntoView {
    view! {
        <footer class="bg-gray-800 text-white mt-12 py-8">
            <div class="container mx-auto px-4">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    <div>
                        <h4 class="text-lg font-bold mb-4">"Datun SIMPelv2"</h4>
                        <p class="text-gray-400 text-sm">
                            "Sistem Informasi Manajemen Pelayanan Hukum Perdata dan Tata Usaha Negara."
                        </p>
                    </div>
                    <div>
                        <h4 class="text-lg font-bold mb-4">"Kontak"</h4>
                        <p class="text-gray-400 text-sm">"Jl. Sultan Hasanuddin No. 1, Kebayoran Baru"</p>
                        <p class="text-gray-400 text-sm">"Jakarta Selatan 12160"</p>
                    </div>
                    <div>
                        <h4 class="text-lg font-bold mb-4">"Versi"</h4>
                        <p class="text-gray-400 text-sm">"v2.1.0 (Build 2025.02)"</p>
                    </div>
                </div>
                <div class="border-t border-gray-700 mt-8 pt-8 text-center text-sm text-gray-500">
                    "© 2025 Kejaksaan Agung Republik Indonesia. Hak Cipta Dilindungi."
                </div>
            </div>
        </footer>
    }
}
