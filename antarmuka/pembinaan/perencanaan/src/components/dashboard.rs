//! # SIMPEL Perencanaan - Dashboard Layout
//!
//! Layout utama untuk dashboard perencanaan dengan navbar, sidebar, dan footer

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

/// Dashboard layout component for SIMPEL Perencanaan
#[component]
pub fn DashboardLayout(
    /// Child content to render
    children: Children,
) -> impl IntoView {
    // Sidebar open state
    let sidebar_open = RwSignal::new(false);

    // Mock user data - in real app this would come from authentication
    let user = User {
        name: "John Doe".to_string(),
        role: "Administrator".to_string(),
        avatar: None,
    };

    // User menu items
    let user_menu_items = vec![
        UserMenuItem::new(
            "Profil",
            Some("/perencanaan/profile".to_string()),
            "fas fa-user",
        ),
        UserMenuItem::new(
            "Pengaturan",
            Some("/perencanaan/settings".to_string()),
            "fas fa-cog",
        ),
        UserMenuItem::new(
            "Bantuan",
            Some("/perencanaan/help".to_string()),
            "fas fa-question-circle",
        ),
        UserMenuItem::new("Logout", None, "fas fa-sign-out-alt"),
    ];

    // Logout handler
    let on_logout: Box<dyn Fn() + Send + Sync> = Box::new(|| {
        // Clear any stored session data
        if let Some(window) = web_sys::window() {
            if let Ok(Some(local_storage)) = window.local_storage() {
                let _ = local_storage.remove_item("auth_token");
                let _ = local_storage.remove_item("user_session");
            }
        }
        // Redirect to portal logout
        if let Some(window) = web_sys::window() {
            let _ = window.location().set_href("/portal/logout");
        }
    });

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
                                on:click=move |_| sidebar_open.update(|open| *open = !*open)
                                aria-label="Toggle sidebar"
                            >
                                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>

                            <div class="flex items-center space-x-3">
                                <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold">
                                    "P"
                                </div>
                                <div>
                                    <h1 class="text-lg font-semibold text-gray-900">"SIMPEL Perencanaan"</h1>
                                    <p class="text-sm text-gray-500">"Sistem Informasi Manajemen Perencanaan"</p>
                                </div>
                            </div>
                        </div>

                        // Right side - User menu
                        <div class="flex items-center space-x-4">
                            <UserMenu
                                user_name=user.name.clone()
                                user_role=user.role.clone()
                                user_avatar=user.avatar.clone()
                                menu_items=user_menu_items
                                _on_logout=on_logout
                            />
                        </div>
                    </div>
                </div>
            </header>

            // Main content area with sidebar
            <div class="flex">
                // Sidebar
                <Sidebar open=sidebar_open />

                // Main content
                <main class="flex-1 min-h-screen">
                    <div class="container mx-auto px-6 py-8">
                        {children()}
                    </div>
                </main>
            </div>

            // Footer
            <Footer />
        </div>
    }
}
