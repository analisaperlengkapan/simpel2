//! Main navigation bar component
//!
//! Top navigation with navy/gold government branding, user menu, search, and theme switcher.
//! Follows Kejaksaan RI design system.

use crate::features::auth::{AuthService, UserSession};
use leptos::prelude::*;
use lib_ui::components::{BrandedLogo, BrandedLogoSize, GlobalSearchBar, NotificationBell};
use lib_ui::prelude::*;

/// Main navigation bar with navy/gold Kejaksaan RI branding
#[component]
pub fn Navbar(
    /// Optional user session
    #[prop(optional)]
    user_session: Option<UserSession>,
    /// Logout callback
    #[prop(optional)]
    on_logout: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    use std::rc::Rc;
    let on_logout_rc = Rc::new(on_logout);
    let (mobile_open, set_mobile_open) = signal(false);

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
        <nav class="bg-gradient-to-r from-navy-800 via-navy-700 to-navy-800 dark:from-navy-900 dark:via-navy-800 dark:to-navy-900 shadow-2xl sticky top-0 z-50" role="navigation" aria-label="Navigasi utama">
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
                                    <BrandedLogo size=BrandedLogoSize::Small class="text-navy-700".to_string() />
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
                        {user_session.as_ref().map(|_| view! {
                            <>
                                <a
                                    href="/portal/dashboard"
                                    class="flex items-center gap-2 px-4 py-2 rounded-lg text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium text-sm"
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
                                    </svg>
                                    "Dashboard"
                                </a>
                                <a
                                    href="/portal/apps"
                                    class="flex items-center gap-2 px-4 py-2 rounded-lg text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium text-sm"
                                >
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                    </svg>
                                    "Aplikasi"
                                </a>
                                // Global Search
                                <div class="flex-1 max-w-md ml-2">
                                    <GlobalSearchBar />
                                </div>
                            </>
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
                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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
                                class="p-2 rounded-lg text-navy-200 hover:text-white hover:bg-white/10 transition-all duration-200"
                                title="Pengaturan"
                                aria-label="Pengaturan"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                            </a>
                        })}

                        {user_session.clone().map(|session| view! {
                            <>
                                // User Info
                                <div class="hidden md:flex items-center space-x-3 px-3 py-1.5 rounded-lg text-white bg-white/10 border border-white/5">
                                    <div class="text-right">
                                        <p class="text-sm font-semibold leading-tight">{session.name}</p>
                                        <p class="text-xs text-gold-300">{session.role.display_name()}</p>
                                    </div>
                                    <div class="relative">
                                        <div class="w-9 h-9 bg-navy-600 rounded-full flex items-center justify-center border-2 border-gold-400/50">
                                            <svg class="w-5 h-5 text-gold-300" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                            </svg>
                                        </div>
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
                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/>
                                    </svg>
                                    <span>"Keluar"</span>
                                </button>
                            </>
                        })}

                        // Mobile Menu Toggle Button
                        <button
                            on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                            class="lg:hidden p-2 rounded-lg text-navy-200 hover:text-white hover:bg-white/10 transition-all duration-200"
                            aria-label="Buka menu navigasi"
                            aria-expanded=move || if mobile_open.get() { "true" } else { "false" }
                        >
                            <svg class=move || format!("w-6 h-6 {}", if mobile_open.get() { "hidden" } else { "" })
                                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                            <svg class=move || format!("w-6 h-6 {}", if mobile_open.get() { "" } else { "hidden" })
                                 fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>

            // Mobile Navigation Drawer
            <div class=move || format!(
                "lg:hidden border-t border-white/10 bg-navy-800 dark:bg-navy-900 transition-all duration-300 overflow-hidden {}",
                if mobile_open.get() { "max-h-screen opacity-100" } else { "max-h-0 opacity-0" }
            )>
                <div class="container mx-auto px-4 py-4 space-y-1">
                    {user_session.as_ref().map(|session| view! {
                        <>
                            // User info on mobile
                            <div class="flex items-center gap-3 px-4 py-3 bg-white/5 rounded-xl mb-3 border border-white/5">
                                <div class="w-10 h-10 bg-navy-600 rounded-full flex items-center justify-center border-2 border-gold-400/50">
                                    <svg class="w-5 h-5 text-gold-300" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd"/>
                                    </svg>
                                </div>
                                <div>
                                    <p class="text-sm font-semibold text-white">{session.name.clone()}</p>
                                    <p class="text-xs text-gold-300">{session.role.display_name()}</p>
                                </div>
                            </div>

                            // Navigation links
                            <a href="/portal/dashboard" class="flex items-center gap-3 px-4 py-3 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/>
                                </svg>
                                "Dashboard"
                            </a>
                            <a href="/portal/apps" class="flex items-center gap-3 px-4 py-3 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"/>
                                </svg>
                                "Aplikasi"
                            </a>
                            <a href="/portal/notifications" class="flex items-center gap-3 px-4 py-3 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                                </svg>
                                "Notifikasi"
                            </a>
                            <a href="/portal/settings" class="flex items-center gap-3 px-4 py-3 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200 font-medium"
                                on:click=move |_| set_mobile_open.set(false)>
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                </svg>
                                "Pengaturan"
                            </a>

                            // Account management links
                            <div class="border-t border-white/10 my-2"></div>
                            <p class="px-4 py-1 text-xs text-gold-400/70 uppercase font-semibold tracking-wider">"Akun"</p>
                            <a href="/portal/profile" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "👤 Profil"
                            </a>
                            <a href="/portal/passkeys" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🔐 Passkey"
                            </a>
                            <a href="/portal/password" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🔒 Ubah Kata Sandi"
                            </a>
                            <a href="/portal/sessions" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "📱 Sesi Aktif"
                            </a>

                            // Admin link
                            <div class="border-t border-white/10 my-2"></div>
                            <a href="/portal/admin" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-navy-100 hover:text-white hover:bg-white/10 transition-all duration-200"
                                on:click=move |_| set_mobile_open.set(false)>
                                "🛡️ Admin Panel"
                            </a>

                            <div class="border-t border-white/10 my-2"></div>

                            // Logout
                            <button
                                on:click=handle_logout_click_mobile
                                class="flex items-center gap-3 px-4 py-3 rounded-xl text-red-300 hover:text-white hover:bg-red-600/50 transition-all duration-200 font-medium w-full"
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
