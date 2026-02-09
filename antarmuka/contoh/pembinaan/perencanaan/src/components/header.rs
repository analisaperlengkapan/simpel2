//! Header Component for Pembinaan Perencanaan Microfrontend
//!
//! Header navigasi untuk sistem Perencanaan dengan Kejaksaan RI branding

use leptos::prelude::*;

/// Header untuk Pembinaan Perencanaan
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
                                "Perencanaan Strategis"
                            </h1>
                            <p class="text-sm text-kejaksaan-muted">
                                "Pembinaan & Pengembangan"
                            </p>
                        </div>
                    </div>

                    // Navigation Desktop
                    <nav class="hidden md:flex items-center space-x-6">
                        <a href="/" class="text-kejaksaan-text hover:text-kejaksaan-primary font-medium transition-colors">
                            "Dashboard"
                        </a>
                        <a href="/strategic" class="text-kejaksaan-text hover:text-kejaksaan-primary font-medium transition-colors">
                            "Rencana Strategis"
                        </a>
                        <a href="/budget" class="text-kejaksaan-text hover:text-kejaksaan-primary font-medium transition-colors">
                            "Anggaran"
                        </a>
                        <a href="/reports" class="text-kejaksaan-text hover:text-kejaksaan-primary font-medium transition-colors">
                            "Laporan"
                        </a>
                    </nav>

                    // User Menu & Mobile Toggle
                    <div class="flex items-center space-x-4">
                        // User Menu
                        <div class="hidden md:flex items-center space-x-2">
                            <span class="text-sm text-kejaksaan-muted">"Admin User"</span>
                        </div>

                        // Mobile Menu Button
                        <button
                            class="md:hidden p-2 rounded-md text-kejaksaan-text hover:bg-kejaksaan-bg"
                            on:click=move |_| set_mobile_menu_open.set(!mobile_menu_open.get())
                        >
                            <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                            </svg>
                        </button>
                    </div>
                </div>

                // Mobile Menu
                <div class=format!("md:hidden {}", if mobile_menu_open.get() { "block" } else { "hidden" })>
                    <div class="px-2 pt-2 pb-3 space-y-1 border-t border-kejaksaan-border">
                        <a href="/" class="block px-3 py-2 text-kejaksaan-text hover:text-kejaksaan-primary hover:bg-kejaksaan-bg rounded-md font-medium transition-colors">
                            "Dashboard"
                        </a>
                        <a href="/strategic" class="block px-3 py-2 text-kejaksaan-text hover:text-kejaksaan-primary hover:bg-kejaksaan-bg rounded-md font-medium transition-colors">
                            "Rencana Strategis"
                        </a>
                        <a href="/budget" class="block px-3 py-2 text-kejaksaan-text hover:text-kejaksaan-primary hover:bg-kejaksaan-bg rounded-md font-medium transition-colors">
                            "Anggaran"
                        </a>
                        <a href="/reports" class="block px-3 py-2 text-kejaksaan-text hover:text-kejaksaan-primary hover:bg-kejaksaan-bg rounded-md font-medium transition-colors">
                            "Laporan"
                        </a>
                    </div>
                </div>
            </div>
        </header>
    }
}
