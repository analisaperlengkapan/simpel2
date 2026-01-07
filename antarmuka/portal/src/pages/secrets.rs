//! Secrets Page - Manage Vault Secrets

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{get_secrets, create_secret, CreateSecretRequest, SecretMetadata};
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn SecretsPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Resources
    let secrets_resource = LocalResource::new(
        move || async move { get_secrets(None).await.ok().unwrap_or_default() },
    );

    // Form Signals
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (new_secret_path, set_new_secret_path) = signal(String::new());
    let (new_secret_key, set_new_secret_key) = signal(String::new());
    let (new_secret_value, set_new_secret_value) = signal(String::new());
    let (error_message, set_error_message) = signal(Option::<String>::None);

    // Handlers
    let handle_create = move |_| {
        let path = new_secret_path.get();
        let key = new_secret_key.get();
        let value = new_secret_value.get();

        if path.is_empty() || key.is_empty() || value.is_empty() {
            set_error_message.set(Some("All fields are required".to_string()));
            return;
        }

        let mut data = HashMap::new();
        data.insert(key, value);

        let req = CreateSecretRequest {
            data,
            metadata: Some(SecretMetadata {
                description: Some("Created via Portal".to_string()),
                ..Default::default()
            }),
            ttl: None,
        };

        leptos::task::spawn_local(async move {
            match create_secret(path, req).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_new_secret_path.set(String::new());
                    set_new_secret_key.set(String::new());
                    set_new_secret_value.set(String::new());
                    set_error_message.set(None);
                    secrets_resource.refetch();
                }
                Err(e) => {
                    let msg = match e {
                        crate::utils::api::ApiError::Unauthorized(_) => "Session expired. Please login again.".to_string(),
                        crate::utils::api::ApiError::Api(msg) => format!("API Error: {}", msg),
                        _ => format!("Failed to create secret: {}", e),
                    };
                    set_error_message.set(Some(msg));
                }
            }
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-8">
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">"Secrets Vault"</h1>
                        <p class="text-gray-600 dark:text-gray-400">"Securely manage your application secrets and configuration"</p>
                    </div>
                    <button
                        on:click=move |_| set_show_create_modal.set(true)
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                    >
                        <span class="text-xl">"+"</span>
                        "New Secret"
                    </button>
                </div>

                // Secrets Table
                <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-100 dark:border-gray-700 overflow-hidden">
                    <div class="overflow-x-auto">
                        <table class="w-full text-left">
                            <thead class="bg-gray-50 dark:bg-gray-700/50 text-gray-600 dark:text-gray-300 uppercase text-sm font-semibold">
                                <tr>
                                    <th class="px-6 py-4">"Path"</th>
                                    <th class="px-6 py-4">"Version"</th>
                                    <th class="px-6 py-4">"Updated At"</th>
                                    <th class="px-6 py-4 text-right">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100 dark:divide-gray-700">
                                <Suspense fallback=move || view! { <tr><td colspan="4" class="px-6 py-4 text-center">"Loading..."</td></tr> }>
                                    {move || {
                                        secrets_resource.get().map(|secrets| {
                                            if secrets.is_empty() {
                                                view! { <tr><td colspan="4" class="px-6 py-4 text-center text-gray-500">"No secrets found"</td></tr> }.into_any()
                                            } else {
                                                secrets.into_iter().map(|secret| view! {
                                                    <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                                                        <td class="px-6 py-4 font-medium text-gray-900 dark:text-white">{secret.path}</td>
                                                        <td class="px-6 py-4 text-gray-600 dark:text-gray-400">{secret.version}</td>
                                                        <td class="px-6 py-4 text-gray-600 dark:text-gray-400">{secret.updated_at}</td>
                                                        <td class="px-6 py-4 text-right">
                                                            <button class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 text-sm font-medium">"View"</button>
                                                        </td>
                                                    </tr>
                                                }).collect_view().into_any()
                                            }
                                        })
                                    }}
                                </Suspense>
                            </tbody>
                        </table>
                    </div>
                </div>

                // Create Modal
                {move || if show_create_modal.get() {
                    Some(view! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
                                <div class="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center">
                                    <h3 class="text-lg font-bold text-gray-900 dark:text-white">"Create New Secret"</h3>
                                    <button
                                        on:click=move |_| set_show_create_modal.set(false)
                                        class="text-gray-400 hover:text-gray-500"
                                    >
                                        "✕"
                                    </button>
                                </div>
                                <div class="p-6 space-y-4">
                                    {move || error_message.get().map(|msg| view! {
                                        <div class="p-3 bg-red-100 text-red-700 rounded-lg text-sm">{msg}</div>
                                    })}

                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                                        <input
                                            type="text"
                                            placeholder="e.g. app/config/db"
                                            prop:value=new_secret_path
                                            on:input=move |ev| set_new_secret_path.set(event_target_value(&ev))
                                            class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                        />
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                            <input
                                                type="text"
                                                placeholder="KEY"
                                                prop:value=new_secret_key
                                                on:input=move |ev| set_new_secret_key.set(event_target_value(&ev))
                                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                            />
                                        </div>
                                        <div>
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                            <input
                                                type="password"
                                                placeholder="VALUE"
                                                prop:value=new_secret_value
                                                on:input=move |ev| set_new_secret_value.set(event_target_value(&ev))
                                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                            />
                                        </div>
                                    </div>
                                </div>
                                <div class="px-6 py-4 bg-gray-50 dark:bg-gray-700/50 flex justify-end gap-3">
                                    <button
                                        on:click=move |_| set_show_create_modal.set(false)
                                        class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-600 rounded-lg font-medium transition-colors"
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        on:click=handle_create
                                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium transition-colors"
                                    >
                                        "Create Secret"
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
