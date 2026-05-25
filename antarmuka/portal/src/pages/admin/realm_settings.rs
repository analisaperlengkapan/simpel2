//! Realm Settings Page (Admin)
//!
//! Comprehensive realm configuration with 9 tabs matching Keycloak's Realm
//! Settings. The **General** tab is wired to `iam_update_realm` (real API);
//! the other tabs are UI previews until the backend lands the matching
//! settings endpoints (separate epic — see plan/Phase 5 deferred items).
//! REQ-PORTAL-019

use crate::components::feedback::ErrorBanner;
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::authenc_api::{RealmInfo, UpdateRealmRequest};
use leptos::prelude::*;
use leptos::task::spawn_local;

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
    let api = use_api_client();
    let (active_tab, set_active_tab) = signal(RealmSettingsTab::General);
    let (session, on_logout) = use_main_layout_session_and_logout();

    // Realm state — we manage the *first* realm returned by the IAM admin
    // API (typically "master"). The list endpoint already supports realm
    // pagination; a realm-picker dropdown is a follow-up.
    let (realm, set_realm) = signal::<Option<RealmInfo>>(None);
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal::<Option<String>>(None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            set_loading.set(true);
            set_load_error.set(None);
            spawn_local(async move {
                match api.iam_list_realms().await {
                    Ok(realms) => {
                        // Prefer the realm explicitly named "master"; fall
                        // back to whatever the IAM API returns first so the
                        // page is still useful on greenfield deployments.
                        let chosen = realms
                            .iter()
                            .find(|r| r.name == "master")
                            .cloned()
                            .or_else(|| realms.into_iter().next());
                        set_realm.set(chosen);
                    }
                    Err(e) => set_load_error.set(Some(format!("Gagal memuat realm: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

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
                        // Honesty banner: non-General tabs are mock-up
                        // previews because the matching backend endpoints
                        // aren't shipped yet. Surfaced explicitly so admins
                        // don't think they're saving real settings.
                        <Show when=move || active_tab.get() != RealmSettingsTab::General>
                            <div class="mb-4 rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800">
                                "⚠ Pratinjau UI — tab ini belum tersambung ke backend. Hanya tab "
                                <strong>"Umum"</strong>
                                " yang menyimpan perubahan secara nyata saat ini."
                            </div>
                        </Show>

                        // === General — wired to /api/v1/iam/realms ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::General>
                            {move || load_error.get().map(|msg| view! {
                                <ErrorBanner message=msg />
                            })}
                            <Show
                                when=move || !loading.get() && realm.get().is_some()
                                fallback=move || view! {
                                    <div class="text-sm text-gray-500 py-12 text-center">
                                        {move || if loading.get() { "Memuat realm..." } else { "Realm tidak ditemukan." }}
                                    </div>
                                }
                            >
                                <GeneralTabForm realm=realm set_realm=set_realm />
                            </Show>
                        </Show>

                        // === Login ===
                        <Show when=move || active_tab.get() == RealmSettingsTab::Login>
                            <div class="space-y-4">
                                <h3 class="font-medium text-gray-900">"Konfigurasi Login"</h3>
                                {["Registrasi pengguna", "Lupa kata sandi", "Ingat saya", "Verifikasi email", "Login dengan email", "Memerlukan SSL"].iter().map(|label| {
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
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Email Pengirim"</label>
                                        <input type="email" placeholder="noreply@example.com" class="w-full px-3 py-2 border rounded-lg text-sm" />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Tampilan Pengirim"</label>
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
                                    {[("Sesi SSO Tidak Aktif", "30 menit"), ("Maksimum Sesi SSO", "10 jam"), ("Sesi Klien Tidak Aktif", "30 menit"), ("Maksimum Sesi Klien", "10 jam"), ("Sesi Offline Tidak Aktif", "30 hari"), ("Batas Waktu Login", "5 menit")].iter().map(|(label, default)| {
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
                                    {[("Algoritma Tanda Tangan Bawaan", "RS256"), ("Masa Berlaku Access Token", "5 menit"), ("Masa Berlaku Refresh Token", "30 menit"), ("Masa Berlaku ID Token", "5 menit")].iter().map(|(label, default)| {
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
                                {["Deteksi brute force", "Penguncian permanen", "Penerapan PKCE", "Kebijakan keamanan konten"].iter().map(|label| {
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
                                        <option value="en">"Bahasa Inggris"</option>
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

/// General tab form. Lifts realm state up to the parent so re-fetches
/// triggered by a successful save propagate to the rest of the page.
#[component]
fn GeneralTabForm(
    realm: ReadSignal<Option<RealmInfo>>,
    set_realm: WriteSignal<Option<RealmInfo>>,
) -> impl IntoView {
    let api = use_api_client();
    let initial = realm.get_untracked().expect("Show fallback guards None case");

    let (display_name, set_display_name) =
        signal(initial.display_name.clone().unwrap_or_default());
    let (enabled, set_enabled) = signal(initial.enabled);
    let (saving, set_saving) = signal(false);
    let (save_error, set_save_error) = signal::<Option<String>>(None);
    let (save_success, set_save_success) = signal(false);

    let realm_id = initial.id.clone();
    let realm_name = initial.name.clone();
    let created_at = initial.created_at.clone();

    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let id = realm_id.clone();
        let req = UpdateRealmRequest {
            display_name: {
                let v = display_name.get();
                if v.trim().is_empty() {
                    None
                } else {
                    Some(v)
                }
            },
            enabled: Some(enabled.get()),
        };

        set_saving.set(true);
        set_save_error.set(None);
        set_save_success.set(false);
        let api = api.clone();
        spawn_local(async move {
            match api.iam_update_realm(&id, &req).await {
                Ok(updated) => {
                    set_realm.set(Some(updated));
                    set_save_success.set(true);
                    set_saving.set(false);
                }
                Err(e) => {
                    set_save_error.set(Some(format!("Gagal menyimpan: {}", e)));
                    set_saving.set(false);
                }
            }
        });
    };

    view! {
        <form class="space-y-5" on:submit=submit>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Realm"</label>
                    <input
                        type="text"
                        value=realm_name
                        class="w-full px-3 py-2 border rounded-lg bg-gray-50 text-gray-500"
                        disabled=true
                    />
                    <p class="text-xs text-gray-400 mt-1">"Nama realm tidak dapat diubah."</p>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Tampilan"</label>
                    <input
                        type="text"
                        placeholder="Nama tampilan realm"
                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                        prop:value=move || display_name.get()
                        on:input=move |e| set_display_name.set(event_target_value(&e))
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Dibuat"</label>
                    <input
                        type="text"
                        value=created_at
                        class="w-full px-3 py-2 border rounded-lg bg-gray-50 text-gray-500"
                        disabled=true
                    />
                </div>
            </div>
            <div class="flex items-center gap-3">
                <label class="relative inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        class="sr-only peer"
                        prop:checked=move || enabled.get()
                        on:change=move |e| {
                            let input: web_sys::HtmlInputElement = event_target(&e);
                            set_enabled.set(input.checked());
                        }
                    />
                    <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full"></div>
                    <span class="ms-3 text-sm font-medium text-gray-700">"Realm Aktif"</span>
                </label>
            </div>

            {move || save_error.get().map(|msg| view! {
                <div class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                    {msg}
                </div>
            })}
            {move || save_success.get().then(|| view! {
                <div class="rounded-md border border-green-200 bg-green-50 px-3 py-2 text-sm text-green-700">
                    "Perubahan realm tersimpan."
                </div>
            })}

            <button
                type="submit"
                prop:disabled=move || saving.get()
                class="px-5 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm disabled:opacity-60 disabled:cursor-not-allowed"
            >
                {move || if saving.get() { "Menyimpan..." } else { "Simpan" }}
            </button>
        </form>
    }
}
