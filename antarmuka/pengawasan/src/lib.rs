// SIMPEL Pengawasan - Microfrontend
// Sistem Informasi Pengawasan Internal Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use shared_microfrontend::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use wasm_bindgen::prelude::*;

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
        <Html attr:lang="id" />
        <Title text="PENGAWASAN - Sistem Pengawasan Internal" />
        <Meta name="description" content="PENGAWASAN - Sistem Pengawasan Internal Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Stylesheet id="leptos" href="/pkg/pengawasan-microfrontend.css"/>

        <Router>
            <Routes fallback=|| "Page not found".into_view()>
                <Route path=StaticSegment("") view=LoginRedirectPage />
                <Route path=StaticSegment("dashboard") view=DashboardPage />
            </Routes>
        </Router>
    }
}

#[component]
fn DashboardPage() -> impl IntoView {
    let content = move || {
        view! {
            <div class="min-h-screen bg-gray-50">
                <PengawasanHeaderWithAuth />
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
    };

    view! {
        <ProtectedRoute children=content />
    }
}

#[component]
fn PengawasanHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-blue-600 text-white p-4">
            <div class="max-w-7xl mx-auto">
                <div class="flex items-center justify-between">
                    <h1 class="text-xl font-bold">"PENGAWASAN - Sistem Pengawasan Internal"</h1>
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-blue-700".to_string() />
                    </div>
                </div>
            </div>
        </header>
    }
}

/// WASM entry point
#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
