// SIMPEL Pidsus - Microfrontend
// Sistem Informasi Penyidikan Pidana Khusus Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use lib_ui::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use wasm_bindgen::prelude::*;

mod pages;
pub use pages::*;

mod types;
pub use types::*;

mod api;

mod components;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="PIDSUS - Penyidikan Khusus" />
        <Meta name="description" content="PIDSUS - Penyidikan Pidana Khusus Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Stylesheet id="leptos" href="/pkg/pidsus-microfrontend.css"/>

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
                <PidsusHeaderWithAuth />
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
    };

    view! {
        <ProtectedRoute children=content />
    }
}

#[component]
fn PidsusHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-red-800 to-red-600 text-white p-4 shadow-lg">
            <div class="max-w-7xl mx-auto">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-bold">"PIDSUS - Penyidikan Pidana Khusus"</h1>
                        <p class="text-red-100">"Kejaksaan Agung RI"</p>
                    </div>
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-red-700".to_string() />
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
