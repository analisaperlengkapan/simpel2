//! Rencana Pemeliharaan page for SIMPEL Perencanaan

use leptos::prelude::*;

#[component]
pub fn RencanaPemeliharaan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div class="bg-white rounded-lg shadow p-6">
                <h1 class="text-2xl font-bold text-gray-900 mb-4">"Rencana Pemeliharaan BMN"</h1>
                <p class="text-gray-600 mb-6">
                    "Kelola rencana pemeliharaan barang milik negara"
                </p>

                <div class="bg-green-50 border border-green-200 rounded-lg p-4">
                    <div class="flex items-center">
                        <i class="fas fa-info-circle text-green-500 mr-3"></i>
                        <div>
                            <p class="text-green-800 font-medium">"Halaman Rencana Pemeliharaan"</p>
                            <p class="text-green-600 text-sm">"Fitur ini sedang dalam pengembangan"</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
