use leptos::prelude::*;
use leptos_meta::*;

mod components;
mod pages;

use components::{Footer, Header};
use pages::PerencanaanDashboard;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/perencanaan-microfrontend.css"/>
        <Meta name="description" content="PEMBINAAN PERENCANAAN - Sistem Pembinaan Perencanaan"/>
        <Title text="PEMBINAAN PERENCANAAN - Sistem Pembinaan Perencanaan"/>

        <div class="min-h-screen bg-gray-50">
            <Header />
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PerencanaanDashboard/>
                </div>
            </main>
            <Footer />
        </div>
    }
}
