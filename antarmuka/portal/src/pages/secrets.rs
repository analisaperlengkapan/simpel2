//! Secrets Management Page
//!
//! Allows users to view, create, and manage their secrets in the vault.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{fetch_api, ApiError};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// List item for secrets
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretListItem {
    /// Path to the secret
    path: String,
    /// Metadata associated with the secret
    metadata: SecretMetadata,
}

/// Metadata for a secret
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct SecretMetadata {
    /// Description of the secret
    description: Option<String>,
    /// Owner of the secret
    owner: Option<String>,
    /// Security classification
    classification: Option<String>,
}

/// Request to create a new secret
#[derive(Clone, Debug, Serialize, Deserialize)]
struct CreateSecretRequest {
    /// Secret data as key-value pairs
    data: HashMap<String, String>,
    /// Optional metadata
    metadata: Option<SecretMetadata>,
    /// Time-to-live in seconds
    ttl: Option<u64>,
}

/// Secret details response
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SecretResponse {
    /// Path to the secret
    path: String,
    /// Secret data
    data: HashMap<String, String>,
    /// Metadata
    metadata: SecretMetadata,
}

/// Generic API response wrapper
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    /// Whether the request was successful
    success: bool,
    /// Response data
    data: Option<T>,
    /// Error details if failed
    error: Option<Value>,
}

use serde_json::Value;

/// Secrets management page component
#[component]
pub fn SecretsPage(
    /// Current user session
    user_session: UserSession,
    /// Logout callback
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // State
    let (secrets, set_secrets) = signal(Vec::<SecretListItem>::new());
    let (loading, set_loading) = signal(true);
    let (error_msg, set_error_msg) = signal(Option::<String>::None);

    // View Secret State
    let (selected_secret, set_selected_secret) = signal(Option::<SecretResponse>::None);
    let (view_modal_open, set_view_modal_open) = signal(false);

    // Create Secret State
    let (create_modal_open, set_create_modal_open) = signal(false);
    let (new_path, set_new_path) = signal(String::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_value, set_new_value) = signal(String::new());
    let (new_desc, set_new_desc) = signal(String::new());

    // Load secrets on mount
    let fetch_secrets = move || {
        set_loading.set(true);
        spawn_local(async move {
            match fetch_api::<()>("GET", "/v1/secrets", None).await {
                Ok(resp) => {
                    if let Ok(body) = resp.json::<ApiResponse<Vec<SecretListItem>>>().await {
                        if body.success {
                            set_secrets.set(body.data.unwrap_or_default());
                        } else {
                            set_error_msg.set(Some("Failed to load secrets".to_string()));
                        }
                    } else {
                        set_error_msg.set(Some("Invalid response format".to_string()));
                    }
                }
                Err(e) => set_error_msg.set(Some(e.to_string())),
            }
            set_loading.set(false);
        });
    };

    Effect::new(move |_| {
        fetch_secrets();
    });

    // Handle viewing a secret
    let view_secret = move |path: String| {
        spawn_local(async move {
            let encoded_path = urlencoding::encode(&path);
            let url = format!("/v1/data/{}", encoded_path);

            match fetch_api::<()>("GET", &url, None).await {
                Ok(resp) => {
                    if let Ok(body) = resp.json::<ApiResponse<SecretResponse>>().await {
                        if body.success {
                            set_selected_secret.set(body.data);
                            set_view_modal_open.set(true);
                        } else {
                            // handle error
                        }
                    }
                }
                Err(_) => { /* handle error */ }
            }
        });
    };

    // Handle creating a secret
    let create_secret = move || {
        let path = new_path.get();
        let key = new_key.get();
        let value = new_value.get();
        let desc = new_desc.get();

        if path.is_empty() || key.is_empty() || value.is_empty() {
            return;
        }

        spawn_local(async move {
            let mut data = HashMap::new();
            data.insert(key, value);

            let metadata = SecretMetadata {
                description: Some(desc),
                owner: None,
                classification: Some("confidential".to_string()),
            };

            let req = CreateSecretRequest {
                data,
                metadata: Some(metadata),
                ttl: None,
            };

            let encoded_path = urlencoding::encode(&path);
            let url = format!("/v1/data/{}", encoded_path);

            match fetch_api("POST", &url, Some(&req)).await {
                Ok(_) => {
                    set_create_modal_open.set(false);
                    // Reset form
                    set_new_path.set(String::new());
                    set_new_key.set(String::new());
                    set_new_value.set(String::new());
                    set_new_desc.set(String::new());
                    // Refresh list
                    fetch_secrets();
                }
                Err(e) => set_error_msg.set(Some(format!("Failed to create: {}", e))),
            }
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Manajemen Rahasia"</h1>
                    <button
                        on:click=move |_| set_create_modal_open.set(true)
                        class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors flex items-center gap-2"
                    >
                        <span>"➕"</span> "Buat Rahasia Baru"
                    </button>
                </div>

                {move || if let Some(err) = error_msg.get() {
                    view! {
                        <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                            {err}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }}

                {move || if loading.get() {
                    view! { <div class="text-center py-8">"Memuat..."</div> }.into_any()
                } else {
                    view! {
                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow overflow-hidden">
                            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                <thead class="bg-gray-50 dark:bg-gray-900">
                                    <tr>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Path"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Deskripsi"</th>
                                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Pemilik"</th>
                                        <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                                    </tr>
                                </thead>
                                <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                                    {secrets.get().into_iter().map(|secret| {
                                        let path = secret.path.clone();
                                        view! {
                                            <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors">
                                                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                    {secret.path.clone()}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                    {secret.metadata.description.unwrap_or_else(|| "-".to_string())}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                                    {secret.metadata.owner.unwrap_or_else(|| "-".to_string())}
                                                </td>
                                                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                    <button
                                                        on:click=move |_| view_secret(secret.path.clone())
                                                        class="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-300 mr-4"
                                                    >
                                                        "Lihat"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                            {if secrets.get().is_empty() {
                                view! { <div class="text-center py-8 text-gray-500">"Belum ada rahasia tersimpan."</div> }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    }.into_any()
                }}
            </div>

            // Create Modal
            {move || if create_modal_open.get() {
                view! {
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                        <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
                            <h2 class="text-xl font-bold mb-4 text-gray-900 dark:text-white">"Buat Rahasia Baru"</h2>

                            <div class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path (contoh: app/db-pass)"</label>
                                    <input
                                        type="text"
                                        on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                        prop:value=new_path.get()
                                        class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                    <input
                                        type="text"
                                        on:input=move |ev| set_new_key.set(event_target_value(&ev))
                                        prop:value=new_key.get()
                                        class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                    <input
                                        type="password"
                                        on:input=move |ev| set_new_value.set(event_target_value(&ev))
                                        prop:value=new_value.get()
                                        class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Deskripsi"</label>
                                    <input
                                        type="text"
                                        on:input=move |ev| set_new_desc.set(event_target_value(&ev))
                                        prop:value=new_desc.get()
                                        class="w-full rounded-lg border-gray-300 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
                                    />
                                </div>
                            </div>

                            <div class="flex justify-end gap-3 mt-6">
                                <button
                                    on:click=move |_| set_create_modal_open.set(false)
                                    class="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg dark:text-gray-300 dark:hover:bg-gray-700"
                                >
                                    "Batal"
                                </button>
                                <button
                                    on:click=move |_| create_secret()
                                    class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
                                >
                                    "Simpan"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }}

            // View Modal
            {move || if view_modal_open.get() {
                if let Some(secret) = selected_secret.get() {
                    view! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
                            <div class="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 w-full max-w-md mx-4">
                                <h2 class="text-xl font-bold mb-4 text-gray-900 dark:text-white">"Detail Rahasia"</h2>

                                <div class="space-y-4">
                                    <div>
                                        <label class="block text-xs font-medium text-gray-500 uppercase">"Path"</label>
                                        <p class="text-gray-900 dark:text-white font-mono">{secret.path}</p>
                                    </div>

                                    <div>
                                        <label class="block text-xs font-medium text-gray-500 uppercase mb-2">"Data"</label>
                                        <div class="bg-gray-50 dark:bg-gray-900 p-3 rounded-lg space-y-2">
                                            {secret.data.iter().map(|(k, v)| {
                                                let k = k.clone();
                                                let v = v.clone();
                                                view! {
                                                    <div class="flex justify-between items-center border-b border-gray-200 dark:border-gray-700 last:border-0 pb-2 last:pb-0">
                                                        <span class="font-medium text-sm text-gray-700 dark:text-gray-300">{k}</span>
                                                        <span class="font-mono text-sm text-gray-900 dark:text-white select-all">{v}</span>
                                                    </div>
                                                }.into_any()
                                            }).collect_view()}
                                        </div>
                                    </div>
                                </div>

                                <div class="flex justify-end mt-6">
                                    <button
                                        on:click=move |_| set_view_modal_open.set(false)
                                        class="px-4 py-2 bg-gray-100 text-gray-700 hover:bg-gray-200 rounded-lg dark:bg-gray-700 dark:text-white dark:hover:bg-gray-600"
                                    >
                                        "Tutup"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            } else {
                view! { <div></div> }.into_any()
            }}
        </MainLayout>
    }
}
