//! Federation Management Page (Admin)
//!
//! Manage external Identity Providers (SAML, OIDC, social login).
//! REQ-PORTAL-014

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::IdentityProviderInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Federation management page
#[component]
pub fn FederationManagementPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (providers, set_providers) = signal(Vec::<IdentityProviderInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.iam_list_identity_providers().await {
                    Ok(list) => set_providers.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat IdP: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

    let on_logout = {
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / Federasi"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Federasi Identity Provider"</h1>
                </div>

                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">"❌ " {msg}</div>
                })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <div class="p-8 text-center text-gray-500">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-3"></div>
                            "Memuat..."
                        </div>
                    }
                >
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <For
                            each=move || providers.get()
                            key=|p| p.alias.clone()
                            children=move |provider| {
                                let is_enabled = provider.enabled;
                                let provider_icon = match provider.provider_type.as_str() {
                                    "saml" => "📜",
                                    "oidc" => "🔐",
                                    "google" => "🔵",
                                    "github" => "⬛",
                                    _ => "🌐",
                                };
                                view! {
                                    <div class={format!(
                                        "bg-white rounded-xl border p-5 {}",
                                        if is_enabled { "border-gray-200" } else { "border-gray-200 opacity-60" }
                                    )}>
                                        <div class="flex items-center gap-3 mb-3">
                                            <span class="text-2xl">{provider_icon}</span>
                                            <div>
                                                <h3 class="font-semibold text-gray-900">{provider.display_name.clone().unwrap_or_else(|| provider.alias.clone())}</h3>
                                                <p class="text-xs text-gray-500">
                                                    {provider.provider_type.clone().to_uppercase()}
                                                    " — "
                                                    {provider.alias.clone()}
                                                </p>
                                            </div>
                                            <span class={format!(
                                                "ml-auto text-xs px-2 py-0.5 rounded-full {}",
                                                if is_enabled { "bg-green-100 text-green-700" } else { "bg-gray-100 text-gray-500" }
                                            )}>
                                                {if is_enabled { "Aktif" } else { "Nonaktif" }}
                                            </span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || providers.get().is_empty()>
                        <div class="text-center py-12 bg-white rounded-xl border">
                            <p class="text-4xl mb-3">"🌐"</p>
                            <p class="text-gray-500">"Belum ada Identity Provider yang dikonfigurasi."</p>
                            <p class="text-sm text-gray-400 mt-2">"Tambahkan IdP untuk mengaktifkan Single Sign-On."</p>
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
