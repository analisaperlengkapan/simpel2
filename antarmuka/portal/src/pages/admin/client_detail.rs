//! Client Detail Page (Admin)
//!
//! Tabbed interface for viewing and managing a single OAuth2/OIDC client.
//! 7 tabs: Settings, Credentials, Roles, Client Scopes, Mappers, Scope, Sessions
//! REQ-PORTAL-017

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::ClientInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Active tab enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum ClientTab {
    Settings,
    Credentials,
    Roles,
    ClientScopes,
    Mappers,
    Scope,
    Sessions,
}

impl ClientTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Settings => "Pengaturan",
            Self::Credentials => "Kredensial",
            Self::Roles => "Peran",
            Self::ClientScopes => "Cakupan Klien",
            Self::Mappers => "Mapper",
            Self::Scope => "Scope",
            Self::Sessions => "Sesi",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Settings => "⚙",
            Self::Credentials => "🔑",
            Self::Roles => "🛡",
            Self::ClientScopes => "📋",
            Self::Mappers => "🔄",
            Self::Scope => "📦",
            Self::Sessions => "📱",
        }
    }

    fn all() -> &'static [ClientTab] {
        &[
            Self::Settings,
            Self::Credentials,
            Self::Roles,
            Self::ClientScopes,
            Self::Mappers,
            Self::Scope,
            Self::Sessions,
        ]
    }
}

/// Client detail page — takes client ID from path
#[component]
pub fn ClientDetailPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let params = leptos_router::hooks::use_params_map();
    let client_id = move || params.get().get("id").unwrap_or_default();

    let (client, set_client) = signal(Option::<ClientInfo>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (active_tab, set_active_tab) = signal(ClientTab::Settings);

    // Load client
    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            let cid = client_id();
            set_loading.set(true);
            spawn_local(async move {
                match api.iam_get_client(&cid).await {
                    Ok(c) => set_client.set(Some(c)),
                    Err(e) => set_error.set(Some(format!("Gagal memuat klien: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

    // Regenerate secret handler
    let (regen_trigger, set_regen_trigger) = signal(0u32);
    {
        let api = api.clone();
        Effect::new(move || {
            let count = regen_trigger.get();
            if count == 0 {
                return;
            }
            let api = api.clone();
            let cid = client_id();
            spawn_local(async move {
                match api.iam_regenerate_client_secret(&cid).await {
                    Ok(_) => set_success.set(Some("Client secret berhasil di-regenerasi".to_string())),
                    Err(e) => set_error.set(Some(format!("Gagal regenerasi secret: {}", e))),
                }
            });
        });
    }

    // Derived signals
    let client_name = move || client.get().and_then(|c| c.name.clone()).unwrap_or_else(|| "Unnamed Client".to_string());
    let client_client_id = move || client.get().map(|c| c.client_id.clone()).unwrap_or_default();
    let client_type = move || client.get().map(|c| c.client_type.clone()).unwrap_or_default();
    let client_enabled = move || client.get().map(|c| c.enabled).unwrap_or(false);
    let client_description = move || client.get().and_then(|c| c.description.clone()).unwrap_or_else(|| "—".to_string());
    let client_created = move || client.get().map(|c| c.created_at.clone()).unwrap_or_default();
    let client_uris = move || client.get().map(|c| c.redirect_uris.clone()).unwrap_or_default();
    let client_id_display = move || client.get().map(|c| c.id.clone()).unwrap_or_default();

    let on_logout = {
        let state = state.clone();
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-4">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / "
                    <a href="/portal/admin/clients" class="hover:text-primary-600">"Klien"</a>
                    " / Detail"
                </nav>

                {move || success.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">"✅ " {msg}</div>
                })}
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">"❌ " {msg}</div>
                })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <div class="bg-white rounded-xl border p-12 text-center">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-4"></div>
                            "Memuat detail klien..."
                        </div>
                    }
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
                                        <h1 class="text-xl font-bold text-gray-900">{client_name}</h1>
                                        <p class="text-sm text-gray-500 font-mono">{client_client_id}</p>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        {move || if client_enabled() {
                                            view! { <span class="px-3 py-1 text-xs font-medium rounded-full bg-green-100 text-green-700">"Aktif"</span> }.into_any()
                                        } else {
                                            view! { <span class="px-3 py-1 text-xs font-medium rounded-full bg-red-100 text-red-700">"Nonaktif"</span> }.into_any()
                                        }}
                                        <span class="px-3 py-1 text-xs font-medium rounded-full bg-indigo-100 text-indigo-700">{client_type}</span>
                                    </div>
                                </div>
                            </div>

                            <div class="grid grid-cols-3 divide-x border-t">
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"ID"</p>
                                    <p class="text-xs font-mono text-gray-700 truncate">{client_id_display}</p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Dibuat"</p>
                                    <p class="text-sm text-gray-700">{client_created}</p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Redirect URIs"</p>
                                    <p class="text-sm text-gray-700">{move || client_uris().len()}</p>
                                </div>
                            </div>
                        </div>

                        // Tabs
                        <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                            <div class="border-b">
                                <nav class="flex overflow-x-auto px-2">
                                    {ClientTab::all().iter().map(|tab| {
                                        let t = *tab;
                                        view! {
                                            <button
                                                on:click=move |_| set_active_tab.set(t)
                                                class=move || {
                                                    if active_tab.get() == t {
                                                        "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600 whitespace-nowrap"
                                                    } else {
                                                        "flex items-center gap-1.5 px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700 border-b-2 border-transparent whitespace-nowrap"
                                                    }
                                                }
                                            >
                                                <span>{t.icon()}</span>
                                                {t.label()}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </nav>
                            </div>

                            <div class="p-6">
                                // === Settings Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Settings>
                                    <div class="space-y-5">
                                        <dl class="space-y-4">
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                <div class="border rounded-lg p-4">
                                                    <dt class="text-xs text-gray-500 mb-1">"Client ID"</dt>
                                                    <dd class="text-sm font-mono font-semibold text-gray-900">{client_client_id}</dd>
                                                </div>
                                                <div class="border rounded-lg p-4">
                                                    <dt class="text-xs text-gray-500 mb-1">"Tipe Klien"</dt>
                                                    <dd class="text-sm font-semibold text-gray-900">{client_type}</dd>
                                                </div>
                                            </div>
                                            <div class="border rounded-lg p-4">
                                                <dt class="text-xs text-gray-500 mb-1">"Deskripsi"</dt>
                                                <dd class="text-sm text-gray-700">{client_description}</dd>
                                            </div>
                                            <div class="border rounded-lg p-4">
                                                <dt class="text-xs text-gray-500 mb-2">"Redirect URIs"</dt>
                                                <dd>
                                                    {move || {
                                                        let uris = client_uris();
                                                        if uris.is_empty() {
                                                            view! { <p class="text-sm text-gray-400 italic">"Tidak ada redirect URI."</p> }.into_any()
                                                        } else {
                                                            view! {
                                                                <ul class="space-y-1">
                                                                    {uris.into_iter().map(|uri| {
                                                                        view! {
                                                                            <li class="text-sm font-mono text-gray-700 bg-gray-50 px-3 py-1.5 rounded">{uri}</li>
                                                                        }
                                                                    }).collect::<Vec<_>>()}
                                                                </ul>
                                                            }.into_any()
                                                        }
                                                    }}
                                                </dd>
                                            </div>
                                        </dl>
                                    </div>
                                </Show>

                                // === Credentials Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Credentials>
                                    <div class="space-y-6">
                                        <div class="bg-amber-50 border border-amber-200 rounded-lg p-4">
                                            <h4 class="font-semibold text-amber-800">"⚠ Client Secret"</h4>
                                            <p class="text-sm text-amber-700 mt-1">"Regenerasi secret akan membuat secret lama tidak valid. Semua aplikasi yang menggunakan secret lama akan gagal otentikasi."</p>
                                        </div>
                                        <div class="border rounded-lg p-5">
                                            <h4 class="font-medium text-gray-900 mb-3">"Regenerasi Client Secret"</h4>
                                            <button
                                                on:click=move |_| set_regen_trigger.set(regen_trigger.get() + 1)
                                                class="px-4 py-2 bg-amber-500 text-white rounded-lg hover:bg-amber-600 transition-colors text-sm"
                                            >
                                                "🔄 Regenerasi Secret"
                                            </button>
                                        </div>
                                    </div>
                                </Show>

                                // === Roles Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Roles>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"🛡"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">"Peran Klien"</h3>
                                        <p class="text-sm text-gray-500">"Peran yang tersedia untuk klien ini."</p>
                                        <p class="text-xs text-gray-400 mt-4">"Belum ada peran klien yang dikonfigurasi."</p>
                                    </div>
                                </Show>

                                // === Client Scopes Tab ===
                                <Show when=move || active_tab.get() == ClientTab::ClientScopes>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"📋"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">"Cakupan Klien"</h3>
                                        <p class="text-sm text-gray-500">"Cakupan default dan opsional yang ditetapkan ke klien ini."</p>
                                        <p class="text-xs text-gray-400 mt-4">"Belum ada cakupan yang dikonfigurasi."</p>
                                    </div>
                                </Show>

                                // === Mappers Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Mappers>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"🔄"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">"Protocol Mappers"</h3>
                                        <p class="text-sm text-gray-500">"Mapper yang mengontrol klaim token untuk klien ini."</p>
                                        <p class="text-xs text-gray-400 mt-4">"Belum ada mapper yang dikonfigurasi."</p>
                                    </div>
                                </Show>

                                // === Scope Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Scope>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"📦"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">"Scope Evaluation"</h3>
                                        <p class="text-sm text-gray-500">"Evaluasi scope yang efektif untuk klien ini berdasarkan konfigurasi realm."</p>
                                        <p class="text-xs text-gray-400 mt-4">"Belum ada evaluasi scope."</p>
                                    </div>
                                </Show>

                                // === Sessions Tab ===
                                <Show when=move || active_tab.get() == ClientTab::Sessions>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"📱"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">"Sesi Aktif"</h3>
                                        <p class="text-sm text-gray-500">"Sesi aktif yang menggunakan klien ini."</p>
                                        <p class="text-xs text-gray-400 mt-4">"Tidak ada sesi aktif."</p>
                                    </div>
                                </Show>
                            </div>
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
