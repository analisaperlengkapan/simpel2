//! 404 Not Found page

use crate::components::layout::AuthLayout;
use leptos::prelude::*;

/// 404 page
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <div class="max-w-md mx-auto text-center">
                <div class="text-8xl font-bold text-gold-400/30 mb-4">"404"</div>
                <h1 class="text-2xl font-bold text-white mb-2">"Halaman Tidak Ditemukan"</h1>
                <p class="text-slate-400 mb-8">
                    "Halaman yang Anda cari tidak ada atau telah dipindahkan."
                </p>
                <a
                    href="/portal/"
                    class="inline-flex items-center px-6 py-3 bg-gradient-to-r from-gold-500 to-gold-600 text-navy-900 font-semibold rounded-xl shadow-lg hover:shadow-xl transition-all"
                >
                    "← Kembali ke Beranda"
                </a>
            </div>
        </AuthLayout>
    }
}
