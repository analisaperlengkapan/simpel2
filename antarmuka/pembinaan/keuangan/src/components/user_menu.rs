//! # User Menu Component for SIMPEL Keuangan
//!
//! Komponen menu pengguna dengan dropdown untuk profil dan logout

use leptos::prelude::*;

/// User menu item structure
#[derive(Clone, Debug)]
pub struct UserMenuItem {
    pub label: String,
    pub href: Option<String>,
    pub icon: String,
}

impl UserMenuItem {
    pub fn new(label: &str, href: Option<String>, icon: &str) -> Self {
        Self {
            label: label.to_string(),
            href,
            icon: icon.to_string(),
        }
    }
}

/// User Menu Component
#[component]
pub fn UserMenu(
    /// User name to display
    user_name: String,
    /// User role/title
    user_role: String,
    /// User avatar URL (optional)
    #[prop(default = None)]
    user_avatar: Option<String>,
    /// Menu items
    menu_items: Vec<UserMenuItem>,
    /// Logout callback
    #[prop(optional)]
    _on_logout: Option<Box<dyn Fn() + Send + Sync>>,
) -> impl IntoView {
    let (is_open, set_is_open) = signal(false);

    // Default menu items if none provided
    let default_items = vec![
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

    let items = if menu_items.is_empty() {
        default_items
    } else {
        menu_items
    };

    // Clone values for display
    let user_name_display = user_name.clone();
    let user_role_display = user_role.clone();
    let user_avatar_display_main = user_avatar.clone();
    let user_avatar_display_header = user_avatar.clone();

    // Get user initial for avatar
    let user_initial = user_name
        .chars()
        .next()
        .unwrap_or('U')
        .to_uppercase()
        .to_string();

    view! {
        <div class="relative">
            // User Menu Button
            <button
                class="flex items-center space-x-3 p-2 rounded-lg hover:bg-gray-100 transition-colors duration-200"
                on:click=move |_| set_is_open.update(|open| *open = !*open)
                aria-label="Menu pengguna"
            >
                // User Avatar
                <div class="w-8 h-8 bg-green-600 rounded-full flex items-center justify-center text-white font-medium text-sm relative overflow-hidden">
                    // Always show initial as fallback
                    <span class="absolute inset-0 flex items-center justify-center">{user_initial.clone()}</span>
                    // Show image if available (positioned on top)
                    {move || {
                        user_avatar_display_main.clone().map(|avatar| {
                            view! {
                                <img
                                    src=avatar
                                    alt="Avatar"
                                    class="w-full h-full rounded-full object-cover"
                                />
                            }
                        })
                    }}
                </div>

                // User Info (hidden on mobile)
                <div class="hidden md:block text-left">
                    <div class="text-sm font-medium text-gray-900">{user_name_display.clone()}</div>
                    <div class="text-xs text-gray-500">{user_role_display.clone()}</div>
                </div>

                // Dropdown Arrow
                <svg
                    class=move || format!(
                        "w-4 h-4 text-gray-400 transition-transform duration-200 {}",
                        if is_open.get() { "rotate-180" } else { "" }
                    )
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
            </button>

            // Dropdown Menu
            <div
                class=move || format!(
                    "absolute right-0 mt-2 w-56 bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50 transition-all duration-200 {} {}",
                    if is_open.get() { "opacity-100 visible" } else { "opacity-0 invisible" },
                    if is_open.get() { "transform scale-100" } else { "transform scale-95" }
                )
            >
                // User Info Header
                <div class="px-4 py-3 border-b border-gray-200">
                    <div class="flex items-center space-x-3">
                        <div class="w-10 h-10 bg-green-600 rounded-full flex items-center justify-center text-white font-medium relative overflow-hidden">
                            // Always show initial
                            <span class="absolute inset-0 flex items-center justify-center">{user_initial.clone()}</span>
                            // Conditionally show image on top
                            {move || {
                                user_avatar_display_header.clone().map(|avatar| {
                                    view! {
                                        <img
                                            src=avatar
                                            alt="Avatar"
                                            class="w-full h-full rounded-full object-cover"
                                        />
                                    }
                                })
                            }}
                        </div>
                        <div>
                            <div class="text-sm font-medium text-gray-900">{user_name_display.clone()}</div>
                            <div class="text-xs text-gray-500">{user_role_display.clone()}</div>
                        </div>
                    </div>
                </div>

                // Menu Items
                <div class="py-1">
                    {items.into_iter().map(|item| {
                        let href = item.href.clone();
                        let label = item.label.clone();
                        let icon = item.icon.clone();

                        {move || {
                            let href_val = href.clone();
                            let label_clone = label.clone();
                            let icon_clone = icon.clone();

                            view! {
                                <div
                                    class="flex items-center px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 hover:text-gray-900 transition-colors duration-200 cursor-pointer"
                                    on:click=move |_| {
                                        set_is_open.set(false);
                                        if let Some(href_val) = href_val.clone() {
                                            if let Some(window) = web_sys::window() {
                                                let _ = window.location().set_href(&href_val);
                                            }
                                        } else {
                                            // Logout action - redirect to portal logout
                                            if let Some(window) = web_sys::window() {
                                                let _ = window.location().set_href("/portal/logout");
                                            }
                                        }
                                    }
                                >
                                    <i class=format!("{} w-4 h-4 mr-3", icon_clone)></i>
                                    {label_clone}
                                </div>
                            }
                        }}
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}
