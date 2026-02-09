//! Laporan Perencanaan page for SIMPEL Perencanaan

use leptos::prelude::*;

#[component]
pub fn LaporanPerencanaan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white rounded-lg shadow p-6">
                <h1 class="text-2xl font-bold text-gray-900 mb-4">"Laporan Perencanaan BMN"</h1>
                <p class="text-gray-600 mb-6">
                    "Akses laporan perencanaan barang milik negara"
                </p>

                <div class="bg-orange-50 border border-orange-200 rounded-lg p-4">
                    <div class="flex items-center">
                        <i class="fas fa-info-circle text-orange-500 mr-3"></i>
                        <div>
                            <p class="text-orange-800 font-medium">"Halaman Laporan Perencanaan"</p>
                            <p class="text-orange-600 text-sm">"Fitur ini sedang dalam pengembangan"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
