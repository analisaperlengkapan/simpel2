//! Home page - Landing page for public access

use crate::components::layout::AuthLayout;
use leptos::prelude::*;

/// Home page component - public landing page
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AuthLayout>
            // Minimalist Landing Page
            // Focused on performance and clarity: single view, no scrolling, core branding only
            <div class="min-h-[85vh] flex flex-col items-center justify-center p-4">
                <div class="glass relative overflow-hidden rounded-3xl p-8 sm:p-12 md:p-16 w-full max-w-3xl text-center shadow-2xl animate-fade-in border border-white/10">

                    // Decorative subtle glow behind logo
                    <div class="absolute top-0 left-1/2 -translate-x-1/2 w-48 h-48 bg-gold-500/20 rounded-full blur-[60px] -z-10"></div>

                    // Official Kejaksaan Logo
                    <div class="relative w-32 h-32 sm:w-40 sm:h-40 mx-auto mb-8 animate-float">
                        <img
                            src="/portal/assets/kejaksaan-logo.png"
                            alt="Logo Kejaksaan Republik Indonesia"
                            class="w-full h-full object-contain drop-shadow-xl"
                        />
                    </div>

                    // Typography Hierarchy
                    <h1 class="text-4xl sm:text-5xl lg:text-5xl font-extrabold tracking-tight mb-4 text-white drop-shadow-md">
                        "Sistem Informasi"
                        <br/>
                        <span class="bg-gradient-to-r from-gold-300 via-gold-400 to-gold-500 bg-clip-text text-transparent">
                            "Manajemen Perlengkapan"
                        </span>
                    </h1>

                    <p class="text-lg sm:text-xl font-medium text-slate-300 mb-2 mt-4 tracking-wide">
                        "KEJAKSAAN REPUBLIK INDONESIA"
                    </p>

                    <p class="text-sm italic text-slate-400 mb-10 pb-4 border-b border-white/10 w-3/4 mx-auto">
                        "\"Demi Keadilan Berdasarkan Ketuhanan Yang Maha Esa\""
                    </p>

                    // Primary Call to Action
                    <div class="flex justify-center">
                        <a
                            href="/portal/login"
                            class="group relative inline-flex items-center justify-center px-10 py-4 text-lg font-bold text-navy-900 bg-gradient-to-r from-gold-400 to-gold-500 rounded-full overflow-hidden transition-all duration-300 shadow-[0_0_20px_rgba(250,204,21,0.3)] hover:shadow-[0_0_30px_rgba(250,204,21,0.5)] hover:-translate-y-1 focus:ring-4 focus:ring-gold-500/50 outline-none"
                        >
                            <span class="absolute inset-0 w-full h-full -mt-1 rounded-lg opacity-30 bg-gradient-to-b from-transparent via-transparent to-black"></span>
                            <span class="relative flex items-center gap-3">
                                <svg class="w-6 h-6 transform group-hover:scale-110 transition-transform duration-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1"/>
                                </svg>
                                "Masuk ke Aplikasi"
                            </span>
                        </a>
                    </div>
                </div>

                // Footer branding / version
                <div class="mt-12 text-center animate-fade-in">
                    <p class="text-sm font-medium text-slate-400">
                        "© 2026 Kejaksaan Republik Indonesia"
                    </p>
                    <p class="text-xs text-slate-500 mt-1">
                        "Portal SIMPEL v2.0 • Performa Tinggi"
                    </p>
                </div>
            </div>
        </AuthLayout>
    }
}
