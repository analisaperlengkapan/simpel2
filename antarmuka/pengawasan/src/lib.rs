use leptos::prelude::*;
use leptos_meta::*;

mod components;
pub use components::*;

mod pages;
pub use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/pengawasan-microfrontend.css"/>
        <Meta name="description" content="PENGAWASAN - Sistem Pengawasan Internal Kejaksaan RI"/>
        <Title text="PENGAWASAN - Sistem Pengawasan Internal"/>

        <div class="min-h-screen bg-gray-50">
            <header class="bg-kejaksaan-blue-600 text-white p-4">
                <h1 class="text-xl font-bold">"PENGAWASAN - Sistem Pengawasan Internal"</h1>
            </header>
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PengawasanDashboard/>
                </div>
            </main>
            <footer class="bg-gray-800 text-white p-4 text-center">
                <p>"© 2024 Kejaksaan Agung Republik Indonesia"</p>
            </footer>
        </div>
    }
}
