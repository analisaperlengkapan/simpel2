//! Secrets Management Page
//! Allows users to view, create, and manage secrets.

use crate::components::layout::MainLayout;
use crate::features::auth::UserSession;
use crate::utils::api::{create_secret, list_secrets};
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn SecretsPage(
    user_session: UserSession,
    on_logout: Box<dyn Fn()>,
) -> impl IntoView {
    // Resources
    let secrets_resource = LocalResource::new(
        || async move { list_secrets(None).await }
    );

    // Signals for new secret form
    let (new_path, set_new_path) = signal(String::new());
    let (new_key, set_new_key) = signal(String::new());
    let (new_value, set_new_value) = signal(String::new());
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (is_loading, set_is_loading) = signal(false);

    // Form submission handler
    let handle_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_msg.set(None);
        set_success_msg.set(None);

        let path = new_path.get();
        let key = new_key.get();
        let value = new_value.get();

        if path.is_empty() || key.is_empty() || value.is_empty() {
            set_error_msg.set(Some("All fields are required".to_string()));
            set_is_loading.set(false);
            return;
        }

        let mut data = HashMap::new();
        data.insert(key, value);

        leptos::task::spawn_local(async move {
            match create_secret(&path, data).await {
                Ok(_) => {
                    set_success_msg.set(Some(format!("Secret '{}' created successfully", path)));
                    set_new_path.set(String::new());
                    set_new_key.set(String::new());
                    set_new_value.set(String::new());
                    secrets_resource.refetch(); // Refresh list
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to create secret: {}", e)));
                }
            }
            set_is_loading.set(false);
        });
    };

    view! {
        <MainLayout user_session=user_session.clone() on_logout=on_logout>
            <div class="container mx-auto px-4 py-8">
                <div class="mb-8">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">"Secrets Management"</h1>
                    <p class="text-gray-600 dark:text-gray-400">"Securely manage your application secrets and configuration."</p>
                </div>

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    // List Secrets
                    <div class="lg:col-span-2 bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">"Stored Secrets"</h2>

                        <Suspense fallback=move || view! { <div class="p-4 text-center">"Loading secrets..."</div> }>
                            {move || {
                                secrets_resource.get().map(|res| match res {
                                    Ok(secrets) => {
                                        if secrets.is_empty() {
                                            view! { <div class="p-4 text-center text-gray-500">"No secrets found."</div> }.into_any()
                                        } else {
                                            view! {
                                                <div class="overflow-x-auto">
                                                    <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                                                        <thead>
                                                            <tr>
                                                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Path"</th>
                                                                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">"Action"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                                                            {secrets.into_iter().map(|path| view! {
                                                                <tr>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-white">
                                                                        {path}
                                                                    </td>
                                                                    <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                                                        <button class="text-indigo-600 hover:text-indigo-900 dark:text-indigo-400 dark:hover:text-indigo-300">
                                                                            "View"
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }).collect_view()}
                                                        </tbody>
                                                    </table>
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                    Err(e) => view! {
                                        <div class="p-4 text-center text-red-500 bg-red-50 rounded-lg">
                                            {format!("Error loading secrets: {}", e)}
                                        </div>
                                    }.into_any()
                                })
                            }}
                        </Suspense>
                    </div>

                    // Create Secret Form
                    <div class="lg:col-span-1 bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 border border-gray-100 dark:border-gray-700 h-fit">
                        <h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">"Add New Secret"</h2>

                        <form on:submit=handle_submit class="space-y-4">
                            {move || error_msg.get().map(|msg| view! {
                                <div class="p-3 bg-red-100 text-red-700 rounded-lg text-sm">{msg}</div>
                            })}
                            {move || success_msg.get().map(|msg| view! {
                                <div class="p-3 bg-green-100 text-green-700 rounded-lg text-sm">{msg}</div>
                            })}

                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Path"</label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
                                    placeholder="e.g., app/db-config"
                                    prop:value=move || new_path.get()
                                    on:input=move |ev| set_new_path.set(event_target_value(&ev))
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Key"</label>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
                                    placeholder="e.g., password"
                                    prop:value=move || new_key.get()
                                    on:input=move |ev| set_new_key.set(event_target_value(&ev))
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">"Value"</label>
                                <input
                                    type="password"
                                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-indigo-500 dark:bg-gray-700 dark:text-white"
                                    placeholder="Secret value"
                                    prop:value=move || new_value.get()
                                    on:input=move |ev| set_new_value.set(event_target_value(&ev))
                                    disabled=move || is_loading.get()
                                />
                            </div>

                            <button
                                type="submit"
                                class="w-full py-2 px-4 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center"
                                disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() {
                                    view! { <span>"Saving..."</span> }.into_any()
                                } else {
                                    view! { <span>"Save Secret"</span> }.into_any()
                                }}
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        </MainLayout>
    }
}
