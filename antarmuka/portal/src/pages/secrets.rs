//! Secrets Page - Manage Vault Secrets
//!
//! Provides a UI for listing, viewing, creating, and deleting secrets.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::fetch_api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SecretListItem {
    pub path: String,
    pub metadata: SecretMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SecretMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub owner: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SecretData {
    pub path: String,
    pub data: HashMap<String, String>,
    pub metadata: SecretMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSecretRequest {
    pub data: HashMap<String, String>,
    pub metadata: Option<SecretMetadata>,
    pub ttl: Option<u64>,
}

#[component]
pub fn SecretsPage(user_session: UserSession, on_logout: Box<dyn Fn()>) -> impl IntoView {
    let (refresh_trigger, set_refresh_trigger) = signal(0);
    let (selected_secret, set_selected_secret) = signal(None::<SecretData>);
    let (is_creating, set_is_creating) = signal(false);

    // Form state
    let (new_path, set_new_path) = signal(String::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_value, set_new_value) = signal(String::new());

    // Signal to store secrets list
    let (secrets_list, set_secrets_list) = signal(None::<Vec<SecretListItem>>);
    let (error_msg, set_error_msg) = signal(None::<String>);

    // Effect to fetch secrets
    let session_effect = user_session.clone();
    Effect::new(move |_| {
        let trigger = refresh_trigger.get(); // Depend on trigger
        let session = session_effect.clone();

        spawn_local(async move {
            match fetch_api::<(), crate::utils::api::ApiResponse<Vec<SecretListItem>>>(
                "GET",
                "/v1/secrets",
                None,
                Some(&session),
            )
            .await {
                Ok(response) => {
                    if let Some(data) = response.data {
                        set_secrets_list.set(Some(data));
                    }
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
        });

        trigger // Return trigger to satisfy Effect return if needed, usually () is fine
    });

    // Fetch details for a specific secret
    // Use StoredValue to avoid FnOnce issues in view closures
    let session_fetch = user_session.clone();
    let fetch_secret_details = StoredValue::new(move |path: String| {
        let session = session_fetch.clone();
        spawn_local(async move {
            let encoded_path = urlencoding::encode(&path);
            let result = fetch_api::<(), crate::utils::api::ApiResponse<SecretData>>(
                "GET",
                &format!("/v1/data/{}", encoded_path),
                None,
                Some(&session),
            )
            .await;

            if let Ok(response) = result {
                if let Some(data) = response.data {
                    set_selected_secret.set(Some(data));
                }
            }
        });
    });

    // Create Secret Action
    let session_create = user_session.clone();
    let create_secret = StoredValue::new(move || {
        let path = new_path.get();
        let key = new_key.get();
        let value = new_value.get();
        let session = session_create.clone();

        if path.is_empty() || key.is_empty() {
            return;
        }

        spawn_local(async move {
            let mut data = HashMap::new();
            data.insert(key, value);

            let payload = CreateSecretRequest {
                data,
                metadata: Some(SecretMetadata {
                    description: Some("Created via Portal".to_string()),
                    tags: vec!["portal".to_string()],
                    owner: Some(session.username.clone()),
                }),
                ttl: None,
            };

            let encoded_path = urlencoding::encode(&path);
            let result = fetch_api::<CreateSecretRequest, crate::utils::api::ApiResponse<SecretData>>(
                "POST",
                &format!("/v1/data/{}", encoded_path),
                Some(&payload),
                Some(&session),
            )
            .await;

            if result.is_ok() {
                set_is_creating.set(false);
                set_new_path.set(String::new());
                set_new_key.set(String::new());
                set_new_value.set(String::new());
                set_refresh_trigger.update(|n| *n += 1);
            }
        });
    });

    // Delete Secret Action
    let session_delete = user_session.clone();
    let delete_secret = StoredValue::new(move |path: String| {
        let session = session_delete.clone();
        spawn_local(async move {
            let encoded_path = urlencoding::encode(&path);
            let _ = fetch_api::<(), serde_json::Value>(
                "DELETE",
                &format!("/v1/data/{}", encoded_path),
                None,
                Some(&session),
            )
            .await;

            set_selected_secret.set(None);
            set_refresh_trigger.update(|n| *n += 1);
        });
    });

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-8">
                    <div>
                        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">"Vault Secrets"</h1>
                        <p class="text-gray-600 dark:text-gray-400">"Manage your secure credentials"</p>
                    </div>
                    <button
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                        on:click=move |_| set_is_creating.set(true)
                    >
                        <span>"➕"</span>
                        "New Secret"
                    </button>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                    // List Section
                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-100 dark:border-gray-700 overflow-hidden col-span-1">
                        <div class="p-4 border-b border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50">
                            <h2 class="font-semibold text-gray-700 dark:text-gray-200">"Available Secrets"</h2>
                        </div>
                        <div class="overflow-y-auto max-h-[600px]">
                            {move || {
                                if let Some(err) = error_msg.get() {
                                    view! { <div class="p-4 text-center text-red-500">{format!("Error: {}", err)}</div> }.into_any()
                                } else if let Some(secrets) = secrets_list.get() {
                                    if secrets.is_empty() {
                                        view! { <div class="p-4 text-center text-gray-500">"No secrets found"</div> }.into_any()
                                    } else {
                                        secrets.into_iter().map(|secret| {
                                            let path = secret.path.clone();
                                            let path_clone = secret.path.clone();
                                            view! {
                                                <div
                                                    class="p-4 hover:bg-blue-50 dark:hover:bg-blue-900/20 cursor-pointer border-b border-gray-100 dark:border-gray-700 transition-colors"
                                                                on:click=move |_| fetch_secret_details.get_value()(path_clone.clone())
                                                >
                                                    <div class="flex items-center gap-3">
                                                        <span class="text-xl">"🔐"</span>
                                                        <div class="overflow-hidden">
                                                            <p class="font-medium text-gray-900 dark:text-white truncate" title=path.clone()>{path.clone()}</p>
                                                            <p class="text-xs text-gray-500 truncate">{secret.metadata.description.unwrap_or_default()}</p>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view().into_any()
                                    }
                                } else {
                                    view! { <div class="p-4 text-center">"Loading..."</div> }.into_any()
                                }
                            }}
                        </div>
                    </div>

                    // Detail/Create Section
                    <div class="md:col-span-2">
                        {move || {
                            if is_creating.get() {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-100 dark:border-gray-700 p-6">
                                        <div class="flex justify-between items-center mb-6">
                                            <h2 class="text-xl font-bold text-gray-900 dark:text-white">"Create New Secret"</h2>
                                            <button
                                                class="text-gray-500 hover:text-gray-700"
                                                on:click=move |_| set_is_creating.set(false)
                                            >
                                                "✕"
                                            </button>
                                        </div>

                                        <div class="space-y-4">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                                                <input
                                                    type="text"
                                                    class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                    placeholder="e.g. app/prod/database"
                                                    prop:value=new_path
                                                    on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                                />
                                            </div>

                                            <div class="grid grid-cols-2 gap-4">
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                                    <input
                                                        type="text"
                                                        class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                        placeholder="Key"
                                                        prop:value=new_key
                                                        on:input=move |ev| set_new_key.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div>
                                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                                    <input
                                                        type="password"
                                                        class="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                        placeholder="Value"
                                                        prop:value=new_value
                                                        on:input=move |ev| set_new_value.set(event_target_value(&ev))
                                                    />
                                                </div>
                                            </div>

                                            <div class="pt-4 flex justify-end gap-3">
                                                <button
                                                    class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg"
                                                    on:click=move |_| set_is_creating.set(false)
                                                >
                                                    "Cancel"
                                                </button>
                                                <button
                                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 shadow-md"
                                                    on:click=move |_| create_secret.get_value()()
                                                >
                                                    "Save Secret"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else if let Some(secret) = selected_secret.get() {
                                view! {
                                    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-100 dark:border-gray-700 p-6">
                                        <div class="flex justify-between items-start mb-6">
                                            <div>
                                                <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-1">{secret.path.clone()}</h2>
                                                <div class="flex gap-2">
                                                    {secret.metadata.tags.into_iter().map(|tag| view! {
                                                        <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300 text-xs rounded-full">
                                                            {tag}
                                                        </span>
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                            <button
                                                class="text-red-600 hover:bg-red-50 p-2 rounded-lg transition-colors"
                                                title="Delete Secret"
                                                on:click=move |_| delete_secret.get_value()(secret.path.clone())
                                            >
                                                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                </svg>
                                            </button>
                                        </div>

                                        <div class="space-y-4">
                                            <div class="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-4 border border-gray-200 dark:border-gray-600">
                                                <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">"Data"</h3>
                                                <div class="space-y-2">
                                                    {secret.data.into_iter().map(|(k, v)| view! {
                                                        <div class="flex items-center justify-between py-2 border-b border-gray-200 dark:border-gray-700 last:border-0">
                                                            <span class="font-mono text-sm text-gray-600 dark:text-gray-400">{k}</span>
                                                            <div class="flex items-center gap-2">
                                                                <span class="font-mono text-sm bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded select-all">{v}</span>
                                                            </div>
                                                        </div>
                                                    }).collect_view()}
                                                </div>
                                            </div>

                                            <div class="grid grid-cols-2 gap-4 text-sm text-gray-500">
                                                <div>
                                                    <span class="block text-xs uppercase">"Owner"</span>
                                                    <span class="text-gray-900 dark:text-white">{secret.metadata.owner.unwrap_or("System".to_string())}</span>
                                                </div>
                                                <div>
                                                    <span class="block text-xs uppercase">"Description"</span>
                                                    <span class="text-gray-900 dark:text-white">{secret.metadata.description.unwrap_or("-".to_string())}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="h-full flex flex-col items-center justify-center text-gray-400 p-12 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-xl">
                                        <div class="text-6xl mb-4 opacity-50">"🔒"</div>
                                        <p class="text-lg font-medium">"Select a secret to view details"</p>
                                        <p class="text-sm">"or create a new one to get started"</p>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
