//! # SIMPEL Perlengkapan - Dashboard Layout
//!
//! Layout utama untuk dashboard perlengkapan dengan navbar, sidebar, dan footer

use leptos::prelude::*;
use leptos_router::components::Outlet;
use shared_microfrontend::components::{AppHeader, Footer};

/// User data structure (mock untuk development)
#[derive(Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub role: String,
}

/// Dashboard Layout Component
///
/// Layout utama yang berisi:
/// - Header dengan user info
/// - Sidebar dengan menu hierarkis
/// - Content area
/// - Footer
#[component]
pub fn DashboardLayout(children: Children) -> impl IntoView {
    // Mock user data (nanti akan diambil dari authentication)
    let user = User {
        name: "Ahmad Suryanto".to_string(),
        email: "ahmad.suryanto@kejaksaan.go.id".to_string(),
        role: "Staff Perlengkapan".to_string(),
        avatar_url: Some("/assets/default-avatar.png".to_string()),
    };

    let (sidebar_open, set_sidebar_open) = signal(false);

    view! {
        <div class="min-h-screen bg-gray-50">
            // Header
            <AppHeader
                title="SIMPEL Perlengkapan - Kejaksaan RI"
            />

            <div class="flex">
                // Sidebar
                <nav class=move || format!(
                    "bg-white shadow-lg transition-all duration-300 {}",
                    if sidebar_open.get() { "w-64" } else { "w-16" }
                )>
                    <div class="p-4">
                        // Toggle button
                        <button
                            class="mb-4 p-2 rounded-lg hover:bg-gray-100"
                            on:click=move |_| set_sidebar_open.update(|open| *open = !*open)
                        >
                            <i class="fas fa-bars text-gray-600"></i>
                        </button>

                        // Menu items
                        <div class="space-y-2">
                            <SidebarItem
                                icon="fas fa-tachometer-alt"
                                title="Dashboard"
                                href="/dashboard"
                                expanded=sidebar_open
                            />

                            <SidebarSection
                                title="Bank Aset"
                                icon="fas fa-database"
                                expanded=sidebar_open
                                items=vec![
                                    ("Daftar Aset".to_string(), "/dashboard/bank-aset/daftar".to_string()),
                                    ("Peta Sebaran Aset".to_string(), "/dashboard/bank-aset/peta".to_string()),
                                    ("Cetak QR Code BMN".to_string(), "/dashboard/bank-aset/qr-code".to_string()),
                                ]
                            />

                            <SidebarSection
                                title="Analisis Kebutuhan"
                                icon="fas fa-chart-line"
                                expanded=sidebar_open
                                items=vec![
                                    ("Kebutuhan Pakaian".to_string(), "/dashboard/analisis/pakaian".to_string()),
                                    ("Kebutuhan BMN".to_string(), "/dashboard/analisis/bmn".to_string()),
                                    ("Standardisasi BMN".to_string(), "/dashboard/analisis/standardisasi".to_string()),
                                ]
                            />

                            <SidebarSection
                                title="Pengadaan"
                                icon="fas fa-shopping-cart"
                                expanded=sidebar_open
                                items=vec![
                                    ("Administrasi".to_string(), "/dashboard/pengadaan/administrasi".to_string()),
                                    ("Distribusi".to_string(), "/dashboard/pengadaan/distribusi".to_string()),
                                ]
                            />

                            <SidebarSection
                                title="Pengelolaan BMN"
                                icon="fas fa-cogs"
                                expanded=sidebar_open
                                items=vec![
                                    ("Pemakaian BMN".to_string(), "/dashboard/pengelolaan/pemakaian".to_string()),
                                    ("Penerimaan Hibah".to_string(), "/dashboard/pengelolaan/hibah".to_string()),
                                    ("Pengalihan BMN".to_string(), "/dashboard/pengelolaan/pengalihan".to_string()),
                                ]
                            />
                        </div>
                    </div>
                </nav>

                // Main Content Area
                <main class="flex-1 p-6">
                    {children()}
                </main>
            </div>

            // Footer
            <Footer
                copyright="© 2024 Kejaksaan Republik Indonesia"
            >
                <div class="text-sm text-gray-500">
                    "SIMPEL Perlengkapan v1.0.0"
                </div>
            </Footer>
        </div>
    }
}

/// Sidebar item component
#[component]
fn SidebarItem(
    icon: &'static str,
    title: &'static str,
    href: &'static str,
    expanded: ReadSignal<bool>,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center p-3 text-gray-700 rounded-lg hover:bg-blue-50 hover:text-blue-600 transition-colors group"
        >
            <i class=format!("{} text-lg", icon)></i>
            {move || expanded.get().then(|| view! {
                <span class="ml-3 font-medium">{title}</span>
            })}
        </a>
    }
}

/// Sidebar section with expandable submenu
#[component]
fn SidebarSection(
    title: &'static str,
    icon: &'static str,
    expanded: ReadSignal<bool>,
    items: Vec<(String, String)>,
) -> impl IntoView {
    let (section_open, set_section_open) = signal(false);

    view! {
        <div class="space-y-1">
            <button
                class="w-full flex items-center justify-between p-3 text-gray-700 rounded-lg hover:bg-gray-100 transition-colors"
                on:click=move |_| set_section_open.update(|open| *open = !*open)
            >
                <div class="flex items-center">
                    <i class=format!("{} text-lg", icon)></i>
                    {move || expanded.get().then(|| view! {
                        <span class="ml-3 font-medium">{title}</span>
                    })}
                </div>
                {move || expanded.get().then(|| view! {
                    <i class=move || format!(
                        "fas fa-chevron-{} text-xs transition-transform",
                        if section_open.get() { "down" } else { "right" }
                    )></i>
                })}
            </button>

            {move || (section_open.get() && expanded.get()).then(|| view! {
                <div class="ml-6 space-y-1">
                    {items.iter().map(|(name, href)| view! {
                        <a
                            href=href.clone()
                            class="block p-2 text-sm text-gray-600 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
                        >
                            {name.clone()}
                        </a>
                    }).collect::<Vec<_>>()}
                </div>
            })}
        </div>
    }
}
