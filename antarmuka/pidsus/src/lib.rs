use leptos::prelude::*;
use leptos_meta::*;

mod pages;
pub use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/pidsus-microfrontend.css"/>
        <Meta name="description" content="PIDSUS - Penyidikan Khusus"/>
        <Title text="PIDSUS - Penyidikan Khusus"/>

        <div class="min-h-screen bg-gray-50">
            <header class="bg-gradient-to-r from-red-800 to-red-600 text-white p-4 shadow-lg">
                <div class="max-w-7xl mx-auto">
                    <h1 class="text-2xl font-bold">"PIDSUS - Penyidikan Pidana Khusus"</h1>
                    <p class="text-red-100">"Kejaksaan Agung RI"</p>
                </div>
            </header>
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PidsusDashboard/>
                </div>
            </main>
            <footer class="bg-gray-800 text-white p-4">
                <div class="max-w-7xl mx-auto text-center">
                    <p>"© 2024 Kejaksaan Agung RI - PIDSUS SIMPelv2"</p>
                </div>
            </footer>
        </div>
    }
}
