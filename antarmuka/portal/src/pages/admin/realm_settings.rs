//! Realm Settings Page (Admin)
//!
//! Comprehensive realm configuration with 9 tabs matching Keycloak's Realm Settings.
//! REQ-PORTAL-019

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_app_state};
use leptos::prelude::*;

/// Active tab enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum RealmSettingsTab {
    General,
    Login,
    Email,
    Themes,
    Keys,
    Sessions,
    Tokens,
    Security,
    Localization,
}

impl RealmSettingsTab {
    fn label(&self) -> &'static str {
        match self {
            Self::General => "Umum",
            Self::Login => "Login",
            Self::Email => "Email",
            Self::Themes => "Tema",
            Self::Keys => "Kunci",
            Self::Sessions => "Sesi",
            Self::Tokens => "Token",
            Self::Security => "Keamanan",
            Self::Localization => "Lokalisasi",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::General => "⚙",
            Self::Login => "🔐",
            Self::Email => "📧",
            Self::Themes => "🎨",
            Self::Keys => "🔑",
            Self::Sessions => "📱",
            Self::Tokens => "🪙",
            Self::Security => "🛡",
            Self::Localization => "🌐",
        }
    }

    fn all() -> &'static [RealmSettingsTab] {
        &[
            Self::General,
            Self::Login,
            Self::Email,
            Self::Themes,
            Self::Keys,
            Self::Sessions,
            Self::Tokens,
            Self::Security,
            Self::Localization,
        ]
    }
}

/// Realm settings page
#[component]
pub fn RealmSettingsPage() -> impl IntoView {
    let state = use_app_state();
    let (active_tab, set_active_tab) = signal(RealmSettingsTab::General);

    let on_logout = {
        let state = state;
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
                    " / Pengaturan Realm"
                </nav>
                <h1 class="text-2xl font-bold text-gray-900 mb-6">"Pengaturan Realm"</h1>

                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <div class="border-b overflow-x-auto">
                        <nav class="flex px-2">
                            {RealmSettingsTab::all().iter().map(|tab| {
                                let t = *tab;
                                view! {
                                    <button
                                        on:click=move |_| set_active_tab.set(t)
                                        class=move || if active_tab.get() == t {
                                            "flex items-center gap-1 px-3 py-3 text-xs font-medium text-primary-600 border-b-2 border-primary-600 whitespace-nowrap"
                                        } else {
                                            "flex items-center gap-1 px-3 py-3 text-xs font-medium text-gray-500 hover:text-gray-700 border-b-2 border-transparent whitespace-nowrap"
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
                        // === General ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::General>
                            <div class="space-y-5">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Realm"</label>
                                        <input type="text" value="master" class="w-full px-3 py-2 border rounded-lg bg-gray-50 text-gray-500" disabled=true />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Display Name"</label>
                                        <input type="text" placeholder="Nama tampilan realm" class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"HTML Display Name"</label>
                                        <input type="text" placeholder="<strong>Realm</strong>" class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Frontend URL"</label>
                                        <input type="url" placeholder="https://auth.example.com" class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500" />
                                    </div>
                                </div>
                                <div class="flex items-center gap-3">
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input type="checkbox" checked=true class="sr-only peer" />
                                        <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                                        <span class="ms-3 text-sm font-medium text-gray-700">"Realm Aktif"</span>
                                    </label>
                                </div>
                                <button class="px-5 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm">"Simpan"</button>
                            </div>
                        </Show>

                        // === Login ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Login>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Konfigurasi Login"</h3>
                                {["Registrasi pengguna", "Lupa password", "Remember me", "Verifikasi email", "Login dengan email", "Memerlukan SSL"].iter().map(|label| {
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
                            </div>
                        </Show>

                        // === Email ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Email>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"SMTP Server"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Host"</label>
                                        <input type="text" placeholder="smtp.example.com" class="w-full px-3 py-2 border rounded-lg text-sm" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Port"</label>
                                        <input type="number" placeholder="587" class="w-full px-3 py-2 border rounded-lg text-sm" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"From Email"</label>
                                        <input type="email" placeholder="noreply@example.com" class="w-full px-3 py-2 border rounded-lg text-sm" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"From Display Name"</label>
                                        <input type="text" placeholder="Authenc" class="w-full px-3 py-2 border rounded-lg text-sm" />
                                    </div>
                                </div>
                                <button class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm">"Uji Koneksi"</button>
                            </div>
                        </Show>

                        // === Themes ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Themes>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Tema"</h3>
                                {["Login", "Akun", "Admin", "Email"].iter().map(|label| {
                                    view! {
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 mb-1">{format!("Tema {}", label)}</label>
                                            <select class="w-full max-w-sm px-3 py-2 border rounded-lg text-sm">
                                                <option>"authenc"</option>
                                                <option>"keycloak"</option>
                                            </select>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </Show>

                        // === Keys ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Keys>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Kunci Kriptografi"</h3>
                                <div class="bg-gray-50 rounded-lg border">
                                    <table class="w-full">
                                        <thead>
                                            <tr class="text-xs text-gray-500 uppercase border-b">
                                                <th class="px-4 py-3 text-left">"Algoritma"</th>
                                                <th class="px-4 py-3 text-left">"Tipe"</th>
                                                <th class="px-4 py-3 text-left">"Kid"</th>
                                                <th class="px-4 py-3 text-left">"Prioritas"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <tr class="border-b"><td class="px-4 py-3 font-mono text-sm">"RS256"</td><td class="px-4 py-3 text-sm">"RSA"</td><td class="px-4 py-3 text-xs font-mono text-gray-500">"auto-generated"</td><td class="px-4 py-3 text-sm">"100"</td></tr>
                                            <tr><td class="px-4 py-3 font-mono text-sm">"HS256"</td><td class="px-4 py-3 text-sm">"HMAC"</td><td class="px-4 py-3 text-xs font-mono text-gray-500">"auto-generated"</td><td class="px-4 py-3 text-sm">"100"</td></tr>
                                        </tbody>
                                    </table>
                                </div>
                            </div>
                        </Show>

                        // === Sessions ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Sessions>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Pengaturan Sesi"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    {[("SSO Session Idle", "30 menit"), ("SSO Session Max", "10 jam"), ("Client Session Idle", "30 menit"), ("Client Session Max", "10 jam"), ("Offline Session Idle", "30 hari"), ("Login Timeout", "5 menit")].iter().map(|(label, default)| {
                                        view! {
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">{*label}</label>
                                                <input type="text" value=*default class="w-full px-3 py-2 border rounded-lg text-sm" />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </Show>

                        // === Tokens ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Tokens>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Pengaturan Token"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    {[("Default Signature Algorithm", "RS256"), ("Access Token Lifespan", "5 menit"), ("Refresh Token Lifespan", "30 menit"), ("ID Token Lifespan", "5 menit")].iter().map(|(label, default)| {
                                        view! {
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">{*label}</label>
                                                <input type="text" value=*default class="w-full px-3 py-2 border rounded-lg text-sm" />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </Show>

                        // === Security ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Security>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Kebijakan Keamanan"</h3>
                                {["Brute force detection", "Permanent lockout", "PKCE enforcement", "Content security policy"].iter().map(|label| {
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
                                <h3 class="font-medium text-gray-900 pt-4">"Kebijakan Password"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    {[("Panjang minimum", "8"), ("Huruf besar minimum", "1"), ("Angka minimum", "1"), ("Karakter khusus minimum", "1")].iter().map(|(label, default)| {
                                        view! {
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">{*label}</label>
                                                <input type="number" value=*default class="w-full px-3 py-2 border rounded-lg text-sm" />
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </Show>

                        // === Localization ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Localization>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Lokalisasi"</h3>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Bahasa Default"</label>
                                    <select class="w-full max-w-sm px-3 py-2 border rounded-lg text-sm">
                                        <option value="id" selected=true>"Bahasa Indonesia"</option>
                                        <option value="en">"English"</option>
                                    </select>
                                </div>
                                <div class="flex items-center justify-between py-2">
                                    <span class="text-sm text-gray-700">"Internasionalisasi aktif"</span>
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input type="checkbox" checked=true class="sr-only peer" />
                                        <div class="w-9 h-5 bg-gray-200 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:after:translate-x-full"></div>
                                    </label>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
