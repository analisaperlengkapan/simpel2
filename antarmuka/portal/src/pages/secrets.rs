//! Secrets Page - Manage secrets
//!
//! Allows users to view, create, and manage secrets in the vault.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{create_secret, get_secrets, CreateSecretRequest, SecretListItem, SecretMetadata};
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::collections::HashMap;

#[component]
pub fn SecretsPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let (secrets, set_secrets) = signal(Vec::<SecretListItem>::new());
    let (loading, set_loading) = signal(true);
    let (load_error, set_load_error) = signal(Option::<String>::None);

    // Effect for listing secrets (Signal+Effect pattern to avoid Send bounds)
    Effect::new(move |_| {
        let _ = refresh_trigger.get(); // Depend on trigger
        set_loading.set(true);
        spawn_local(async move {
            match get_secrets(None).await {
                Ok(data) => {
                    set_secrets.set(data);
                    set_load_error.set(None);
                }
                Err(e) => {
                    set_load_error.set(Some(format!("Failed to load secrets: {}", e)));
                }
            }
            set_loading.set(false);
        });
    });

    // Form state
    let (is_modal_open, set_is_modal_open) = signal(false);
    let (path, set_path) = signal(String::new());
    let (secret_key, set_secret_key) = signal(String::new());
    let (secret_value, set_secret_value) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (is_submitting, set_is_submitting) = signal(false);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_submitting.set(true);
        set_error_msg.set(None);
        set_success_msg.set(None);

        let p = path.get();
        let k = secret_key.get();
        let v = secret_value.get();
        let d = description.get();

        if p.is_empty() || k.is_empty() || v.is_empty() {
            set_error_msg.set(Some("Path, Key, and Value are required".to_string()));
            set_is_submitting.set(false);
            return;
        }

        let mut data = HashMap::new();
        data.insert(k, v);

        let request = CreateSecretRequest {
            data,
            metadata: Some(SecretMetadata {
                description: if d.is_empty() { None } else { Some(d) },
                tags: vec![],
                owner: None,
                classification: None,
            }),
            ttl: None,
        };

        spawn_local(async move {
            match create_secret(&p, &request).await {
                Ok(_) => {
                    set_success_msg.set(Some("Secret created successfully!".to_string()));
                    set_path.set(String::new());
                    set_secret_key.set(String::new());
                    set_secret_value.set(String::new());
                    set_description.set(String::new());
                    set_is_modal_open.set(false);
                    set_refresh_trigger.update(|n| *n += 1);
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to create secret: {}", e)));
                }
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-6">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Secrets Management"</h1>
                        <p class="text-sm text-gray-600 dark:text-gray-400">"Manage your secure variables and credentials"</p>
                    </div>
                    <button
                        on:click=move |_| set_is_modal_open.set(true)
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                    >
                        <span>"➕"</span>
                        "New Secret"
                    </button>
                </div>

                // Feedback Messages
                {move || error_msg.get().map(|msg| view! {
                    <div class="mb-4 p-4 bg-red-100 border border-red-400 text-red-700 rounded-lg">
                        {msg}
                    </div>
                })}
                {move || success_msg.get().map(|msg| view! {
                    <div class="mb-4 p-4 bg-green-100 border border-green-400 text-green-700 rounded-lg">
                        {msg}
                    </div>
                })}

                // Secrets Table
                <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                    {move || {
                        if loading.get() {
                            view! { <div class="p-8 text-center">"Loading secrets..."</div> }.into_any()
                        } else if let Some(e) = load_error.get() {
                            view! {
                                <div class="p-8 text-center text-red-500">
                                    {e}
                                </div>
                            }.into_any()
                        } else {
                            let current_secrets = secrets.get();
                            if current_secrets.is_empty() {
                                view! {
                                    <div class="p-8 text-center text-gray-500">
                                        "No secrets found. Create one to get started!"
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                        <thead class="bg-gray-50 dark:bg-gray-700">
                                            <tr>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Path"</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Version"</th>
                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Updated"</th>
                                                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                                            {current_secrets.into_iter().map(|secret| view! {
                                                <tr>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                        {secret.path}
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                        {secret.version}
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                        {secret.updated_at}
                                                    </td>
                                                    <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                        <button class="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300 mr-3">"View"</button>
                                                        <button class="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-300">"Delete"</button>
                                                    </td>
                                                </tr>
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>

            // Create Secret Modal
            {move || if is_modal_open.get() {
                view! {
                    <div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full p-6">
                            <h2 class="text-xl font-bold mb-4 text-gray-900 dark:text-white">"Create New Secret"</h2>
                            <form on:submit=handle_submit>
                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                                    <input
                                        type="text"
                                        placeholder="e.g., app/prod/database"
                                        class="w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                        prop:value=move || path.get()
                                        on:input=move |ev| set_path.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                    <input
                                        type="text"
                                        placeholder="e.g., password"
                                        class="w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                        prop:value=move || secret_key.get()
                                        on:input=move |ev| set_secret_key.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="mb-4">
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                    <input
                                        type="password"
                                        placeholder="Secret value..."
                                        class="w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                        prop:value=move || secret_value.get()
                                        on:input=move |ev| set_secret_value.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="mb-6">
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Description (Optional)"</label>
                                    <input
                                        type="text"
                                        placeholder="What is this secret for?"
                                        class="w-full px-3 py-2 border rounded-md dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                        prop:value=move || description.get()
                                        on:input=move |ev| set_description.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="flex justify-end gap-3">
                                    <button
                                        type="button"
                                        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-md"
                                        on:click=move |_| set_is_modal_open.set(false)
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        type="submit"
                                        disabled=move || is_submitting.get()
                                        class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50"
                                    >
                                        {move || if is_submitting.get() { "Saving..." } else { "Create" }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div class="hidden"></div> }.into_any()
            }}
        </MainLayout>
    }
}
