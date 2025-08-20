use leptos::prelude::*;
use leptos_meta::*;

// Import shared components
use shared_microfrontend::{KejaksaanHeader, KejaksaanFooter};

mod pages;
use pages::IntelDashboard;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        // Document metadata
        <Title text="Intel - SIMPelv2"/>
        <Meta name="description" content="Sistem Informasi Intelligence - Kejaksaan Agung RI"/>

        // Simple App Layout
        <div class="min-h-screen bg-gray-50">
            <KejaksaanHeader />

            <div class="bg-blue-600 text-white py-4">
                <div class="container mx-auto px-4">
                    <h1 class="text-2xl font-bold">"Intel"</h1>
                    <p class="text-blue-200">"Sistem Intelligence"</p>
                </div>
            </div>

            <main class="container mx-auto px-4 py-8">
                <IntelDashboard />
            </main>

            <KejaksaanFooter />
        </div>
    }
}
