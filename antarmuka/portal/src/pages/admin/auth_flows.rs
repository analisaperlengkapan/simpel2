//! Authentication Flows & Events Configuration Page (Admin)
//!
//! Manage authentication flows, required actions, and event listeners.
//! REQ-PORTAL-020

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_app_state};
use leptos::prelude::*;

/// Active tab
#[derive(Clone, Copy, PartialEq, Eq)]
enum AuthFlowTab {
    Flows,
    RequiredActions,
    Policies,
    Events,
    EventListeners,
}

impl AuthFlowTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Flows => "Alur Autentikasi",
            Self::RequiredActions => "Aksi Wajib",
            Self::Policies => "Kebijakan",
            Self::Events => "Event",
            Self::EventListeners => "Listener",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Flows => "🔀",
            Self::RequiredActions => "⚡",
            Self::Policies => "📜",
            Self::Events => "📊",
            Self::EventListeners => "📡",
        }
    }

    fn all() -> &'static [AuthFlowTab] {
        &[Self::Flows, Self::RequiredActions, Self::Policies, Self::Events, Self::EventListeners]
    }
}

/// Auth Flows & Events page
#[component]
pub fn AuthFlowsPage() -> impl IntoView {
    let state = use_app_state();
    let (active_tab, set_active_tab) = signal(AuthFlowTab::Flows);

    let on_logout = {
        let state = state.clone();
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / Autentikasi"
                </nav>
                <h1 class="text-2xl font-bold text-gray-900 mb-6">"Autentikasi & Event"</h1>

                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <div class="border-b">
                        <nav class="flex overflow-x-auto px-2">
                            {AuthFlowTab::all().iter().map(|tab| {
                                let t = *tab;
                                view! {
                                    <button
                                        on:click=move |_| set_active_tab.set(t)
                                        class=move || if active_tab.get() == t {
                                            "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600 whitespace-nowrap"
                                        } else {
                                            "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700 border-b-2 border-transparent whitespace-nowrap"
                                        }
                                    >
                                        <span>{t.icon()}</span>
                                        {t.label()}
                                    </button>
                                }
                            }).collect::<Vec<_>>()}
                        </nav>
                    </div>

                    <div class="p-6">
                        // === Flows ===
                        <Show when=move || active_tab.get() == AuthFlowTab::Flows>
                            <div class="space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="font-medium text-gray-900">"Alur Autentikasi"</h3>
                                    <button class="px-3 py-1.5 text-sm bg-primary-600 text-white rounded-lg hover:bg-primary-700">"＋ Buat Alur"</button>
                                </div>
                                {["Browser Flow", "Direct Grant Flow", "Registration Flow", "Reset Credentials Flow", "Client Authentication Flow"].iter().map(|name| {
                                    view! {
                                        <div class="flex items-center justify-between px-4 py-3 bg-gray-50 rounded-lg border">
                                            <div class="flex items-center gap-3">
                                                <span class="text-lg">"🔀"</span>
                                                <div>
                                                    <p class="text-sm font-medium text-gray-900">{*name}</p>
                                                    <p class="text-xs text-gray-500">"Built-in flow"</p>
                                                </div>
                                            </div>
                                            <span class="text-xs px-2 py-0.5 bg-blue-100 text-blue-700 rounded-full">"Built-in"</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // === Required Actions ===
                        <Show when=move || active_tab.get() == AuthFlowTab::RequiredActions>
                            <div class="space-y-3">
                                <h3 class="font-medium text-gray-900">"Aksi Wajib"</h3>
                                {[("Verify Email", true), ("Update Password", true), ("Configure OTP", false), ("Update Profile", false), ("Terms and Conditions", false)].iter().map(|(name, enabled)| {
                                    view! {
                                        <div class="flex items-center justify-between px-4 py-3 border rounded-lg">
                                            <span class="text-sm text-gray-700">{*name}</span>
                                            <span class={if *enabled { "text-xs px-2 py-0.5 bg-green-100 text-green-700 rounded-full" } else { "text-xs px-2 py-0.5 bg-gray-100 text-gray-500 rounded-full" }}>
                                                {if *enabled { "Aktif" } else { "Nonaktif" }}
                                            </span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // === Policies ===
                        <Show when=move || active_tab.get() == AuthFlowTab::Policies>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Kebijakan Autentikasi"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"OTP Policy Type"</label>
                                        <select class="w-full px-3 py-2 border rounded-lg text-sm">
                                            <option>"totp"</option>
                                            <option>"hotp"</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"OTP Hash Algorithm"</label>
                                        <select class="w-full px-3 py-2 border rounded-lg text-sm">
                                            <option>"SHA1"</option>
                                            <option>"SHA256"</option>
                                            <option>"SHA512"</option>
                                        </select>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // === Events ===
                        <Show when=move || active_tab.get() == AuthFlowTab::Events>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Konfigurasi Event"</h3>
                                {["Login Events", "Admin Events", "Save Events"].iter().map(|label| {
                                    view! {
                                        <div class="flex items-center justify-between py-2 border-b">
                                            <span class="text-sm text-gray-700">{*label}</span>
                                            <label class="relative inline-flex items-center cursor-pointer">
                                                <input type="checkbox" class="sr-only peer" />
                                                <div class="w-9 h-5 bg-gray-200 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
                                            </label>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Event Expiration (hari)"</label>
                                    <input type="number" value="90" class="w-full max-w-xs px-3 py-2 border rounded-lg text-sm" />
                                </div>
                            </div>
                        </Show>

                        // === Event Listeners ===
                        <Show when=move || active_tab.get() == AuthFlowTab::EventListeners>
                            <div class="space-y-3">
                                <h3 class="font-medium text-gray-900">"Event Listeners"</h3>
                                {["email", "jboss-logging"].iter().map(|name| {
                                    view! {
                                        <div class="flex items-center justify-between px-4 py-3 border rounded-lg">
                                            <div class="flex items-center gap-2">
                                                <span>"📡"</span>
                                                <span class="text-sm font-medium text-gray-700">{*name}</span>
                                            </div>
                                            <span class="text-xs px-2 py-0.5 bg-green-100 text-green-700 rounded-full">"Aktif"</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
