//! # Header Component
//!
//! Header navigasi utama untuk Portal SIMPelv2 dengan:
//! - Responsive design (desktop & mobile)
//! - Kejaksaan RI branding
//! - Accessibility-first navigation

use leptos::prelude::*;
use shared_microfrontend::prelude::*;

/// Header utama portal dengan navigation
#[component]
pub fn Header() -> impl IntoView {
    let (mobile_menu_open, set_mobile_menu_open) = signal(false);

    view! {
        <header class="bg-white shadow-sm border-b border-kejaksaan-border">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Logo dan Brand
                    <div class="flex items-center space-x-4">
                        <img
                            src="/assets/logo-kejaksaan.png"
                            alt="Kejaksaan RI"
                            class="h-10 w-auto"
                        />
                        <div class="hidden md:block">
                            <h1 class="text-xl font-bold text-kejaksaan-text">
                                "SIMPelv2"
                            </h1>
                            <p class="text-xs text-kejaksaan-text-muted">
                                "Portal Utama"
                            </p>
                        </div>
                    </div>

                    // Desktop Navigation
                    <nav class="hidden md:flex items-center space-x-6">
                        <a href="/" class="text-kejaksaan-text hover:text-kejaksaan-primary transition-colors">
                            "Beranda"
                        </a>
                        <a href="/about" class="text-kejaksaan-text hover:text-kejaksaan-primary transition-colors">
                            "Tentang"
                        </a>
                        <a href="/help" class="text-kejaksaan-text hover:text-kejaksaan-primary transition-colors">
                            "Bantuan"
                        </a>
                    </nav>

                    // User Actions
                    <div class="flex items-center space-x-3">
                        <KejButton variant=ButtonVariant::Secondary class="hidden md:inline-flex">
                            "🔍 Search"
                        </KejButton>

                        // Mobile menu button
                        <button
                            class="md:hidden p-2 rounded-md text-kejaksaan-text hover:bg-kejaksaan-bg transition-colors"
                            on:click=move |_| set_mobile_menu_open.update(|open| *open = !*open)
                        >
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                            </svg>
                        </button>
                    </div>
                </div>

                // Mobile Navigation Menu
                <div class=move || if mobile_menu_open.get() {
                    "md:hidden py-4 border-t border-kejaksaan-border"
                } else {
                    "hidden"
                }>
                    <nav class="flex flex-col space-y-2">
                        <a href="/" class="p-2 text-kejaksaan-text hover:bg-kejaksaan-bg rounded-md transition-colors">
                            "Beranda"
                        </a>
                        <a href="/about" class="p-2 text-kejaksaan-text hover:bg-kejaksaan-bg rounded-md transition-colors">
                            "Tentang"
                        </a>
                        <a href="/help" class="p-2 text-kejaksaan-text hover:bg-kejaksaan-bg rounded-md transition-colors">
                            "Bantuan"
                        </a>
                        <div class="pt-2">
                            <KejButton variant=ButtonVariant::Secondary class="w-full">
                                "🔍 Search"
                            </KejButton>
                        </div>
                    </nav>
                </div>
            </div>
        </header>
    }
}
