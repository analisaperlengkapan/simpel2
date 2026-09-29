//! Main navigation bar component
//!
//! Top navigation with navy/gold government branding, user menu, search, and theme switcher.
//! Follows Kejaksaan RI design system.

use crate::components::navigation::menu::{PortalMenuItem, resolve_menu_sections, topbar_items};
use crate::components::navigation::role_switcher::RoleSwitcher;
use crate::features::auth::{AuthService, UserSession};
use crate::features::profile::page::PegawaiAvatar;
use leptos::prelude::*;
use lib_ui::components::{
    AvatarSize, BrandedLogo, BrandedLogoSize, GlobalSearchBar, NotificationBell,
};
use lib_ui::prelude::*;

/// Main navigation bar with navy/gold Kejaksaan RI branding
#[component]
pub fn Navbar(
    /// Optional user session.
    ///
    /// Accepts a raw `Option<UserSession>` because legacy call-sites
    /// pass through a context-derived `Option`. Defaults to `None`
    /// when omitted thanks to the explicit `default` attribute.
    #[prop(default = None)]
    user_session: Option<UserSession>,
    /// Logout callback. Same `Option` rationale as `user_session`.
    #[prop(default = None)]
    on_logout: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_logout_rc = Rc::new(on_logout);
    let (mobile_open, set_mobile_open) = signal(false);
    let quick_links = topbar_items(user_session.as_ref());
    let menu_sections = resolve_menu_sections(user_session.as_ref());

    let handle_logout_click = {
        let on_logout = Rc::clone(&on_logout_rc);
        move |_| {
            if let Some(ref callback) = *on_logout {
                callback();
            } else {
                AuthService::logout();
            }
        }
    };

    let handle_logout_click_mobile = {
        let on_logout = Rc::clone(&on_logout_rc);
        move |_| {
            set_mobile_open.set(false);
            if let Some(ref callback) = *on_logout {
                callback();
            } else {
                AuthService::logout();
            }
        }
    };

    let handle_theme_toggle = move |_| {
        toggle_theme();
    };

    view! {
        <nav
            class="bg-gradient-to-r from-navy-800 via-navy-700 to-navy-800 dark:from-navy-900 dark:via-navy-800 dark:to-navy-900 shadow-2xl sticky top-0 z-50"
            role="navigation"
            aria-label="Navigasi utama"
        >
            // Gold accent top border
            <div class="absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r from-gold-400 via-gold-500 to-gold-400"></div>

            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Brand
                    <div class="flex items-center space-x-4">
                        <a href="/portal" class="flex items-center space-x-3 group">
                            <div class="relative">
                                <div class="absolute inset-0 bg-gold-400/30 rounded-xl blur-md opacity-0 group-hover:opacity-100 transition-opacity"></div>
                                <div class="relative bg-white p-2 rounded-xl shadow-lg transition-all duration-300 group-hover:scale-105">
                                    <BrandedLogo
                                        size=BrandedLogoSize::Small
                                        class="text-navy-700".to_string()
                                    />
                                </div>
                            </div>
                            <div class="hidden md:block">
                                <h1 class="text-white text-lg font-bold tracking-tight">
                                    "Portal " <span class="text-gold-400">"SIMPEL"</span>
                                </h1>
                                <p class="text-navy-200 text-xs font-medium">
                                    "Kejaksaan Agung RI"
                                </p>
                            </div>
                        </a>
                    </div>

                    // Desktop Navigation
                    <div class="hidden lg:flex items-center flex-1 justify-center space-x-1 mx-8">
                        {user_session
                            .as_ref()
                            .map(|_| {
                                view! {
                                    <>
                                        {quick_links
                                            .clone()
                                            .into_iter()
                                            .take(4)
                                            .map(|item| {
                                                let class = topbar_link_class(item.href);
                                                view! {
                                                    <a href=item.href class=class>
                                                        {item.label}
                                                    </a>
                                                }
                                            })
                                            .collect_view()} // Global Search
                                        <div class="flex-1 max-w-md ml-2">
                                            <GlobalSearchBar />
                                        </div>
                                    </>
                                }
                            })}
                    </div>

                    // Right Side Actions
                    <div class="flex items-center space-x-1.5">
                        // Theme Toggle
                        <button
                            on:click=handle_theme_toggle
                            class="p-2 rounded-lg text-navy-200 hover:text-white hover:bg-white/10 transition-all duration-200"
                            title="Toggle Theme"
                            aria-label="Toggle dark mode"
                        >
                            <svg
                                class="w-5 h-5"
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                                />
                            </svg>
                        </button>

                        // Notification Bell
                        {user_session.as_ref().map(|_| view! { <NotificationBell /> })}

                        // Settings Link
                        {user_session
                            .as_ref()
                            .map(|_| {
                                view! {
                                    <a
                                        href="/portal/settings"
                                        class="p-2 rounded-lg text-navy-200 hover:text-white hover:bg-white/10 transition-all duration-200"
                                        title="Pengaturan"
                                        aria-label="Pengaturan"
                                    >
                                        <svg
                                            class="w-5 h-5"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                                            />
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                            />
                                        </svg>
                                    </a>
                                }
                            })}

                        {user_session
                            .clone()
                            .map(|session| {
                                view! {
                                    <>
                                        // Session role activation — renders only when the
                                        // user holds several roles and the issuer enforces
                                        // a single active one.
                                        <RoleSwitcher session=session.clone() />

                                        // User Info
                                        <div class="hidden md:flex items-center space-x-3 px-3 py-1.5 rounded-lg text-white bg-white/10 border border-white/5">
                                            <div class="text-right">
                                                <p class="text-sm font-semibold leading-tight">
                                                    {session.name}
                                                </p>
                                                <p class="text-xs text-gold-300">
                                                    {session.role.display_name()}
                                                </p>
                                            </div>
                                            <div class="relative">
                                                // The employee's own photo,
                                                // from the /me query the
                                                // profile page already caches —
                                                // this mount costs no extra
                                                // request. Falls back to their
                                                // initials, and to the generic
                                                // circle before /me resolves.
                                                <PegawaiAvatar
                                                    size=AvatarSize::Medium
                                                    class="!w-9 !h-9 border-2 border-gold-400/50"
                                                />
                                                // Online indicator
                                                <div class="absolute bottom-0 right-0 w-2.5 h-2.5 bg-emerald-400 border-2 border-navy-700 rounded-full"></div>
                                            </div>
                                        </div>

                                        // Logout Button
                                        <button
                                            on:click=handle_logout_click
                                            class="hidden md:flex items-center gap-2 px-3 py-2 rounded-lg text-navy-200 hover:text-white hover:bg-red-600/80 transition-all duration-200 text-sm font-medium"
                                            title="Keluar dari sistem"
                                        >
                                            <svg
                                                class="w-4 h-4"
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                                                />
                                            </svg>
                                            <span>"Keluar"</span>
                                        </button>
                                    </>
                                }
                            })}

                        // Mobile Menu Toggle Button
                        <button
                            on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                            class="lg:hidden p-2 rounded-lg text-navy-200 hover:text-white hover:bg-white/10 transition-all duration-200"
                            aria-label="Buka menu navigasi"
                            aria-expanded=move || if mobile_open.get() { "true" } else { "false" }
                        >
                            <svg
                                class=move || {
                                    format!(
                                        "w-6 h-6 {}",
                                        if mobile_open.get() { "hidden" } else { "" },
                                    )
                                }
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M4 6h16M4 12h16M4 18h16"
                                />
                            </svg>
                            <svg
                                class=move || {
                                    format!(
                                        "w-6 h-6 {}",
                                        if mobile_open.get() { "" } else { "hidden" },
                                    )
                                }
                                fill="none"
                                stroke="currentColor"
                                viewBox="0 0 24 24"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M6 18L18 6M6 6l12 12"
                                />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Mobile Navigation Drawer
            <div class=move || {
                format!(
                    "lg:hidden border-t border-white/10 bg-navy-800 dark:bg-navy-900 transition-all duration-300 overflow-hidden {}",
                    if mobile_open.get() { "max-h-screen opacity-100" } else { "max-h-0 opacity-0" },
                )
            }>
                <div class="container mx-auto px-4 py-4 space-y-1">
                    {user_session
                        .as_ref()
                        .map(|session| {
                            view! {
                                <>
                                    // User info on mobile
                                    <div class="flex items-center gap-3 px-4 py-3 bg-white/5 rounded-xl mb-3 border border-white/5">
                                        <PegawaiAvatar
                                            size=AvatarSize::Medium
                                            class="border-2 border-gold-400/50"
                                        />
                                        <div>
                                            <p class="text-sm font-semibold text-white">
                                                {session.name.clone()}
                                            </p>
                                            <p class="text-xs text-gold-300">
                                                {session.role.display_name()}
                                            </p>
                                        </div>
                                    </div>

                                    // Navigation links
                                    {menu_sections
                                        .clone()
                                        .into_iter()
                                        .map(|section| {
                                            let section_items = section.items.clone();
                                            let mobile_signal = set_mobile_open;
                                            view! {
                                                <div class="space-y-1">
                                                    <p class="px-4 pt-2 text-xs text-gold-300/80 uppercase tracking-wider font-semibold">
                                                        {section.title}
                                                    </p>
                                                    {section_items
                                                        .into_iter()
                                                        .map(|item| {
                                                            view! {
                                                                <MobileMenuItem item=item set_mobile_open=mobile_signal />
                                                            }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            }
                                        })
                                        .collect_view()}

                                    <div class="border-t border-white/10 my-2"></div>

                                    // Logout
                                    <button
                                        on:click=handle_logout_click_mobile
                                        class="flex items-center gap-3 px-4 py-3 rounded-xl text-red-300 hover:text-white hover:bg-red-600/50 transition-all duration-200 font-medium w-full"
                                    >
                                        <svg
                                            class="w-5 h-5"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                stroke-width="2"
                                                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                                            />
                                        </svg>
                                        "Keluar dari Sistem"
                                    </button>
                                </>
                            }
                        })}
                </div>
            </div>
        </nav>
    }
}

#[component]
fn MobileMenuItem(item: PortalMenuItem, set_mobile_open: WriteSignal<bool>) -> impl IntoView {
    let has_children = !item.children.is_empty();
    let children = item.children.clone();
    let class = if current_path().starts_with(item.href) {
        "flex items-center gap-3 px-4 py-2.5 rounded-xl text-white bg-white/10 transition-all duration-200 font-medium".to_string()
    } else {
        "flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200".to_string()
    };

    view! {
        <div class="space-y-1">
            <a href=item.href class=class on:click=move |_| set_mobile_open.set(false)>
                {item.label}
            </a>

            {if has_children {
                view! {
                    <ul class="pl-3 space-y-1">
                        {children
                            .into_iter()
                            .map(|child| {
                                view! {
                                    <li>
                                        <a
                                            href=child.href
                                            class="block px-4 py-2 rounded-lg text-sm text-navy-200 hover:text-white hover:bg-white/10 transition-colors"
                                            on:click=move |_| set_mobile_open.set(false)
                                        >
                                            {child.label}
                                        </a>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                    .into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}

fn topbar_link_class(href: &'static str) -> String {
    if current_path().starts_with(href) {
        "px-4 py-2 rounded-lg text-white bg-white/15 transition-all duration-200 font-medium text-sm"
            .to_string()
    } else {
        "px-4 py-2 rounded-lg text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium text-sm"
            .to_string()
    }
}

fn current_path() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().pathname().ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        String::new()
    }
}
