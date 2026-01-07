//! Secrets Management Page

use leptos::prelude::*;
use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{list_secrets, create_secret, SecretListItem};
use leptos::task::spawn_local;
use std::collections::HashMap;

/// Secrets page component
#[component]
pub fn SecretsPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    let (secrets, set_secrets) = signal(Vec::<SecretListItem>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(None::<String>);

    // Modal state
    let (show_modal, set_show_modal) = signal(false);
    let (new_path, set_new_path) = signal(String::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_value, set_new_value) = signal(String::new());

    // Fetch secrets
    let fetch_secrets = move || {
        set_loading.set(true);
        spawn_local(async move {
            match list_secrets(None).await {
                Ok(data) => {
                    set_secrets.set(data);
                    set_error.set(None);
                },
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    // Initial fetch
    Effect::new(move |_| {
        fetch_secrets();
    });

    // Handle create
    let handle_create = move |_| {
        let path = new_path.get();
        let key = new_key.get();
        let value = new_value.get();

        if path.is_empty() || key.is_empty() {
            return;
        }

        spawn_local(async move {
            let mut data = HashMap::new();
            data.insert(key, value);

            match create_secret(&path, data).await {
                Ok(_) => {
                    set_show_modal.set(false);
                    set_new_path.set(String::new());
                    set_new_key.set(String::new());
                    set_new_value.set(String::new());
                    fetch_secrets();
                },
                Err(e) => set_error.set(Some(e.to_string())),
            }
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Manajemen Rahasia"</h1>
                    <button
                        on:click=move |_| set_show_modal.set(true)
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                    >
                        "Tambah Rahasia"
                    </button>
                </div>

                {move || if let Some(err) = error.get() {
                    view! {
                        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
                            <span class="block sm:inline">{err}</span>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}

                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg overflow-hidden border border-gray-100 dark:border-gray-700">
                    <div class="overflow-x-auto">
                        <table class="w-full text-left">
                            <thead class="bg-gray-50 dark:bg-gray-700/50 text-gray-600 dark:text-gray-400 text-sm uppercase">
                                <tr>
                                    <th class="px-6 py-4 font-medium">"Path"</th>
                                    <th class="px-6 py-4 font-medium">"Versi"</th>
                                    <th class="px-6 py-4 font-medium">"Dibuat"</th>
                                    <th class="px-6 py-4 font-medium">"Aksi"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                {move || if loading.get() {
                                    view! {
                                        <tr>
                                            <td colspan="4" class="px-6 py-8 text-center text-gray-500">
                                                "Memuat data..."
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else if secrets.get().is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="4" class="px-6 py-8 text-center text-gray-500">
                                                "Belum ada rahasia yang tersimpan"
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    secrets.get().into_iter().map(|secret| view! {
                                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                                            <td class="px-6 py-4 text-gray-900 dark:text-white font-medium">
                                                {secret.path}
                                            </td>
                                            <td class="px-6 py-4 text-gray-600 dark:text-gray-400">
                                                {secret.version}
                                            </td>
                                            <td class="px-6 py-4 text-gray-600 dark:text-gray-400">
                                                {secret.created_at}
                                            </td>
                                            <td class="px-6 py-4">
                                                <button class="text-blue-600 hover:text-blue-800 text-sm font-medium">
                                                    "Detail"
                                                </button>
                                            </td>
                                        </tr>
                                    }).collect_view().into_any()
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>

                // Modal
                {move || if show_modal.get() {
                    view! {
                        <div class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4 z-50">
                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md p-6">
                                <h3 class="text-lg font-bold text-gray-900 dark:text-white mb-4">"Tambah Rahasia Baru"</h3>

                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                                        <input
                                            type="text"
                                            placeholder="e.g. app/config"
                                            class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 focus:ring-2 focus:ring-blue-500 outline-none"
                                            on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                            prop:value=new_path
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                        <input
                                            type="text"
                                            placeholder="Key name"
                                            class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 focus:ring-2 focus:ring-blue-500 outline-none"
                                            on:input=move |ev| set_new_key.set(event_target_value(&ev))
                                            prop:value=new_key
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                        <input
                                            type="password"
                                            placeholder="Secret value"
                                            class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 focus:ring-2 focus:ring-blue-500 outline-none"
                                            on:input=move |ev| set_new_value.set(event_target_value(&ev))
                                            prop:value=new_value
                                        />
                                    </div>
                                </div>

                                <div class="flex justify-end gap-3 mt-6">
                                    <button
                                        on:click=move |_| set_show_modal.set(false)
                                        class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                                    >
                                        "Batal"
                                    </button>
                                    <button
                                        on:click=handle_create
                                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                    >
                                        "Simpan"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}
            </div>
        </MainLayout>
    }
}
