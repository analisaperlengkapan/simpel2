//! Header component for Badiklat Training & Education System

use leptos::prelude::*;
use lib_ui::prelude::*;

/// Header component khusus untuk sistem Badiklat
#[component]
pub fn BadiklatHeader() -> impl IntoView {
    view! {
        <header class="bg-kejaksaan-primary text-white shadow-lg">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between py-4">
                    // Logo dan Brand
                    <div class="flex items-center space-x-4">
                        <a href="/" class="flex items-center space-x-3 hover:opacity-80 transition-opacity">
                            <div class="w-12 h-12 bg-white rounded-lg flex items-center justify-center">
                                <span class="text-2xl">"🎓"</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold">"Badiklat SIMPelv2"</h1>
                                <p class="text-sm text-kejaksaan-primary-light">"Pendidikan & Pelatihan"</p>
                            </div>
                        </a>
                    </div>

                    // Navigation Menu
                    <nav class="hidden md:flex space-x-1">
                        <NavigationLink href="/" text="Dashboard" icon="🏠" />
                        <NavigationLink href="/programs" text="Program" icon="📚" />
                        <NavigationLink href="/schedule" text="Jadwal" icon="📅" />
                        <NavigationLink href="/participants" text="Peserta" icon="👥" />
                        <NavigationLink href="/instructors" text="Instruktur" icon="👨‍🏫" />
                        <NavigationLink href="/certificates" text="Sertifikat" icon="🏆" />
                        <NavigationLink href="/reports" text="Laporan" icon="📊" />
                    </nav>

                    // Action Buttons
                    <div class="flex items-center space-x-2">
                        <Button variant=ButtonVariant::Ghost class="text-white border-white hover:bg-white hover:text-kejaksaan-primary">
                            "📋 Daftar Program"
                        </Button>
                        <Button variant=ButtonVariant::Ghost class="text-white border-white hover:bg-white hover:text-kejaksaan-primary">
                            "👤 Profile"
                        </Button>
                    </div>

                    // Mobile Menu Button
                    <button class="md:hidden p-2 hover:bg-kejaksaan-primary-dark rounded-lg transition-colors">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </button>
                </div>

                // Training Status Bar
                <div class="border-t border-kejaksaan-primary-light py-2">
                    <div class="flex items-center justify-between text-sm">
                        <div class="flex items-center space-x-6">
                            <StatusItem icon="📈" label="Program Aktif" value="5" />
                            <StatusItem icon="👥" label="Peserta" value="148" />
                            <StatusItem icon="🎯" label="Completion Rate" value="95%" />
                        </div>
                        <div class="text-kejaksaan-primary-light">
                            "Last Updated: " <span class="text-white">"09:30 WIB"</span>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    }
}

/// Component untuk navigation link
#[component]
fn NavigationLink(
    /// URL tujuan
    href: &'static str,
    /// Teks link
    text: &'static str,
    /// Icon emoji
    icon: &'static str,
) -> impl IntoView {
    view! {
        <a href=href
           class="flex items-center space-x-2 px-3 py-2 rounded-lg hover:bg-kejaksaan-primary-dark transition-colors duration-200">
            <span>{icon}</span>
            <span class="text-sm font-medium">{text}</span>
        </a>
    }
}

/// Component untuk status item di bar
#[component]
fn StatusItem(
    /// Icon emoji
    icon: &'static str,
    /// Label item
    label: &'static str,
    /// Nilai yang ditampilkan
    value: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-2">
            <span>{icon}</span>
            <span class="text-kejaksaan-primary-light">{label}":"</span>
            <span class="text-white font-medium">{value}</span>
        </div>
    }
}
