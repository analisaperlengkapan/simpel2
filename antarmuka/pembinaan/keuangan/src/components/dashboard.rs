//! # SIMPEL Keuangan - Dashboard Layout
//!
//! Layout utama untuk dashboard keuangan dengan navbar, sidebar, dan footer

use crate::components::sidebar::Sidebar;
use crate::components::user_menu::{UserMenu, UserMenuItem};
use leptos::prelude::*;
use shared_microfrontend::components::Footer;

/// User information structure
#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}

/// Dashboard layout component for SIMPEL Keuangan
#[component]
pub fn DashboardLayout(
    /// User information
    user: User,
    /// Sidebar open state
    sidebar_open: RwSignal<bool>,
    /// Children content
    children: Children,
) -> impl IntoView {
    // Toggle sidebar function
    let toggle_sidebar = move |_| {
        sidebar_open.update(|open| *open = !*open);
    };

    // User menu items
    let user_menu_items = vec![
        UserMenuItem::new(
            "Profil",
            Some("/keuangan/profile".to_string()),
            "fas fa-user",
        ),
        UserMenuItem::new(
            "Pengaturan",
            Some("/keuangan/settings".to_string()),
            "fas fa-cog",
        ),
        UserMenuItem::new(
            "Bantuan",
            Some("/keuangan/help".to_string()),
            "fas fa-question-circle",
        ),
        UserMenuItem::new("Logout", None, "fas fa-sign-out-alt"),
    ];

    // Logout handler
    let on_logout: Option<Box<dyn Fn() + Send + Sync>> = Some(Box::new(|| {
        // Clear any stored session data
        if let Some(window) = web_sys::window()
            && let Ok(Some(local_storage)) = window.local_storage()
        {
            let _ = local_storage.remove_item("auth_token");
            let _ = local_storage.remove_item("user_session");
        }
        // Redirect to portal logout
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/portal/logout");
        }
    }));

    view! {
        <div class="min-h-screen bg-gray-50">
            // Custom Header with User Menu
            <header class="bg-white shadow-sm border-b border-gray-200 sticky top-0 z-50">
                <div class="container mx-auto px-4 py-3">
                    <div class="flex items-center justify-between">
                        // Left side - Logo and toggle
                        <div class="flex items-center space-x-4">
                            <button
                                class="p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-500"
                                on:click=toggle_sidebar
                                aria-label="Toggle sidebar"
                            >
                                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>

                            <div class="flex items-center space-x-3">
                                <div class="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center text-white font-bold">
                                    "K"
                                </div>
                                <div class="hidden md:block">
                                    <h1 class="text-lg font-bold text-gray-900">"SIMPEL KEUANGAN"</h1>
                                    <p class="text-xs text-gray-500">"Kejaksaan Republik Indonesia"</p>
                                </div>
                            </div>
                        </div>

                        // Right side - User Menu
                        <div class="flex items-center space-x-4">
                            // Notifications (placeholder)
                            <button class="p-2 rounded-md text-gray-600 hover:text-gray-900 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-blue-500">
                                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5zM15 7h5l-5 5V7zM5 17h5l-5 5v-5zM5 7h5L5 2v5z" />
                                </svg>
                            </button>

                            // User Menu
                            <UserMenu
                                user_name=user.name.clone()
                                user_role=user.role.clone()
                                user_avatar=user.avatar.clone()
                                menu_items=user_menu_items
                                on_logout=on_logout
                            />
                        </div>
                    </div>
                </div>
            </header>

            <div class="flex">
                // Sidebar
                <Sidebar _sidebar_open=sidebar_open />

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
                    "SIMPEL Keuangan v1.0.0"
                </div>
            </Footer>
        </div>
    }
}

/// Sidebar item component
#[component]
fn SidebarItem(
    #[allow(unused_variables)] icon: &'static str,
    #[allow(unused_variables)] title: &'static str,
    #[allow(unused_variables)] href: &'static str,
    #[allow(unused_variables)] expanded: ReadSignal<bool>,
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
    #[allow(unused_variables)] title: &'static str,
    #[allow(unused_variables)] icon: &'static str,
    #[allow(unused_variables)] expanded: ReadSignal<bool>,
    #[allow(unused_variables)] items: Vec<(String, String)>,
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
