//! Application launcher card component
//!
//! Card for launching microfrontend applications

use crate::features::microfrontends::{AppStatus, MicrofrontendApp};
use leptos::prelude::*;

/// Application card component
#[component]
pub fn AppCard(
    /// Application data
    app: MicrofrontendApp,
) -> impl IntoView {
    let color_classes = app.color.to_classes();
    let is_available = app.status.is_available();

    // Check if this is Pembinaan - route internally instead of opening new tab
    let _is_pembinaan = app.id == "pembinaan";
    let _app_url = app.url.clone();

    let handle_click = move |_| {
        if is_available {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if _is_pembinaan {
                        // Navigate to internal route
                        let _ = window.location().set_href("/pembinaan");
                    } else {
                        // Open app in new tab
                        let _ = window.open_with_url_and_target(&_app_url, "_blank");
                    }
                }
            }
            // Suppress unused variables when not targeting wasm32
            #[cfg(not(target_arch = "wasm32"))]
            {}
        }
    };

    let _app_url2 = app.url.clone();
    let handle_keydown = move |ev: web_sys::KeyboardEvent| {
        if is_available && (ev.key() == "Enter" || ev.key() == " ") {
            ev.prevent_default();
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    if _is_pembinaan {
                        let _ = window.location().set_href("/pembinaan");
                    } else {
                        let _ = window.open_with_url_and_target(&_app_url2, "_blank");
                    }
                }
            }
            // Suppress unused variables when not targeting wasm32
            #[cfg(not(target_arch = "wasm32"))]
            {}
        }
    };

    view! {
        <button
            on:click=handle_click
            on:keydown=handle_keydown
            disabled=!is_available
            tabindex=if is_available { "0" } else { "-1" }
            aria-label=format!("Buka aplikasi {}", app.name)
            class=format!(
                "group relative bg-gradient-to-br {} text-white rounded-2xl shadow-lg p-8 transition-all duration-300 transform hover:scale-105 hover:shadow-2xl disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 text-left w-full overflow-hidden focus:outline-none focus:ring-4 focus:ring-white/50",
                color_classes
            )
        >
            // Animated background pattern
            <div class="absolute inset-0 opacity-10 group-hover:opacity-20 transition-opacity duration-300">
                <div class="absolute inset-0" style="background-image: radial-gradient(circle at 2px 2px, white 1px, transparent 0); background-size: 30px 30px;"></div>
            </div>

            // Status badge
            {(!matches!(app.status, AppStatus::Active)).then(|| view! {
                <div class="absolute top-3 right-3 z-10">
                    <span class="inline-flex items-center gap-1 bg-white/30 backdrop-blur-md text-white text-xs font-bold px-3 py-1.5 rounded-full border border-white/20 shadow-lg">
                        {match app.status {
                            AppStatus::Beta => "🧪",
                            AppStatus::Maintenance => "🔧",
                            AppStatus::Disabled => "🚫",
                            _ => "",
                        }}
                        {app.status.badge_text()}
                    </span>
                </div>
            })}

            // Icon with animation
            <div class="relative z-10 mb-6">
                <div class="inline-flex items-center justify-center w-20 h-20 bg-white/20 backdrop-blur-sm rounded-2xl group-hover:scale-110 group-hover:rotate-3 transition-all duration-300 shadow-lg">
                    <span class="text-5xl">{app.icon.clone()}</span>
                </div>
            </div>

            // Content
            <div class="relative z-10">
                <h3 class="text-2xl font-bold mb-3 group-hover:translate-x-1 transition-transform duration-300">
                    {app.name.clone()}
                </h3>
                <p class="text-sm text-white/90 leading-relaxed mb-4 line-clamp-2">
                    {app.description.clone()}
                </p>

                // Action indicator
                <div class="flex items-center gap-2 text-sm font-medium text-white/80 group-hover:text-white transition-colors">
                    <span>"Buka Aplikasi"</span>
                    <svg
                        class="w-5 h-5 group-hover:translate-x-1 transition-transform duration-300"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M13 7l5 5m0 0l-5 5m5-5H6"
                        />
                    </svg>
                </div>
            </div>

            // Shine effect on hover
            <div class="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent transform -skew-x-12 translate-x-full group-hover:translate-x-[-200%] transition-transform duration-1000"></div>
            </div>

            // Bottom gradient overlay
            <div class="absolute bottom-0 left-0 right-0 h-24 bg-gradient-to-t from-black/20 to-transparent pointer-events-none"></div>
        </button>
    }
}
