//! 404 Not Found Page

use leptos::prelude::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-[60vh] text-center px-4">
            <div class="text-8xl font-black text-gray-200 mb-4 select-none">"404"</div>
            <h1 class="text-2xl font-bold text-gray-800 mb-2">"Halaman Tidak Ditemukan"</h1>
            <p class="text-gray-500 mb-6 max-w-sm">
                "Halaman yang kamu cari tidak ada atau sudah dipindahkan."
            </p>
            <a
                href="/perlengkapan/dashboard"
                class="inline-flex items-center gap-2 px-5 py-2.5 bg-blue-600 text-white \
                       rounded-lg font-medium text-sm hover:bg-blue-700 transition-colors"
            >
                <i class="fas fa-home"></i>
                "Kembali ke Dashboard"
            </a>
        </div>
    }
}
