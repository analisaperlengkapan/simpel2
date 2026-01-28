// SIMPEL Pemulihan Aset - Microfrontend
// Sistem Informasi Pemulihan Aset Negara untuk Kejaksaan RI

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

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="PEMULIHAN ASET - Sistem Pemulihan Aset Negara" />
        <Meta name="description" content="PEMULIHAN ASET - Sistem Pemulihan Aset Negara Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Stylesheet id="leptos" href="/pkg/pemulihan-aset-microfrontend.css"/>

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
                <PemulihanAsetHeaderWithAuth />
                <main class="flex-1">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <PemulihanAsetDashboard/>
                    </div>
                </main>
            </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

#[component]
fn PemulihanAsetHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-green-700 text-white shadow-lg">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center">
                        <span class="text-2xl mr-3">"💰"</span>
                        <h1 class="text-xl font-bold">"PEMULIHAN ASET - Sistem Pemulihan Aset Negara"</h1>
                    </div>
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-green-800".to_string() />
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
