//! App shell chrome — header, sidebar mount, footer.
//!
//! Extracted from `lib.rs` so the root component only wires routing
//! and the chrome owns its own markup. All inline styles were replaced
//! with Tailwind classes that reference the design tokens declared in
//! `tailwind.config.js`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::APP_VERSION;
use crate::components::profile_menu::ProfileMenu;
use crate::routes;

#[component]
pub fn AppHeader(#[prop(into)] on_toggle_sidebar: Callback<()>) -> impl IntoView {
    view! {
        <header class="sticky top-0 z-50 border-b border-white/[0.06] bg-surface-panel">
            <div class="flex h-14 items-center justify-between px-5">
                <div class="flex items-center gap-3">
                    <button
                        type="button"
                        on:click=move |_| on_toggle_sidebar.run(())
                        class="focus-ring rounded-md p-1 text-slate-400 hover:text-slate-100 lg:hidden"
                        aria-label="Buka menu"
                    >
                        <i class="fas fa-bars text-lg"></i>
                    </button>
                    <A
                        href=routes::path::DASHBOARD
                        attr:class="flex items-center gap-3 no-underline"
                    >
                        <img
                            src="/perlengkapan/assets/kejaksaan-logo.png"
                            alt="Kejaksaan RI"
                            class="h-8 w-8 object-contain"
                        />
                        <span class="text-lg font-extrabold tracking-tight text-white">
                            "SIMPEL"
                        </span>
                    </A>
                </div>
                <ProfileMenu />
            </div>
        </header>
    }
}

#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="border-t border-white/[0.04] bg-navy-950/90 px-5 py-2.5 lg:ml-[250px]">
            <div class="flex items-center justify-between">
                <span class="text-[0.7rem] text-slate-500">
                    "SIMPEL v" {APP_VERSION} " · Kejaksaan Agung RI"
                </span>
                <span class="text-[0.65rem] text-slate-600">
                    "© 2026 Biro Perlengkapan"
                </span>
            </div>
        </footer>
    }
}
