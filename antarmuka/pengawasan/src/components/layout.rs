//! Layout components for Pengawasan microfrontend

use leptos::prelude::*;

/// Main Layout wrapper component
#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <nav class="bg-kejaksaan-blue-700 text-white px-6 py-4">
                <div class="flex items-center justify-between">
                    <h1 class="text-xl font-bold">"Pengawasan BMN"</h1>
                    <div class="flex items-center space-x-4">
                        <a href="/pengawasan" class="hover:underline">"Dashboard"</a>
                        <a href="/pengawasan/schedules" class="hover:underline">"Jadwal"</a>
                    </div>
                </div>
            </nav>
            <main class="container mx-auto px-6 py-8">
                {children()}
            </main>
            <footer class="bg-gray-100 border-t px-6 py-4 text-center text-gray-600">
                <p>"© 2025 Kejaksaan RI - SIMPelv2"</p>
            </footer>
        </div>
    }
}
