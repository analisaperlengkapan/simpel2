use leptos::prelude::*;
use leptos_meta::*;
use shared_microfrontend::components::AppHeader;

mod pages;
pub use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/pemulihan-aset-microfrontend.css"/>
        <Meta name="description" content="PEMULIHAN ASET - Sistem Pemulihan Aset Negara"/>
        <Title text="PEMULIHAN ASET - Sistem Pemulihan Aset Negara"/>

        <div class="min-h-screen bg-gray-50">
            <AppHeader title="PEMULIHAN ASET - Sistem Pemulihan Aset Negara" />
            <main class="flex-1">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                    <PemulihanAsetDashboard/>
                </div>
            </main>
        </div>
    }
}
