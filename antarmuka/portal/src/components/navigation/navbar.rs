//! Main navigation bar component
//!
//! Top navigation with branding, user menu, and theme switcher

use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use lib_ui::components::{BrandedLogo, BrandedLogoSize, GlobalSearchBar, NotificationBell};
use lib_ui::prelude::*;

/// Main navigation bar
#[component]
pub fn Navbar(
    /// Optional user session
    #[prop(optional)]
    user_session: Option<UserSession>,
    /// Logout callback
    #[prop(optional)]
    on_logout: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    // Store logout callback
    use std::rc::Rc;
    let on_logout_rc = Rc::new(on_logout);
    let (mobile_open, set_mobile_open) = signal(false);

    let handle_logout_click = {
        let on_logout = Rc::clone(&on_logout_rc);
        move |_| {
            AuthService::logout();
            if let Some(ref callback) = *on_logout {
                callback();
            }
        }
    };

    let handle_logout_click_mobile = {
        let on_logout = Rc::clone(&on_logout_rc);
        move |_| {
            set_mobile_open.set(false);
            AuthService::logout();
            if let Some(ref callback) = *on_logout {
                callback();
            }
        }
    };

    let handle_theme_toggle = move |_| {
        toggle_theme();
    };

    view! {
        <nav class="bg-gradient-to-r from-red-600 via-red-500 to-orange-500 dark:from-red-800 dark:to-red-900 shadow-2xl sticky top-0 z-50 backdrop-blur-lg bg-opacity-95" role="navigation" aria-label="Navigasi utama">
            // Subtle top border for depth
            <div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-yellow-400 via-red-400 to-pink-400"></div>

            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Brand - Enhanced with Custom Branding
                    <div class="flex items-center space-x-4">
                        <a href="/portal" class="flex items-center space-x-3 group">
                            <div class="relative">
                                // Glow effect
                                <div class="absolute inset-0 bg-white rounded-xl blur-md opacity-50 group-hover:opacity-75 transition-opacity"></div>
                                <div class="relative bg-white p-2.5 rounded-xl shadow-lg transition-all duration-300 group-hover:scale-110 group-hover:rotate-3">
                                    <BrandedLogo size=BrandedLogoSize::Small class="text-primary".to_string() />
                                </div>
                            </div>
                            <div class="hidden md:block">
                                <h1 class="text-white text-lg font-bold tracking-tight">
                                    "Portal SIMPEL"
                                </h1>
                                <p class="text-red-100 text-xs font-medium">
                                    "Kejaksaan Agung RI"
                                </p>
                            </div>
                        </a>
                    </div>

                    // Desktop Navigation - Enhanced
                    <div class="hidden lg:flex items-center flex-1 justify-center space-x-2 mx-8">
                        {user_session.as_ref().map(|_| view! {
                            <>
                                <a
                                    href="/portal/dashboard"
                                    class="flex items-center gap-2 px-4 py-2 rounded-lg text-white hover:bg-white/20 transition-all duration-200 font-medium backdrop-blur-sm"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
                                    </svg>
                                    "Dashboard"
                                </a>
                                <a
                                    href="/portal/apps"
                                    class="flex items-center gap-2 px-4 py-2 rounded-lg text-white hover:bg-white/20 transition-all duration-200 font-medium backdrop-blur-sm"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                    </svg>
                                    "Aplikasi"
                                </a>

                                // Global Search
                                <div class="flex-1 max-w-md">
                                    <GlobalSearchBar />
                                </div>
                            </>
                        })}
                    </div>

                    // Right Side Actions - Enhanced
                    <div class="flex items-center space-x-2">
                        // Theme Toggle - Enhanced
                        <button
                            on:click=handle_theme_toggle
                            class="p-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200 backdrop-blur-sm group"
                            title="Toggle Theme"
                            aria-label="Toggle dark mode"
                        >
                            <svg class="w-5 h-5 group-hover:rotate-12 transition-transform duration-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/>
                            </svg>
                        </button>

                        // Notification Bell
                        {user_session.as_ref().map(|_| view! {
                            <NotificationBell />
                        })}

                        // Settings Link
                        {user_session.as_ref().map(|_| view! {
                            <a
                                href="/portal/settings"
                                class="p-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200 backdrop-blur-sm group"
                                title="Pengaturan"
                                aria-label="Pengaturan"
                            >
                                <svg class="w-5 h-5 group-hover:rotate-45 transition-transform duration-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                            </a>
                        })}

                        {user_session.clone().map(|session| view! {
                            <>
                                // User Info - Simplified
                                <div class="hidden md:flex items-center space-x-3 px-3 py-2 rounded-xl text-white bg-white/10 backdrop-blur-sm">
                                    <div class="text-right">
                                        <p class="text-sm font-semibold">{session.name}</p>
                                        <p class="text-xs text-red-100">{session.role.display_name()}</p>
                                    </div>
                                    <div class="relative">
                                        <div class="w-10 h-10 bg-white/30 backdrop-blur-sm rounded-full flex items-center justify-center border-2 border-white/50">
                                            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                            </svg>
                                        </div>
                                        // Online indicator
                                        <div class="absolute bottom-0 right-0 w-3 h-3 bg-green-400 border-2 border-white rounded-full"></div>
                                    </div>
                                </div>

                                // Logout Button
                                <button
                                    on:click=handle_logout_click
                                    class="hidden md:flex items-center gap-2 px-4 py-2 rounded-xl bg-red-500 hover:bg-red-600 text-white transition-all duration-200 font-medium shadow-lg hover:shadow-xl"
                                    title="Keluar dari sistem"
                                >
                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                                    </svg>
                                    <span>"Keluar"</span>
                                </button>

                            </>
                        })}

                        // Mobile Menu Toggle Button
                        <button
                            on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                            class="lg:hidden p-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200 backdrop-blur-sm"
                            aria-label="Buka menu navigasi"
                            aria-expanded=move || if mobile_open.get() { "true" } else { "false" }
                        >
                            // Hamburger icon (hidden when open)
                            <svg class=move || format!("w-6 h-6 {}", if mobile_open.get() { "hidden" } else { "" })
                                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                            // Close icon (hidden when closed)
                            <svg class=move || format!("w-6 h-6 {}", if mobile_open.get() { "" } else { "hidden" })
                                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Mobile Navigation Drawer (CSS-toggled, no <Show> to avoid Send+Sync)
            <div class=move || format!(
                "lg:hidden border-t border-white/20 bg-red-700/95 dark:bg-red-900/95 backdrop-blur-lg transition-all duration-300 overflow-hidden {}",
                if mobile_open.get() { "max-h-screen opacity-100" } else { "max-h-0 opacity-0" }
            )>
                <div class="container mx-auto px-4 py-4 space-y-2">
                    {user_session.as_ref().map(|session| view! {
                        <>
                            // User info on mobile
                            <div class="flex items-center gap-3 px-4 py-3 bg-white/10 rounded-xl mb-3">
                                <div class="w-10 h-10 bg-white/30 rounded-full flex items-center justify-center border-2 border-white/50">
                                    <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                    </svg>
                                </div>
                                <div>
                                    <p class="text-sm font-semibold text-white">{session.name.clone()}</p>
                                    <p class="text-xs text-red-100">{session.role.display_name()}</p>
                                </div>
                            </div>

                            // Navigation links
                            <a href="/portal/dashboard" class="flex items-center gap-3 px-4 py-3 rounded-xl text-white hover:bg-white/20 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
                                </svg>
                                "Dashboard"
                            </a>
                            <a href="/portal/apps" class="flex items-center gap-3 px-4 py-3 rounded-xl text-white hover:bg-white/20 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                </svg>
                                "Aplikasi"
                            </a>
                            <a href="/portal/notifications" class="flex items-center gap-3 px-4 py-3 rounded-xl text-white hover:bg-white/20 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                                </svg>
                                "Notifikasi"
                            </a>
                            <a href="/portal/settings" class="flex items-center gap-3 px-4 py-3 rounded-xl text-white hover:bg-white/20 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                                "Pengaturan"
                            </a>

                            // Account management links
                            <div class="border-t border-white/20 my-2"></div>
                            <p class="px-4 py-1 text-xs text-white/60 uppercase font-semibold">"Akun"</p>
                            <a href="/portal/profile" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "👤 Profil"
                            </a>
                            <a href="/portal/passkeys" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🔐 Passkey"
                            </a>
                            <a href="/portal/password" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🔒 Ubah Kata Sandi"
                            </a>
                            <a href="/portal/sessions" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "📱 Sesi Aktif"
                            </a>

                            // Admin link (visible to all - admin check done server-side)
                            <div class="border-t border-white/20 my-2"></div>
                            <a href="/portal/admin" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-white hover:bg-white/20 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🛡️ Admin Panel"
                            </a>

                            // Divider
                            <div class="border-t border-white/20 my-2"></div>

                            // Logout
                            <button
                                on:click=handle_logout_click_mobile
                                class="flex items-center gap-3 px-4 py-3 rounded-xl text-white hover:bg-red-800 transition-all duration-200 font-medium w-full"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                                </svg>
                                "Keluar dari Sistem"
                            </button>
                        </>
                    })}
                </div>
            </div>
        </nav>
    }
}
