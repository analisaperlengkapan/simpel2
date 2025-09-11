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
        <Stylesheet id="leptos" href="/pkg/pidmil-microfrontend.css"/>
        <Meta name="description" content="PIDMIL - Penyidikan Militer"/>
        <Title text="PIDMIL - Penyidikan Militer"/>

        <div class="min-h-screen bg-gray-50">
            <header class="bg-red-800 text-white shadow-lg">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex items-center justify-between h-16">
                        <div class="flex items-center">
                            <i class="fas fa-shield-alt text-2xl mr-3"></i>
                            <h1 class="text-xl font-bold">"PIDMIL - Penyidikan Militer"</h1>
                        </div>
                        <div class="text-sm">
                            "Kejaksaan Agung Republik Indonesia"
                        </div>
                    </div>
                </div>
            </header>

            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PidmilDashboard/>
                </div>
            </main>

            <footer class="bg-gray-800 text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                    <div class="text-center">
                        <p class="text-sm">"© 2024 Kejaksaan Agung Republik Indonesia. Semua hak dilindungi."</p>
                    </div>
                </div>
            </footer>
        </div>
    }
}
