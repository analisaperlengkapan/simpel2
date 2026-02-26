//! Halaman placeholder untuk fitur yang sedang dalam pengembangan.

use leptos::prelude::*;

/// Tampilan "Segera Hadir" untuk modul yang belum selesai diimplementasi.
#[component]
pub fn UnderConstruction(
    /// Judul halaman / nama modul
    #[prop(into)]
    title: String,
    /// Deskripsi singkat fungsi modul ini
    #[prop(into, optional)]
    description: Option<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center min-h-[60vh] text-center px-4">
            <div class="mb-6 w-20 h-20 rounded-full bg-amber-100 flex items-center justify-center">
                <i class="fas fa-hard-hat text-amber-500 text-3xl"></i>
            </div>
            <h1 class="text-2xl font-bold text-gray-800 mb-2">{title}</h1>
            <p class="text-gray-500 mb-2 max-w-sm">
                {description.unwrap_or_else(|| "Fitur ini sedang dalam pengembangan.".to_string())}
            </p>
            <p class="text-xs text-gray-400 mb-6">"Akan segera tersedia dalam rilis berikutnya."</p>
            <a
                href="/perlengkapan/dashboard"
                class="inline-flex items-center gap-2 px-5 py-2.5 bg-gray-700 text-white \
                       rounded-lg font-medium text-sm hover:bg-gray-800 transition-colors"
            >
                <i class="fas fa-arrow-left"></i>
                "Kembali ke Dashboard"
            </a>
        </div>
    }
}
