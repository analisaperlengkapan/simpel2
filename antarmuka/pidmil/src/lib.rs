// SIMPEL Pidmil - Microfrontend
// Sistem Informasi Penyidikan Militer Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::StaticSegment;
use leptos_router::components::{Route, Router, Routes};
use shared_microfrontend::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use wasm_bindgen::prelude::*;

mod components;
pub use components::{
    ActionButton, CaseCard, FormGroup, FormSelect, MilitaryHeader, SearchBox, StatsCard,
    StatusBadge,
};

mod pages;
pub use pages::{
    PidmilDashboard, PidmilKasus, PidmilLaporan, PidmilPenyidikan, PidmilTersangka,
};

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="PIDMIL - Penyidikan Militer" />
        <Meta name="description" content="PIDMIL - Penyidikan Militer Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Stylesheet id="leptos" href="/pkg/pidmil-microfrontend.css"/>

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
                <PidmilHeaderWithAuth />
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
    };

    view! {
        <ProtectedRoute children=content />
    }
}

#[component]
fn PidmilHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-red-800 text-white shadow-lg">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center">
                        <i class="fas fa-shield-alt text-2xl mr-3"></i>
                        <h1 class="text-xl font-bold">"PIDMIL - Penyidikan Militer"</h1>
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
