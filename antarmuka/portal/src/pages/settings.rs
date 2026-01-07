//! Settings page - User settings and preferences

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::trigger_garbage_collection;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_microfrontend::components::{BrandingEditor, ThemeEditor};

/// Settings page component - user preferences and customization
#[component]
pub fn SettingsPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // State for showing theme editor modal
    let (show_theme_editor, set_show_theme_editor) = signal(false);

    // State for showing branding editor modal
    let (show_branding_editor, set_show_branding_editor) = signal(false);

    // State for GC operation
    let (gc_status, set_gc_status) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);

    let handle_gc = {
        let token = user_session.access_token.clone();
        move |_| {
            if let Some(token) = token.clone() {
                set_is_loading.set(true);
                set_gc_status.set(None);
                spawn_local(async move {
                    match trigger_garbage_collection(&token).await {
                        Ok(result) => {
                            let msg = format!("GC Success: Operation '{}' completed in {}ms", result.operation, result.duration_ms);
                            set_gc_status.set(Some(msg));
                        }
                        Err(e) => {
                            set_gc_status.set(Some(format!("GC Failed: {}", e)));
                        }
                    }
                    set_is_loading.set(false);
                });
            } else {
                set_gc_status.set(Some("Error: No access token available".to_string()));
            }
        }
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                // Page Header
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                        "Pengaturan"
                    </h1>
                    <p class="text-gray-600 dark:text-gray-400">
                        "Kelola preferensi dan kustomisasi tampilan Anda"
                    </p>
                </div>

                // Settings Grid
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    // Appearance Settings
                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <div class="flex items-center mb-6">
                            <div class="w-12 h-12 bg-gradient-to-br from-purple-500 to-purple-600 rounded-lg flex items-center justify-center mr-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
                                </svg>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-gray-900 dark:text-white">
                                    "Tampilan"
                                </h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    "Kustomisasi tema dan warna"
                                </p>
                            </div>
                        </div>

                        <div class="space-y-4">
                            // Theme Editor Button
                            <button
                                on:click=move |_| set_show_theme_editor.set(true)
                                class="w-full p-4 bg-gradient-to-r from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 rounded-xl hover:shadow-md transition-all group border border-purple-200 dark:border-purple-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-purple-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"🎨"</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="font-semibold text-gray-900 dark:text-white">"Editor Tema"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Sesuaikan warna dan mode tema"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-purple-600 dark:text-purple-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </button>

                            // Branding Editor Button
                            <button
                                on:click=move |_| set_show_branding_editor.set(true)
                                class="w-full p-4 bg-gradient-to-r from-indigo-50 to-indigo-100 dark:from-indigo-900/20 dark:to-indigo-800/20 rounded-xl hover:shadow-md transition-all group border border-indigo-200 dark:border-indigo-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-indigo-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"🏢"</span>
                                        </div>
                                        <div class="text-left">
                                            <p class="font-semibold text-gray-900 dark:text-white">"Custom Branding"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Kustomisasi logo dan warna unit kerja"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-indigo-600 dark:text-indigo-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </button>

                            // Info Card
                            <div class="p-4 bg-blue-50 dark:bg-blue-900/20 rounded-xl border border-blue-200 dark:border-blue-800">
                                <div class="flex items-start gap-3">
                                    <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                        <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
                                    </svg>
                                    <div class="flex-1">
                                        <p class="text-sm font-medium text-blue-900 dark:text-blue-200 mb-1">
                                            "Tentang Kustomisasi"
                                        </p>
                                        <p class="text-xs text-blue-800 dark:text-blue-300">
                                            "Editor tema mengubah mode gelap/terang dan warna sistem. Custom branding memungkinkan setiap unit kerja memiliki logo dan skema warna sendiri dengan validasi aksesibilitas otomatis."
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Account Settings
                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <div class="flex items-center mb-6">
                            <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                </svg>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-gray-900 dark:text-white">
                                    "Akun"
                                </h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    "Informasi profil Anda"
                                </p>
                            </div>
                        </div>

                        <div class="space-y-4">
                            // User Info
                            <div class="p-4 bg-gray-50 dark:bg-gray-700/50 rounded-xl">
                                <div class="space-y-3">
                                    <div>
                                        <label class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                            "Nama"
                                        </label>
                                        <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">
                                            {user_session.name.clone()}
                                        </p>
                                    </div>
                                    <div>
                                        <label class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                            "Username"
                                        </label>
                                        <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">
                                            {user_session.username.clone()}
                                        </p>
                                    </div>
                                    <div>
                                        <label class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                            "Role"
                                        </label>
                                        <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">
                                            {user_session.role.display_name()}
                                        </p>
                                    </div>
                                    <div>
                                        <label class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                                            "Divisi"
                                        </label>
                                        <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">
                                            {user_session.division.clone()}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Security Settings
                    <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <div class="flex items-center mb-6">
                            <div class="w-12 h-12 bg-gradient-to-br from-red-500 to-red-600 rounded-lg flex items-center justify-center mr-4">
                                <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                </svg>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-gray-900 dark:text-white">
                                    "Keamanan"
                                </h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400">
                                    "Pengaturan keamanan akun"
                                </p>
                            </div>
                        </div>

                        <div class="space-y-4">
                            // MFA Status
                            <div class="p-4 bg-green-50 dark:bg-green-900/20 rounded-xl border border-green-200 dark:border-green-800">
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-green-500 rounded-lg flex items-center justify-center">
                                            <svg class="w-5 h-5 text-white" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
                                            </svg>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"MFA Aktif"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Autentikasi dua faktor diaktifkan"</p>
                                        </div>
                                    </div>
                                    <span class="text-2xl">"✓"</span>
                                </div>
                            </div>

                            // Backup Codes Link
                            <a
                                href="/mfa/backup-codes"
                                class="block p-4 bg-gradient-to-r from-orange-50 to-orange-100 dark:from-orange-900/20 dark:to-orange-800/20 rounded-xl hover:shadow-md transition-all group border border-orange-200 dark:border-orange-800"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <div class="w-10 h-10 bg-orange-500 rounded-lg flex items-center justify-center">
                                            <span class="text-xl">"🔑"</span>
                                        </div>
                                        <div>
                                            <p class="font-semibold text-gray-900 dark:text-white">"Kode Cadangan"</p>
                                            <p class="text-xs text-gray-600 dark:text-gray-400">"Lihat kode cadangan MFA"</p>
                                        </div>
                                    </div>
                                    <svg class="w-5 h-5 text-orange-600 dark:text-orange-400 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                                    </svg>
                                </div>
                            </a>
                        </div>
                    </div>
                </div>

                // Admin Section - GC Trigger
                {move || {
                    if user_session.role.is_admin() {
                        view! {
                            <div class="mt-8 bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border-l-4 border-red-500">
                                <h2 class="text-xl font-semibold mb-4 text-gray-800 dark:text-gray-200">"System Maintenance (Admin)"</h2>
                                <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                                    "Trigger system-wide garbage collection to clean up expired sessions and secrets."
                                </p>

                                <div class="flex items-center gap-4">
                                    <button
                                        on:click=handle_gc
                                        disabled=move || is_loading.get()
                                        class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                    >
                                        {move || if is_loading.get() { "Running..." } else { "Run Garbage Collection" }}
                                    </button>

                                    {move || gc_status.get().map(|status| {
                                        let color = if status.contains("Failed") { "text-red-600" } else { "text-green-600" };
                                        view! { <span class={format!("text-sm font-medium {}", color)}>{status}</span> }
                                    })}
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
            </div>

            // Theme Editor Modal
            <Show when=move || show_theme_editor.get()>
                <ThemeEditor on_close=Callback::new(move |_| set_show_theme_editor.set(false)) />
            </Show>

            // Branding Editor Modal
            <Show when=move || show_branding_editor.get()>
                <BrandingEditor on_close=Callback::new(move |_| set_show_branding_editor.set(false)) />
            </Show>
        </MainLayout>
    }
}
