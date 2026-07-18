//! Client Detail Page (Admin)
//!
//! Read-only inspection of a single OAuth2/OIDC client, backed by
//! `GET /api/v1/iam/clients/{id}`. The old page mirrored Keycloak's 7-tab
//! console where five tabs were hard-coded "not configured" placeholders
//! and the credentials tab called a regenerate-secret endpoint that was a
//! NotImplemented stub — all removed with the #45 IAM trim. Clients are
//! seeded configuration; their secret never leaves the backend.
//! REQ-PORTAL-017

use crate::components::feedback::{ErrorBanner, LoadingPanel};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::async_load::load_value_once;
use crate::utils::authenc_api::ClientInfo;
use leptos::prelude::*;

/// Client detail page — takes client ID from path
#[component]
pub fn ClientDetailPage() -> impl IntoView {
    let api = use_api_client();

    let params = leptos_router::hooks::use_params_map();
    let client_id = move || params.get().get("id").unwrap_or_default();

    let (client, set_client) = signal(Option::<ClientInfo>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    // Load client
    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            let cid = client_id();
            load_value_once(
                move || {
                    let api = api.clone();
                    async move { api.iam_get_client(&cid).await.map(Some) }
                },
                set_client,
                set_error,
                set_loading,
                "Gagal memuat detail klien",
            );
        });
    }

    // Derived signals
    let client_name = move || {
        client
            .get()
            .and_then(|c| c.name.clone())
            .unwrap_or_else(|| "Klien Tanpa Nama".to_string())
    };
    let client_client_id = move || {
        client
            .get()
            .map(|c| c.client_id.clone())
            .unwrap_or_default()
    };
    let client_type = move || {
        client
            .get()
            .map(|c| c.client_type.clone())
            .unwrap_or_default()
    };
    let client_enabled = move || client.get().map(|c| c.enabled).unwrap_or(false);
    let client_created = move || {
        client
            .get()
            .map(|c| c.created_at.clone())
            .unwrap_or_default()
    };
    let client_uris = move || {
        client
            .get()
            .map(|c| c.redirect_uris.clone())
            .unwrap_or_default()
    };
    let client_id_display = move || client.get().map(|c| c.id.clone()).unwrap_or_default();

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-4">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / "
                    <a href="/portal/admin/clients" class="hover:text-primary-600">
                        "Klien"
                    </a>
                    " / Detail"
                </nav>

                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! { <LoadingPanel message="Memuat detail klien..." /> }
                >
                    <Show when=move || client.get().is_some()>
                        // Client header
                        <div class="bg-white rounded-xl border border-gray-200 overflow-hidden mb-6">
                            <div class="px-6 py-5 bg-gradient-to-r from-indigo-50 via-purple-50 to-pink-50">
                                <div class="flex items-center gap-4">
                                    <div class="w-14 h-14 rounded-xl bg-indigo-100 flex items-center justify-center text-2xl">
                                        "🔐"
                                    </div>
                                    <div class="flex-1">
                                        <h1 class="text-xl font-bold text-gray-900">
                                            {client_name}
                                        </h1>
                                        <p class="text-sm text-gray-500 font-mono">
                                            {client_client_id}
                                        </p>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        {move || {
                                            if client_enabled() {
                                                view! {
                                                    <span class="px-3 py-1 text-xs font-medium rounded-full bg-green-100 text-green-700">
                                                        "Aktif"
                                                    </span>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <span class="px-3 py-1 text-xs font-medium rounded-full bg-red-100 text-red-700">
                                                        "Nonaktif"
                                                    </span>
                                                }
                                                    .into_any()
                                            }
                                        }}
                                        <span class="px-3 py-1 text-xs font-medium rounded-full bg-indigo-100 text-indigo-700">
                                            {client_type}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            <div class="grid grid-cols-3 divide-x border-t">
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"ID Klien"</p>
                                    <p class="text-xs font-mono text-gray-700 truncate">
                                        {client_id_display}
                                    </p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Dibuat"</p>
                                    <p class="text-sm text-gray-700">{client_created}</p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"URI Pengalihan"</p>
                                    <p class="text-sm text-gray-700">
                                        {move || client_uris().len()}
                                    </p>
                                </div>
                            </div>
                        </div>

                        // Configuration details
                        <div class="bg-white rounded-xl border border-gray-200 p-6">
                            <h2 class="text-lg font-semibold text-gray-900 mb-4">
                                "Konfigurasi Klien"
                            </h2>
                            <dl class="space-y-4">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div class="border rounded-lg p-4">
                                        <dt class="text-xs text-gray-500 mb-1">"ID Klien"</dt>
                                        <dd class="text-sm font-mono font-semibold text-gray-900">
                                            {client_client_id}
                                        </dd>
                                    </div>
                                    <div class="border rounded-lg p-4">
                                        <dt class="text-xs text-gray-500 mb-1">"Tipe Klien"</dt>
                                        <dd class="text-sm font-semibold text-gray-900">
                                            {client_type}
                                        </dd>
                                    </div>
                                </div>
                                <div class="border rounded-lg p-4">
                                    <dt class="text-xs text-gray-500 mb-2">"Redirect URIs"</dt>
                                    <dd>
                                        {move || {
                                            let uris = client_uris();
                                            if uris.is_empty() {
                                                view! {
                                                    <p class="text-sm text-gray-400 italic">
                                                        "Tidak ada redirect URI."
                                                    </p>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <ul class="space-y-1">
                                                        {uris
                                                            .into_iter()
                                                            .map(|uri| {
                                                                view! {
                                                                    <li class="text-sm font-mono text-gray-700 bg-gray-50 px-3 py-1.5 rounded">
                                                                        {uri}
                                                                    </li>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                    </ul>
                                                }
                                                    .into_any()
                                            }
                                        }}
                                    </dd>
                                </div>
                                <div class="bg-gray-50 border border-gray-200 rounded-lg p-4">
                                    <p class="text-xs text-gray-500">
                                        "Klien merupakan konfigurasi ter-seed. Rahasia klien tidak pernah ditampilkan; perubahan konfigurasi dilakukan lewat seed/migrasi."
                                    </p>
                                </div>
                            </dl>
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
