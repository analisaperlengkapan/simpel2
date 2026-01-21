//! Secrets Page - Manage secrets
//!
//! Allows users to view, create, and delete secrets.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{
    SecretMetadata, create_secret, get_secret, list_secrets, delete_secret,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

/// Secrets page component
#[component]
pub fn SecretsPage(
    /// Current user session data
    user_session: UserSession,
    /// Callback function to handle user logout
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Resources
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let secrets_resource = LocalResource::new(move || async move {
        let _ = refresh_trigger.get(); // Depend on signal
        list_secrets().await
    });

    // Form State
    let (is_modal_open, set_is_modal_open) = signal(false);
    let (path, set_path) = signal(String::new());
    let (key, set_key) = signal(String::new());
    let (value, set_value) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (is_submitting, set_is_submitting) = signal(false);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // Actions
    let handle_create = move |_| {
        set_is_submitting.set(true);
        set_error_msg.set(None);

        spawn_local(async move {
            let mut data = HashMap::new();
            data.insert(key.get(), value.get());

            let metadata = SecretMetadata {
                description: Some(description.get()),
                tags: vec![],
                owner: None, // Default to current user
                classification: None,
            };

            match create_secret(&path.get(), data, Some(metadata)).await {
                Ok(_) => {
                    set_is_modal_open.set(false);
                    set_path.set(String::new());
                    set_key.set(String::new());
                    set_value.set(String::new());
                    set_description.set(String::new());
                    set_refresh_trigger.update(|n| *n += 1);
                }
                Err(e) => set_error_msg.set(Some(e)),
            }
            set_is_submitting.set(false);
        });
    };

    let handle_delete = move |path: String| {
        spawn_local(async move {
            if gloo_utils::window()
                .confirm_with_message(&format!("Are you sure you want to delete {}?", path))
                .unwrap_or(false)
            {
                match delete_secret(&path).await {
                    Ok(_) => set_refresh_trigger.update(|n| *n += 1),
                    Err(e) => {
                        let _ = gloo_utils::window().alert_with_message(&format!("Error: {}", e));
                    }
                }
            }
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-8">
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                            "Manajemen Rahasia"
                        </h1>
                        <p class="text-gray-600 dark:text-gray-400">
                            "Kelola data rahasia dan konfigurasi aplikasi dengan aman."
                        </p>
                    </div>
                    <button
                        class="px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg flex items-center transition-colors shadow-lg"
                        on:click=move |_| set_is_modal_open.set(true)
                    >
                        <span class="text-xl mr-2">"+"</span>
                        "Tambah Rahasia"
                    </button>
                </div>

                // Secrets List
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-100 dark:border-gray-700 overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full text-left">
                            <thead class="bg-gray-50 dark:bg-gray-900/50 text-gray-600 dark:text-gray-400 font-semibold text-sm">
                                <tr>
                                    <th class="p-4">"Path"</th>
                                    <th class="p-4">"Deskripsi"</th>
                                    <th class="p-4">"Versi"</th>
                                    <th class="p-4">"Terakhir Update"</th>
                                    <th class="p-4 text-right">"Aksi"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                <Suspense fallback=move || view! {
                                    <tr>
                                        <td colspan="5" class="p-8 text-center">
                                            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                                        </td>
                                    </tr>
                                }>
                                    {move || {
                                        match secrets_resource.get() {
                                            Some(Ok(secrets)) if !secrets.is_empty() => {
                                                secrets.into_iter().map(|secret| {
                                                    let path_del = secret.path.clone();
                                                    view! {
                                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                                                            <td class="p-4 font-medium text-gray-900 dark:text-white">
                                                                <div class="flex items-center">
                                                                    <span class="mr-2">"🔐"</span>
                                                                    {secret.path}
                                                                </div>
                                                            </td>
                                                            <td class="p-4 text-gray-600 dark:text-gray-400">
                                                                {secret.metadata.description.unwrap_or_else(|| "-".to_string())}
                                                            </td>
                                                            <td class="p-4 text-gray-600 dark:text-gray-400">
                                                                <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded text-xs font-mono">
                                                                    "v"{secret.version}
                                                                </span>
                                                            </td>
                                                            <td class="p-4 text-gray-500 text-sm">
                                                                {secret.updated_at.chars().take(10).collect::<String>()}
                                                            </td>
                                                            <td class="p-4 text-right">
                                                                <button
                                                                    class="text-red-500 hover:text-red-700 p-2 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                                                                    on:click=move |_| handle_delete(path_del.clone())
                                                                    title="Hapus"
                                                                >
                                                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                                    </svg>
                                                                </button>
                                                            </td>
                                                        </tr>
                                                        }.into_any()
                                                }).collect_view()
                                            },
                                                Some(Ok(_)) => vec![view! {
                                                <tr>
                                                    <td colspan="5" class="p-8 text-center text-gray-500">
                                                        "Belum ada data rahasia tersimpan."
                                                    </td>
                                                </tr>
                                                }.into_any()].collect_view(),
                                                Some(Err(e)) => vec![view! {
                                                <tr>
                                                    <td colspan="5" class="p-8 text-center text-red-500">
                                                        "Gagal memuat data: " {e}
                                                    </td>
                                                </tr>
                                                }.into_any()].collect_view(),
                                                None => vec![view! {
                                                 <tr>
                                                    <td colspan="5" class="p-8 text-center">
                                                        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                                                    </td>
                                                </tr>
                                                }.into_any()].collect_view(),
                                        }
                                    }}
                                </Suspense>
                            </tbody>
                        </table>
                    </div>
                </div>

                // Modal
                {move || if is_modal_open.get() {
                    Some(view! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
                            <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-xl w-full max-w-lg overflow-hidden animate-fade-in">
                                <div class="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50 dark:bg-gray-900/50">
                                    <h3 class="text-lg font-bold text-gray-900 dark:text-white">
                                        "Tambah Rahasia Baru"
                                    </h3>
                                    <button
                                        on:click=move |_| set_is_modal_open.set(false)
                                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                                    >
                                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                        </svg>
                                    </button>
                                </div>
                                <div class="p-6 space-y-4">
                                    {move || error_msg.get().map(|msg| view! {
                                        <div class="p-3 bg-red-100 text-red-700 rounded-lg text-sm">
                                            {msg}
                                        </div>
                                    })}

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Path"
                                        </label>
                                        <input
                                            type="text"
                                            class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary focus:border-transparent"
                                            placeholder="e.g. app/config/database"
                                            prop:value=path
                                            on:input=move |ev| set_path.set(event_target_value(&ev))
                                        />
                                        <p class="mt-1 text-xs text-gray-500">"Lokasi penyimpanan rahasia"</p>
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                "Key"
                                            </label>
                                            <input
                                                type="text"
                                                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary focus:border-transparent"
                                                placeholder="Key"
                                                prop:value=key
                                                on:input=move |ev| set_key.set(event_target_value(&ev))
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                "Value"
                                            </label>
                                            <input
                                                type="password"
                                                class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary focus:border-transparent"
                                                placeholder="Value"
                                                prop:value=value
                                                on:input=move |ev| set_value.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                            "Deskripsi"
                                        </label>
                                        <textarea
                                            class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-primary focus:border-transparent"
                                            rows="3"
                                            prop:value=description
                                            on:input=move |ev| set_description.set(event_target_value(&ev))
                                        ></textarea>
                                    </div>
                                </div>
                                <div class="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 flex justify-end space-x-3">
                                    <button
                                        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors"
                                        on:click=move |_| set_is_modal_open.set(false)
                                    >
                                        "Batal"
                                    </button>
                                    <button
                                        class="px-4 py-2 bg-primary hover:bg-primary-dark text-white rounded-lg transition-colors shadow-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center"
                                        on:click=handle_create
                                        prop:disabled=move || is_submitting.get()
                                    >
                                        {move || if is_submitting.get() {
                                            view! { <span class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></span> "Menyimpan..." }.into_any()
                                        } else {
                                            view! { "Simpan" }.into_any()
                                        }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    })
                } else {
                    None
                }}
            </div>
        </MainLayout>
    }
}
