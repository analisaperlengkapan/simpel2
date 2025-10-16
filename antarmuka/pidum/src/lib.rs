// SIMPEL Pidum - Microfrontend
// Sistem Informasi Pidana Umum untuk Kejaksaan RI

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

mod pages;
pub use pages::*;

mod types;
pub use types::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html attr:lang="id" />
        <Title text="PIDUM - Sistem Pidana Umum" />
        <Meta name="description" content="PIDUM - Sistem Pidana Umum Kejaksaan RI" />
        <Meta charset="utf-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1" />
        <Stylesheet id="leptos" href="/pkg/pidum-microfrontend.css"/>

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
                <PidumHeaderWithAuth />
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
fn PidumHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-blue-900 text-white shadow-lg">
            <div class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <a href="/dashboard" class="flex items-center space-x-3">
                            <span class="text-2xl">"⚖️"</span>
                            <div>
                                <h1 class="text-xl font-bold">"Pidum SIMPelv2"</h1>
                                <p class="text-sm text-blue-300">"Pidana Umum"</p>
                            </div>
                        </a>
                    </div>
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-blue-800".to_string() />
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
