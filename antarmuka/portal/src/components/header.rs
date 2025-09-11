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
        <header class="simpelv2-header">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Logo dan Brand menggunakan komponen shared
                    <Logo show_text=true />

                    // Desktop Navigation
                    <nav class="hidden md:flex items-center space-x-6">
                        <a href="/" class="nav-item text-white hover:text-yellow-300 transition-colors">
                            "Beranda"
                        </a>
                        <a href="/about" class="nav-item text-white hover:text-yellow-300 transition-colors">
                            "Tentang"
                        </a>
                        <a href="/help" class="nav-item text-white hover:text-yellow-300 transition-colors">
                            "Bantuan"
                        </a>
                    </nav>

                    // User Actions
                    <div class="flex items-center space-x-3">
                        <Button variant=ButtonVariant::Secondary class="hidden md:inline-flex bg-white/10 text-white border-white/20 hover:bg-white/20">
                            "🔍 Search"
                        </Button>

                        // Mobile menu button
                        <button
                            class="md:hidden p-2 rounded-md text-white hover:bg-white/10 transition-colors"
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
                    "md:hidden py-4 border-t border-white/20"
                } else {
                    "hidden"
                }>
                    <nav class="flex flex-col space-y-2">
                        <a href="/" class="p-2 text-white hover:bg-white/10 rounded-md transition-colors">
                            "Beranda"
                        </a>
                        <a href="/about" class="p-2 text-white hover:bg-white/10 rounded-md transition-colors">
                            "Tentang"
                        </a>
                        <a href="/help" class="p-2 text-white hover:bg-white/10 rounded-md transition-colors">
                            "Bantuan"
                        </a>
                        <div class="pt-2">
                            <Button variant=ButtonVariant::Secondary class="w-full bg-white/10 text-white border-white/20 hover:bg-white/20">
                                "🔍 Search"
                            </Button>
                        </div>
                    </nav>
                </div>
            </div>
        </header>
    }
}
