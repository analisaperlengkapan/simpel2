//! Roles Management Page (Admin)
//!
//! Manage roles and permissions.
//! REQ-PORTAL-013

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::RoleInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Roles management page
#[component]
pub fn RolesManagementPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (roles, set_roles) = signal(Vec::<RoleInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.iam_list_roles().await {
                    Ok(list) => set_roles.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat peran: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

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
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / Peran"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Peran & Hak Akses"</h1>
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
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || roles.get()
                            key=|r| r.id.clone()
                            children=move |role| {
                                let perm_count = role.permissions.len();
                                view! {
                                    <div class="bg-white rounded-xl border border-gray-200 p-5">
                                        <div class="flex items-center justify-between mb-2">
                                            <h3 class="font-semibold text-gray-900">{role.name.clone()}</h3>
                                        </div>
                                        <p class="text-sm text-gray-500 mb-3">
                                            {role.description.clone().unwrap_or_else(|| "Tanpa deskripsi".to_string())}
                                        </p>
                                        <div class="flex items-center gap-2 text-xs text-gray-400">
                                            <span>"🛡️ " {format!("{} permission", perm_count)}</span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || roles.get().is_empty()>
                        <div class="text-center py-12 bg-white rounded-xl border">
                            <p class="text-4xl mb-3">"🛡️"</p>
                            <p class="text-gray-500">"Belum ada peran."</p>
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
