use leptos::prelude::*;
use leptos_meta::*;
use shared_microfrontend::components::{AppHeader, KejaksaanFooter};

mod pages;
use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/perlengkapan-microfrontend.css"/>
        <Meta name="description" content="PEMBINAAN PERLENGKAPAN - Sistem Pembinaan Perlengkapan"/>
        <Title text="PEMBINAAN PERLENGKAPAN - Sistem Pembinaan Perlengkapan"/>

        <div class="min-h-screen bg-gray-50">
            <AppHeader title="PEMBINAAN PERLENGKAPAN" />
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PerlengkapanDashboard/>
                </div>
            </main>
            <KejaksaanFooter />
        </div>
    }
}
