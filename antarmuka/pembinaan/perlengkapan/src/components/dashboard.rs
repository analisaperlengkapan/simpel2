//! # SIMPEL Perlengkapan - Dashboard Layout
//!
//! Layout utama untuk dashboard perlengkapan dengan navbar, sidebar, dan footer

use crate::components::sidebar::Sidebar;
use crate::components::user_menu::{UserMenu, UserMenuItem};
use leptos::prelude::*;
use lib_ui::components::Logo;

/// User information structure
#[derive(Clone, Debug)]
pub struct User {
    pub name: String,
    pub role: String,
    pub avatar: Option<String>,
}

#[allow(dead_code)]
impl User {
    pub fn new(name: String, role: String, avatar: Option<String>) -> Self {
        Self { name, role, avatar }
    }
}

/// Dashboard layout component for SIMPEL Perlengkapan
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
            Some("/perlengkapan/profile".to_string()),
            "fas fa-user",
        ),
        UserMenuItem::new(
            "Pengaturan",
            Some("/perlengkapan/settings".to_string()),
            "fas fa-cog",
        ),
        UserMenuItem::new(
            "Bantuan",
            Some("/perlengkapan/help".to_string()),
            "fas fa-question-circle",
        ),
        UserMenuItem::new("Logout", None, "fas fa-sign-out-alt"),
    ];

    // Logout handler
    let handle_logout = || {
        // Clear local storage (both Perlengkapan and Portal tokens)
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.remove_item("auth_token");
                let _ = storage.remove_item("jwt_token");
                let _ = storage.remove_item("user_session");
            }
            // Redirect to Portal Login
            let _ = window.location().set_href("/portal/login");
        }
    };

    view! {
        <div class="min-h-screen bg-gray-50">
            // Modern Header with gradient and animations
            <header class="bg-gradient-to-r from-white via-blue-50 to-indigo-50 shadow-lg border-b border-blue-100 sticky top-0 z-50 backdrop-blur-sm bg-opacity-95">
                <div class="container mx-auto px-4 py-4">
                    <div class="flex items-center justify-between">
                        // Left side - Logo and toggle with animations
                        <div class="flex items-center space-x-4">
                            <button
                                class="group p-2.5 rounded-xl text-gray-600 hover:text-emerald-700 hover:bg-white hover:shadow-md focus:outline-none focus:ring-2 focus:ring-emerald-500 lg:hidden transform hover:scale-110 transition-all duration-200"
                                on:click=toggle_sidebar
                                aria-label="Toggle sidebar"
                            >
                                <svg class="h-6 w-6 transition-transform group-hover:rotate-180 duration-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>

                            // Enhanced Logo section with shared component
                            <div class="flex items-center space-x-3 group">
                                <div class="transform transition-all duration-300 group-hover:scale-105">
                                    <Logo show_text=true />
                                </div>
                                <div class="hidden md:flex items-center ml-3 pl-3 border-l-2 border-emerald-300">
                                    <div class="flex flex-col">
                                        <div class="flex items-center gap-2">
                                            <i class="fas fa-boxes text-emerald-600 text-sm"></i>
                                            <h1 class="text-sm font-bold bg-gradient-to-r from-emerald-700 to-blue-700 bg-clip-text text-transparent">
                                                "PERLENGKAPAN"
                                            </h1>
                                        </div>
                                        <p class="text-xs text-gray-600 font-medium">"Manajemen Aset & BMN"</p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Right side - Enhanced action buttons and User Menu
                        <div class="flex items-center space-x-2 md:space-x-3">
                            // Search button (placeholder for future)
                            <button class="hidden sm:flex items-center gap-2 px-3 py-2 rounded-lg text-gray-600 hover:text-emerald-700 hover:bg-white hover:shadow-md focus:outline-none focus:ring-2 focus:ring-emerald-500 transition-all duration-200"
                                aria-label="Search">
                                <i class="fas fa-search text-sm"></i>
                                <span class="hidden lg:inline text-sm font-medium">"Cari..."</span>
                            </button>

                            // Notifications with badge
                            <button class="relative p-2.5 rounded-lg text-gray-600 hover:text-blue-700 hover:bg-white hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all duration-200 group"
                                aria-label="Notifications">
                                <i class="fas fa-bell text-lg group-hover:animate-swing"></i>
                                <span class="absolute top-1 right-1 w-2.5 h-2.5 bg-red-500 rounded-full ring-2 ring-white animate-pulse"></span>
                            </button>

                            // Quick actions (placeholder)
                            <button class="hidden md:flex items-center gap-2 px-3 py-2 rounded-lg text-white bg-gradient-to-r from-emerald-600 to-blue-600 hover:shadow-lg focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all duration-200 hover:scale-105"
                                aria-label="Quick action">
                                <i class="fas fa-plus text-sm"></i>
                                <span class="text-sm font-medium">"Tambah"</span>
                            </button>

                            // User Menu with enhanced styling
                            <UserMenu
                                user_name=user.name.clone()
                                user_role=user.role.clone()
                                user_avatar=user.avatar.clone()
                                menu_items=user_menu_items
                                on_logout=handle_logout
                            />
                        </div>
                    </div>
                </div>
            </header>

            <div class="flex min-h-[calc(100vh-80px)]">
                // Sidebar with smooth transitions
                <Sidebar _sidebar_open=sidebar_open />

                // Main Content Area with gradient background
                <main class="flex-1 bg-gradient-to-br from-gray-50 via-blue-50/30 to-indigo-50/30 p-4 md:p-6 lg:p-8 overflow-auto">
                    <div class="max-w-7xl mx-auto">
                        {children()}
                    </div>
                </main>
            </div>

            // Enhanced Footer with gradient
            <footer class="bg-gradient-to-r from-white via-blue-50 to-indigo-50 border-t border-blue-100 py-6">
                <div class="container mx-auto px-4">
                    <div class="flex flex-col md:flex-row justify-between items-center gap-4">
                        <div class="flex items-center gap-2 text-sm text-gray-700">
                            <i class="fas fa-balance-scale text-emerald-600"></i>
                            <p class="font-medium">"© 2024 Kejaksaan Republik Indonesia"</p>
                        </div>
                        <div class="flex items-center gap-4 text-xs text-gray-600">
                            <div class="flex items-center gap-1.5 px-3 py-1.5 bg-white rounded-full shadow-sm">
                                <div class="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                                <span class="font-medium">"SIMPelv2 Perlengkapan"</span>
                                <span class="text-emerald-600 font-bold">"v1.0.0"</span>
                            </div>
                            <button class="hover:text-emerald-600 transition-colors" aria-label="Help">
                                <i class="fas fa-question-circle"></i>
                            </button>
                        </div>
                    </div>
                </div>
            </footer>
        </div>
    }
}

// Sidebar components removed as they are unused.
// Use shared components or re-implement when needed.
