//! Secrets Page - Manage secrets
//!
//! Allows users to view, create, and manage secrets in the vault.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{create_secret, list_secrets, SecretListItem};
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
    // Signals for new secret form
    let (new_path, set_new_path) = signal(String::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_value, set_new_value) = signal(String::new());
    let (is_creating, set_is_creating) = signal(false);

    // Resource for listing secrets
    // We use a trigger signal to refresh the list after creation
    let (refresh_trigger, set_refresh_trigger) = signal(0);

    // Note: We use LocalResource because gloo_net types are not Send/Sync (WASM single threaded)
    let secrets_resource = LocalResource::new(
        move || {
            refresh_trigger.get();
            async move {
                list_secrets("").await
            }
        },
    );

    // Handle form submission
    let handle_create = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_is_creating.set(true);

        let path = new_path.get();
        let key = new_key.get();
        let value = new_value.get();

        spawn_local(async move {
            let mut data = HashMap::new();
            data.insert(key, value);

            match create_secret(&path, data).await {
                Ok(_) => {
                    // Reset form and refresh list
                    set_new_path.set(String::new());
                    set_new_key.set(String::new());
                    set_new_value.set(String::new());
                    set_refresh_trigger.update(|n| *n += 1);
                },
                Err(e) => {
                    // TODO: Show toast notification
                    leptos::logging::error!("Failed to create secret: {}", e);
                }
            }
            set_is_creating.set(false);
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-8">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Manajemen Rahasia"</h1>
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                        on:click=move |_| {
                            // Toggle modal or focus form
                        }
                    >
                        "Tambah Rahasia"
                    </button>
                </div>

                // Simple Creation Form (Inline for now)
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 mb-8">
                    <h3 class="text-lg font-semibold mb-4 text-gray-900 dark:text-white">"Buat Rahasia Baru"</h3>
                    <form on:submit=handle_create class="grid grid-cols-1 md:grid-cols-4 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                            <input
                                type="text"
                                placeholder="app/config/db"
                                prop:value=new_path
                                on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                            <input
                                type="text"
                                placeholder="password"
                                prop:value=new_key
                                on:input=move |ev| set_new_key.set(event_target_value(&ev))
                                class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                            <input
                                type="password"
                                placeholder="secret-value"
                                prop:value=new_value
                                on:input=move |ev| set_new_value.set(event_target_value(&ev))
                                class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                required
                            />
                        </div>
                        <div class="flex items-end">
                            <button
                                type="submit"
                                disabled=move || is_creating.get()
                                class="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
                            >
                                {move || if is_creating.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                        </div>
                    </form>
                </div>

                // Secrets List
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden">
                    <div class="p-6 border-b border-gray-100 dark:border-gray-700">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Daftar Rahasia"</h3>
                    </div>

                    <div class="overflow-x-auto">
                        <table class="w-full">
                            <thead class="bg-gray-50 dark:bg-gray-700">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Path"</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Metadata"</th>
                                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Aksi"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                <Suspense fallback=move || view! { <tr><td colspan="3" class="px-6 py-4 text-center">"Memuat..."</td></tr> }>
                                    {move || {
                                        secrets_resource.get().map(|result: Result<Vec<SecretListItem>, String>| match result {
                                            Ok(secrets) => {
                                                if secrets.is_empty() {
                                                    view! { <tr><td colspan="3" class="px-6 py-4 text-center text-gray-500">"Belum ada rahasia tersimpan"</td></tr> }.into_any()
                                                } else {
                                                    secrets.into_iter().map(|secret| view! {
                                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors">
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                                {secret.path}
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                                // Placeholder for metadata
                                                                "-"
                                                            </td>
                                                            <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                <button class="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300 mr-3">"Lihat"</button>
                                                                <button class="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300">"Hapus"</button>
                                                            </td>
                                                        </tr>
                                                    }).collect_view().into_any()
                                                }
                                            },
                                            Err(e) => view! { <tr><td colspan="3" class="px-6 py-4 text-center text-red-500">{format!("Error: {}", e)}</td></tr> }.into_any()
                                        })
                                    }}
                                </Suspense>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
