//! Realms Management Page (Admin)
//!
//! Manage multi-tenant realms (create, edit, delete).
//! REQ-PORTAL-011

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::RealmInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Realms management page
#[component]
pub fn RealmsManagementPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (realms, set_realms) = signal(Vec::<RealmInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, _set_success) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.iam_list_realms().await {
                    Ok(list) => set_realms.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat realm: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

    let on_logout = {
        let state = state;
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
                    " / Realm"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Realm"</h1>
                </div>

                {move || success.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">"✅ " {msg}</div>
                })}
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">"❌ " {msg}</div>
                })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <div class="p-8 text-center text-gray-500">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-3"></div>
                            "Memuat realm..."
                        </div>
                    }
                >
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || realms.get()
                            key=|r| r.id.clone()
                            children=move |realm| {
                                let is_enabled = realm.enabled;
                                view! {
                                    <div class="bg-white rounded-xl border border-gray-200 p-5">
                                        <div class="flex items-center justify-between mb-2">
                                            <h3 class="font-semibold text-gray-900">{realm.name.clone()}</h3>
                                            <span class={if is_enabled {
                                                "text-xs px-2 py-0.5 rounded-full bg-green-100 text-green-700"
                                            } else {
                                                "text-xs px-2 py-0.5 rounded-full bg-gray-100 text-gray-500"
                                            }}>
                                                {if is_enabled { "Aktif" } else { "Nonaktif" }}
                                            </span>
                                        </div>
                                        <p class="text-sm text-gray-500 mb-3">
                                            {realm.display_name.clone().unwrap_or_else(|| realm.name.clone())}
                                        </p>
                                        <div class="text-xs text-gray-400">
                                            "ID: " {realm.id.clone()}
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || realms.get().is_empty()>
                        <div class="text-center py-12 bg-white rounded-xl border">
                            <p class="text-4xl mb-3">"🏢"</p>
                            <p class="text-gray-500">"Belum ada realm."</p>
                        </div>
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
