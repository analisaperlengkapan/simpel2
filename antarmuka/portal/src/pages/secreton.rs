//! Secreton Page
//!
//! View and manage secure secrets and cryptographic keys.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{SecretListItem, list_secrets};
use leptos::prelude::*;

/// Secreton page component
#[component]
pub fn SecretonPage(user_session: UserSession, on_logout: Box<dyn Fn()>) -> impl IntoView {
    let secrets = LocalResource::new(move || async move {
        list_secrets().await.ok()
    });

    view! {
        <MainLayout user_session=user_session on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
                // Header
                <div>
                    <div class="flex items-center gap-3 mb-2">
                        <div class="w-10 h-10 rounded-xl bg-indigo-600 flex items-center justify-center shadow-lg shadow-indigo-200">
                            <span class="text-xl text-white">"🔐"</span>
                        </div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Secreton"</h1>
                    </div>
                    <p class="text-gray-500 dark:text-gray-400 max-w-2xl">
                        "Kelola informasi rahasia, kunci enkripsi, dan kebijakan keamanan dengan standar enkripsi tingkat tinggi."
                    </p>
                </div>

                // Content Grid
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    // Left Column: Navigation/Filters
                    <div class="space-y-4">
                        <div class="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 overflow-hidden shadow-sm">
                            <div class="p-4 border-b border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50">
                                <h3 class="text-xs font-bold text-gray-400 uppercase tracking-wider">"Navigasi"</h3>
                            </div>
                            <nav class="p-2 space-y-1">
                                <a href="/portal/secreton" class="flex items-center gap-3 px-4 py-2.5 rounded-xl bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 font-medium">
                                    "📦 Daftar Rahasia"
                                </a>
                                <a href="/portal/secreton/keys" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors font-medium">
                                    "🔑 Kunci Enkripsi"
                                </a>
                                <a href="/portal/secreton/policies" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors font-medium">
                                    "🛡️ Kebijakan Akses"
                                </a>
                                <a href="/portal/admin/audit" class="flex items-center gap-3 px-4 py-2.5 rounded-xl text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors font-medium">
                                    "📜 Audit Log"
                                </a>
                            </nav>
                        </div>

                        // Quick Stats
                        <div class="bg-indigo-600 rounded-2xl p-6 text-white shadow-xl shadow-indigo-200 dark:shadow-none">
                            <h3 class="text-indigo-100 text-sm font-medium mb-1">"Status Keamanan"</h3>
                            <div class="text-2xl font-bold mb-4">"Mesin Terbuka"</div>
                            <div class="space-y-3">
                                <div class="flex justify-between text-sm">
                                    <span class="text-indigo-100">"Enkripsi"</span>
                                    <span class="font-bold">"AES-256"</span>
                                </div>
                                <div class="flex justify-between text-sm">
                                    <span class="text-indigo-100">"Sertifikasi"</span>
                                    <span class="font-bold">"ISO 27001"</span>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Right Column: Main Content
                    <div class="lg:col-span-2 space-y-6">
                        <div class="flex items-center justify-between">
                            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">"Daftar Rahasia"</h2>
                            <button class="px-4 py-2 bg-navy-700 hover:bg-navy-800 text-white rounded-xl text-sm font-medium transition-colors shadow-sm">
                                "+ Tambah Rahasia"
                            </button>
                        </div>

                        <div class="bg-white dark:bg-gray-800 rounded-2xl border border-gray-200 dark:border-gray-700 shadow-sm overflow-hidden">
                            <Suspense fallback=|| view! {
                                <div class="p-8 text-center text-gray-400">
                                    <div class="animate-spin w-8 h-8 border-2 border-indigo-600 border-t-transparent rounded-full mx-auto mb-4"></div>
                                    "Memuat data rahasia..."
                                </div>
                            }>
                                {move || secrets.get().map(|s_opt| {
                                    if let Some(data) = s_opt {
                                        if data.is_empty() {
                                            view! {
                                                <div class="p-12 text-center space-y-3">
                                                    <div class="text-4xl">"📭"</div>
                                                    <div class="text-gray-900 dark:text-white font-medium">"Belum ada rahasia"</div>
                                                    <p class="text-sm text-gray-500 max-w-xs mx-auto">
                                                        "Gunakan tombol Tambah Rahasia untuk menyimpan informasi sensitif pertama Anda."
                                                    </p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <ul class="divide-y divide-gray-100 dark:divide-gray-700">
                                                    {data.into_iter().map(|item| view! {
                                                        <SecretItem item=item />
                                                    }).collect_view()}
                                                </ul>
                                            }.into_any()
                                        }
                                    } else {
                                        view! {
                                            <div class="p-12 text-center text-red-500">
                                                "Gagal memuat data rahasia. Pastikan Anda memiliki akses."
                                            </div>
                                        }.into_any()
                                    }
                                })}
                            </Suspense>
                        </div>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}

#[component]
fn SecretItem(item: SecretListItem) -> impl IntoView {
    view! {
        <li class="p-4 hover:bg-gray-50 dark:hover:bg-gray-700/30 transition-colors group">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-lg bg-gray-100 dark:bg-gray-700 flex items-center justify-center text-xl">
                        "📄"
                    </div>
                    <div>
                        <div class="text-sm font-semibold text-gray-900 dark:text-white group-hover:text-indigo-600 transition-colors">
                            {item.path}
                        </div>
                        <div class="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-2">
                            <span>"Versi " {item.version}</span>
                            <span>"•"</span>
                            <span>{item.updated_at}</span>
                        </div>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <button class="p-2 text-gray-400 hover:text-navy-700 dark:hover:text-gold-500 transition-colors">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                    </button>
                    <button class="p-2 text-gray-400 hover:text-red-600 transition-colors">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                    </button>
                </div>
            </div>
        </li>
    }
}
