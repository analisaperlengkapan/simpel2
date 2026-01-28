// SIMPEL Pidum - Microfrontend
// Sistem Informasi Pidana Umum untuk Kejaksaan RI
#![recursion_limit = "256"]

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use lib_ui::components::auth::{LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile};
use wasm_bindgen::prelude::*;

mod api;
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
        <Link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0/css/all.min.css" />

        <Router>
            <Routes fallback=|| "Page not found".into_view()>
                <Route path=StaticSegment("") view=|| view! { <LoginRedirectPage /> } />
                <Route path=StaticSegment("dashboard") view=DashboardWithLayout />
                <Route path=StaticSegment("perkara") view=PerkaraListLayout />
                <Route path=StaticSegment("perkara/create") view=PerkaraCreateLayout />
                <Route path=StaticSegment("perkara/:id") view=PerkaraDetailLayout />
            </Routes>
        </Router>
    }
}

// Layout wrapper for Dashboard
#[component]
fn DashboardWithLayout() -> impl IntoView {
    let content = move || {
        view! {
            <div class="min-h-screen bg-gray-50 flex flex-col">
                <PidumHeaderWithAuth />
                <main class="flex-1">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <PidumDashboard/>
                    </div>
                </main>
            </div>
        }
    };
    view! { <ProtectedRoute children=content /> }
}

// Layout wrapper for Detail
#[component]
fn PerkaraDetailLayout() -> impl IntoView {
    let content = move || {
        view! {
            <div class="min-h-screen bg-gray-50 flex flex-col">
                <PidumHeaderWithAuth />
                <main class="flex-1">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <PerkaraDetail/>
                    </div>
                </main>
            </div>
        }
    };
    view! { <ProtectedRoute children=content /> }
}

// Layout wrapper for List
#[component]
fn PerkaraListLayout() -> impl IntoView {
    let content = move || {
        view! {
            <div class="min-h-screen bg-gray-50 flex flex-col">
                <PidumHeaderWithAuth />
                <main class="flex-1">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <DaftarPerkara/>
                    </div>
                </main>
            </div>
        }
    };
    view! { <ProtectedRoute children=content /> }
}

// Layout wrapper for Create
#[component]
fn PerkaraCreateLayout() -> impl IntoView {
    let content = move || {
        view! {
            <div class="min-h-screen bg-gray-50 flex flex-col">
                <PidumHeaderWithAuth />
                <main class="flex-1">
                    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                        <InputPerkara/>
                    </div>
                </main>
            </div>
        }
    };
    view! { <ProtectedRoute children=content /> }
}

#[component]
fn PidumHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-blue-900 text-white shadow-lg sticky top-0 z-50">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center space-x-8">
                        <a href="/dashboard" class="flex items-center space-x-3 group">
                            <span class="text-2xl group-hover:scale-110 transition-transform">"⚖️"</span>
                            <div>
                                <h1 class="text-xl font-bold tracking-tight">"Pidum SIMPelv2"</h1>
                                <p class="text-xs text-blue-300 font-medium tracking-wide uppercase">"Kejaksaan RI"</p>
                            </div>
                        </a>

                        <nav class="hidden md:flex space-x-1">
                            <a href="/dashboard" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-blue-800 transition-colors">"Dashboard"</a>
                            <a href="/perkara" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-blue-800 transition-colors">"Perkara"</a>
                            <a href="#" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-blue-800 transition-colors opacity-50 cursor-not-allowed">"Jadwal Sidang"</a>
                            <a href="#" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-blue-800 transition-colors opacity-50 cursor-not-allowed">"Laporan"</a>
                        </nav>
                    </div>

                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white text-sm font-medium".to_string() />
                        <div class="h-6 w-px bg-blue-800"></div>
                        <LogoutButton class="text-white hover:bg-blue-800 px-3 py-1 rounded text-sm transition-colors".to_string() />
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
