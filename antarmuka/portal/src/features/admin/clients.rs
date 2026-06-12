//! OAuth2 Clients Management Page (Admin)
//!
//! Manage OAuth2/OIDC client applications.
//! REQ-PORTAL-012

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::async_load::load_vec_once;
use crate::utils::authenc_api::ClientInfo;
use leptos::prelude::*;

/// OAuth2 clients management page
#[component]
pub fn ClientsManagementPage() -> impl IntoView {
    let api = use_api_client();

    let (clients, set_clients) = signal(Vec::<ClientInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            load_vec_once(
                move || {
                    let api = api.clone();
                    async move { api.iam_list_clients().await }
                },
                set_clients,
                set_error,
                set_loading,
                "Gagal memuat klien",
            );
        });
    }

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / Klien OAuth2"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Klien OAuth2"</h1>
                </div>

                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! { <LoadingPanel message="Memuat data klien OAuth2..." /> }
                >
                    <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                        <table class="w-full">
                            <thead class="bg-gray-50 border-b">
                                <tr>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Client ID"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Nama"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Tipe"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Status"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Redirect URI"
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-200">
                                <For
                                    each=move || clients.get()
                                    key=|c| c.id.clone()
                                    children=move |client| {
                                        let is_enabled = client.enabled;
                                        let redirect_uris = client.redirect_uris.clone();
                                        view! {
                                            <tr class="hover:bg-gray-50">
                                                <td class="px-4 py-3 font-mono text-sm text-gray-900">
                                                    {client.client_id.clone()}
                                                </td>
                                                <td class="px-4 py-3 text-gray-700">
                                                    {client.name.clone().unwrap_or_default()}
                                                </td>
                                                <td class="px-4 py-3 text-gray-500 text-sm">
                                                    {client.client_type.clone()}
                                                </td>
                                                <td class="px-4 py-3">
                                                    <span class=if is_enabled {
                                                        "text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700"
                                                    } else {
                                                        "text-xs px-2 py-0.5 rounded-full bg-red-100 text-red-700"
                                                    }>{if is_enabled { "Aktif" } else { "Nonaktif" }}</span>
                                                </td>
                                                <td class="px-4 py-3 text-xs text-gray-500 max-w-xs truncate">
                                                    {redirect_uris.join(", ")}
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>

                    <Show when=move || clients.get().is_empty()>
                        <div class="mt-4">
                            <EmptyPanel
                                title="Belum ada klien OAuth2"
                                message="Daftar klien masih kosong pada realm aktif."
                            />
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
