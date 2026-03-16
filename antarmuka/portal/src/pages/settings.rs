//! Settings page - User preferences and customization

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use leptos::prelude::*;
use lib_ui::components::{BrandingEditor, ThemeEditor};

/// Settings page component
#[component]
pub fn SettingsPage(user_session: UserSession, on_logout: Box<dyn Fn()>) -> impl IntoView {
    let (show_theme_editor, set_show_theme_editor) = signal(false);
    let (show_branding_editor, set_show_branding_editor) = signal(false);

    let mfa_enabled = user_session.mfa_enabled;

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="max-w-4xl mx-auto px-4 sm:px-6 py-6">
                <div class="mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Pengaturan"</h1>
                    <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Kelola preferensi dan kustomisasi tampilan"</p>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-2 gap-5">

                    // ── Tampilan ──────────────────────────────────────
                    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5">
                        <SectionHeader title="Tampilan" desc="Tema dan warna"
                            icon_path="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />

                        <div class="space-y-3 mt-4">
                            <SettingsButton
                                label="Editor Tema"
                                desc="Mode gelap/terang dan warna"
                                on_click=move |_| set_show_theme_editor.set(true)
                            />
                            <SettingsButton
                                label="Custom Branding"
                                desc="Logo dan warna unit kerja"
                                on_click=move |_| set_show_branding_editor.set(true)
                            />
                        </div>
                    </div>

                    // ── Akun ──────────────────────────────────────────
                    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5">
                        <SectionHeader title="Akun" desc="Informasi profil"
                            icon_path="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />

                        <div class="mt-4 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg space-y-3">
                            <AccountField label="NIP" value=user_session.nip.clone().unwrap_or_else(|| user_session.username.clone()) />
                            <AccountField label="Nama" value=user_session.name.clone() />
                            <AccountField label="Jabatan" value=user_session.jabatan.clone().unwrap_or_else(|| "-".into()) />
                            <AccountField label="Satuan Kerja" value=user_session.division.clone() />
                            <AccountField label="Role" value=user_session.role.display_name() />
                        </div>
                    </div>

                    // ── Keamanan ──────────────────────────────────────
                    <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5">
                        <SectionHeader title="Keamanan" desc="Autentikasi dan akses"
                            icon_path="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />

                        <div class="space-y-3 mt-4">
                            // MFA status from real session data
                            <div class="p-3 rounded-lg border border-gray-200 dark:border-gray-700 flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <div class=move || format!("w-8 h-8 rounded-lg flex items-center justify-center {}",
                                        if mfa_enabled { "bg-green-100 dark:bg-green-900/30" } else { "bg-amber-100 dark:bg-amber-900/30" }
                                    )>
                                        <svg class=move || format!("w-4 h-4 {}",
                                            if mfa_enabled { "text-green-600 dark:text-green-400" } else { "text-amber-600 dark:text-amber-400" }
                                        ) fill="currentColor" viewBox="0 0 20 20">
                                            <path fill-rule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                                        </svg>
                                    </div>
                                    <div>
                                        <p class="text-sm font-medium text-gray-900 dark:text-white">"Autentikasi Multi-Faktor"</p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">
                                            {if mfa_enabled { "MFA aktif — akun terlindungi" } else { "MFA belum aktif" }}
                                        </p>
                                    </div>
                                </div>
                                {if mfa_enabled {
                                    view! {
                                        <span class="px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-700 dark:bg-green-900/50 dark:text-green-300">"Aktif"</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <a href="/portal/mfa/setup" class="px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-700 dark:bg-blue-900/50 dark:text-blue-300 hover:bg-blue-200 transition-colors">"Aktifkan"</a>
                                    }.into_any()
                                }}
                            </div>

                            {mfa_enabled.then(|| view! {
                                <a href="/portal/mfa/backup-codes" class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group">
                                    <div class="flex items-center gap-3">
                                        <div class="w-8 h-8 bg-orange-100 dark:bg-orange-900/30 rounded-lg flex items-center justify-center">
                                            <svg class="w-4 h-4 text-orange-600 dark:text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z"/>
                                            </svg>
                                        </div>
                                        <div>
                                            <p class="text-sm font-medium text-gray-900 dark:text-white">"Kode Cadangan"</p>
                                            <p class="text-xs text-gray-500 dark:text-gray-400">"Lihat kode pemulihan MFA"</p>
                                        </div>
                                    </div>
                                    <svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </a>
                            })}

                            <a href="/portal/password" class="flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group">
                                <div class="flex items-center gap-3">
                                    <div class="w-8 h-8 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
                                        <svg class="w-4 h-4 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"/>
                                        </svg>
                                    </div>
                                    <div>
                                        <p class="text-sm font-medium text-gray-900 dark:text-white">"Ubah Password"</p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400">"Ganti password akun"</p>
                                    </div>
                                </div>
                                <svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                </svg>
                            </a>
                        </div>
                    </div>
                </div>
            </div>

            // Modals
            <Show when=move || show_theme_editor.get()>
                <ThemeEditor on_close=Callback::new(move |_| set_show_theme_editor.set(false)) />
            </Show>
            <Show when=move || show_branding_editor.get()>
                <BrandingEditor on_close=Callback::new(move |_| set_show_branding_editor.set(false)) />
            </Show>
        </MainLayout>
    }
}

#[component]
fn SectionHeader(
    title: &'static str,
    desc: &'static str,
    icon_path: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3">
            <div class="w-9 h-9 bg-blue-50 dark:bg-blue-900/20 rounded-lg flex items-center justify-center">
                <svg class="w-4.5 h-4.5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
                </svg>
            </div>
            <div>
                <h2 class="text-base font-semibold text-gray-900 dark:text-white">{title}</h2>
                <p class="text-xs text-gray-500 dark:text-gray-400">{desc}</p>
            </div>
        </div>
    }
}

#[component]
fn AccountField(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div>
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">{label}</p>
            <p class="text-sm font-semibold text-gray-900 dark:text-white mt-0.5">{value}</p>
        </div>
    }
}

#[component]
fn SettingsButton(
    label: &'static str,
    desc: &'static str,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <button
            on:click=on_click
            class="w-full flex items-center justify-between p-3 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left group"
        >
            <div>
                <p class="text-sm font-medium text-gray-900 dark:text-white">{label}</p>
                <p class="text-xs text-gray-500 dark:text-gray-400">{desc}</p>
            </div>
            <svg class="w-4 h-4 text-gray-400 group-hover:text-gray-600 dark:group-hover:text-gray-300 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
            </svg>
        </button>
    }
}
