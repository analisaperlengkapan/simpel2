//! 404 Not Found page

use crate::components::layout::AuthLayout;
use leptos::prelude::*;

/// 404 Not Found page component
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <div class="text-center">
                <div class="text-8xl mb-8">"🔍"</div>
                <h1 class="text-6xl font-bold text-gray-900 dark:text-white mb-4">
                    "404"
                </h1>
                <p class="text-xl text-gray-600 dark:text-gray-400 mb-8">
                    "Halaman yang Anda cari tidak ditemukan"
                </p>
                <a href="/" class="text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 font-medium inline-block">
                    "← Kembali ke Beranda"
                </a>
            </div>
        </AuthLayout>
    }
}
