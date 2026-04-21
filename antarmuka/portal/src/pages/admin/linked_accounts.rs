//! Linked Accounts & Consents Page (Admin/User)
//!
//! Manage linked social/federated accounts and OAuth2 consent grants.
//! REQ-PORTAL-021

use crate::components::layout::main_layout::MainLayout;
use crate::components::feedback::EmptyPanel;
use crate::utils::app_state::use_main_layout_session_and_logout;
use leptos::prelude::*;

/// Active tab
#[derive(Clone, Copy, PartialEq, Eq)]
enum LinkedTab {
    LinkedAccounts,
    Consents,
}

impl LinkedTab {
    fn label(&self) -> &'static str {
        match self {
            Self::LinkedAccounts => "Akun Tertaut",
            Self::Consents => "Persetujuan",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::LinkedAccounts => "🔗",
            Self::Consents => "✅",
        }
    }
}

/// Linked Accounts & Consents page
#[component]
pub fn LinkedAccountsPage() -> impl IntoView {
    let (active_tab, set_active_tab) = signal(LinkedTab::LinkedAccounts);
    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-5xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / Akun Tertaut"
                </nav>
                <h1 class="text-2xl font-bold text-gray-900 mb-6">"Akun Tertaut & Persetujuan"</h1>

                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <div class="border-b">
                        <nav class="flex px-2">
                            {[LinkedTab::LinkedAccounts, LinkedTab::Consents].iter().map(|tab| {
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
                        // === Linked Accounts ===
                        <Show when=move || active_tab.get() == LinkedTab::LinkedAccounts>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Penyedia Identitas Tertaut"</h3>
                                {[("Google", "🔵", false), ("GitHub", "⚫", false), ("Microsoft", "🟦", false), ("SAML", "🔶", false)].iter().map(|(name, icon, linked)| {
                                    view! {
                                        <div class="flex items-center justify-between px-4 py-4 border rounded-lg">
                                            <div class="flex items-center gap-3">
                                                <span class="text-xl">{*icon}</span>
                                                <div>
                                                    <p class="text-sm font-medium text-gray-900">{*name}</p>
                                                    <p class="text-xs text-gray-500">{if *linked { "Terhubung" } else { "Belum terhubung" }}</p>
                                                </div>
                                            </div>
                                            <button class={if *linked {
                                                "px-3 py-1.5 text-xs text-red-600 bg-red-50 rounded-lg hover:bg-red-100"
                                            } else {
                                                "px-3 py-1.5 text-xs text-primary-600 bg-primary-50 rounded-lg hover:bg-primary-100"
                                            }}>
                                                {if *linked { "Putuskan" } else { "Hubungkan" }}
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // === Consents ===
                        <Show when=move || active_tab.get() == LinkedTab::Consents>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Persetujuan OAuth2"</h3>
                                <p class="text-sm text-gray-500">"Aplikasi yang telah diberikan akses ke akun Anda."</p>
                                <EmptyPanel
                                    title="Belum ada persetujuan"
                                    message="Belum ada aplikasi yang diberikan akses OAuth2."
                                />
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
