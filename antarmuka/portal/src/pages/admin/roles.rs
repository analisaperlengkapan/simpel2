//! Roles Management Page (Admin)
//!
//! Manage roles in the active realm. Backed by IAM Admin
//! `/api/v1/iam/roles` (list/create/delete). Permissions assignment is
//! Phase 2 follow-up — Keycloak's full "scope/policy" model lives on
//! `permissions.rs`.
//! REQ-PORTAL-013

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::async_load::load_vec_once;
use crate::utils::authenc_api::{CreateRoleRequest, RoleInfo};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Roles management page
#[component]
pub fn RolesManagementPage() -> impl IntoView {
    let api = use_api_client();

    let (roles, set_roles) = signal(Vec::<RoleInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (show_create, set_show_create) = signal(false);
    let (deleting, set_deleting) = signal::<Option<RoleInfo>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    {
        let api = api.clone();
        Effect::new(move || {
            // Tracking the tick re-runs this effect whenever a mutation
            // signals it should refresh — avoids manually re-fetching after
            // every callback at every call site.
            let _ = reload_tick.get();
            let api = api.clone();
            load_vec_once(
                move || {
                    let api = api.clone();
                    async move { api.iam_list_roles().await }
                },
                set_roles,
                set_error,
                set_loading,
                "Gagal memuat peran",
            );
        });
    }

    let (session, on_logout) = use_main_layout_session_and_logout();

    let bump_reload = move || set_reload_tick.update(|t| *t += 1);
    let on_create_clicked = move |_| set_show_create.set(true);
    let close_create = Callback::new(move |_| set_show_create.set(false));
    let saved_create = Callback::new(move |_| {
        set_show_create.set(false);
        bump_reload();
    });
    let close_delete = Callback::new(move |_| set_deleting.set(None));
    let confirm_delete = Callback::new(move |_| {
        set_deleting.set(None);
        bump_reload();
    });

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                    " / Peran"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Peran & Hak Akses"</h1>
                    <button
                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium"
                        on:click=on_create_clicked
                    >
                        "＋ Peran Baru"
                    </button>
                </div>

                {move || error.get().map(|msg| view! {
                    <ErrorBanner message=msg />
                })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <LoadingPanel message="Memuat data peran..." />
                    }
                >
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || roles.get()
                            key=|r| r.id.clone()
                            children=move |role| {
                                let perm_count = role.permissions.len();
                                let role_for_delete = role.clone();
                                view! {
                                    <div class="bg-white rounded-xl border border-gray-200 p-5">
                                        <div class="flex items-center justify-between mb-2">
                                            <h3 class="font-semibold text-gray-900">{role.name.clone()}</h3>
                                            <button
                                                class="text-xs text-red-600 hover:text-red-800 font-medium"
                                                title="Hapus peran"
                                                on:click=move |_| set_deleting.set(Some(role_for_delete.clone()))
                                            >
                                                "🗑️ Hapus"
                                            </button>
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
                        <EmptyPanel
                            title="Belum ada peran"
                            message="Data peran belum tersedia pada realm aktif."
                        />
                    </Show>
                </Show>

                {move || show_create.get().then(|| view! {
                    <CreateRoleModal on_close=close_create on_saved=saved_create />
                })}

                {move || deleting.get().map(|role| view! {
                    <DeleteRoleModal role=role on_close=close_delete on_confirmed=confirm_delete />
                })}
            </div>
        </MainLayout>
    }
}

/// Modal: create a new realm role.
#[component]
fn CreateRoleModal(
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_saved: Callback<()>,
) -> impl IntoView {
    let api = use_api_client();
    let (name, set_name) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal::<Option<String>>(None);

    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let name_value = name.get();
        if name_value.trim().is_empty() {
            set_submit_error.set(Some("Nama peran wajib diisi.".to_string()));
            return;
        }
        let desc_value = description.get();
        let req = CreateRoleRequest {
            name: name_value.trim().to_string(),
            description: if desc_value.trim().is_empty() {
                None
            } else {
                Some(desc_value)
            },
            // Permissions assignment is a separate flow (see permissions.rs)
            // — we ship an empty role here so admins don't have to think
            // about Keycloak's authorization model up front.
            permissions: Vec::new(),
        };

        set_submitting.set(true);
        set_submit_error.set(None);
        let api = api.clone();
        let on_saved = on_saved;
        spawn_local(async move {
            match api.iam_create_role(&req).await {
                Ok(_) => {
                    set_submitting.set(false);
                    on_saved.run(());
                }
                Err(e) => {
                    set_submitting.set(false);
                    set_submit_error.set(Some(format!("Gagal membuat peran: {}", e)));
                }
            }
        });
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <form
                class="w-full max-w-md bg-white rounded-xl shadow-xl p-6 space-y-4"
                on:submit=submit
            >
                <header class="flex items-start justify-between">
                    <h2 class="text-lg font-semibold text-gray-900">"Peran Baru"</h2>
                    <button
                        type="button"
                        class="text-gray-400 hover:text-gray-600 text-xl leading-none"
                        on:click=move |_| on_close.run(())
                    >
                        "×"
                    </button>
                </header>

                <label class="block">
                    <span class="block text-sm font-medium text-gray-700 mb-1">"Nama Peran"</span>
                    <input
                        type="text"
                        class="w-full px-3 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary-500"
                        placeholder="contoh: admin_satker"
                        prop:value=move || name.get()
                        on:input=move |e| set_name.set(event_target_value(&e))
                    />
                </label>

                <label class="block">
                    <span class="block text-sm font-medium text-gray-700 mb-1">"Deskripsi (opsional)"</span>
                    <textarea
                        class="w-full px-3 py-2 border border-gray-200 rounded-lg focus:ring-2 focus:ring-primary-500"
                        rows="3"
                        placeholder="Untuk apa peran ini digunakan?"
                        prop:value=move || description.get()
                        on:input=move |e| set_description.set(event_target_value(&e))
                    />
                </label>

                {move || submit_error.get().map(|msg| view! {
                    <div class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                        {msg}
                    </div>
                })}

                <div class="flex justify-end gap-2 pt-2">
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="submit"
                        prop:disabled=move || submitting.get()
                        class="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg disabled:opacity-60 disabled:cursor-not-allowed"
                    >
                        {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                    </button>
                </div>
            </form>
        </div>
    }
}

/// Modal: confirm delete a role.
#[component]
fn DeleteRoleModal(
    role: RoleInfo,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_confirmed: Callback<()>,
) -> impl IntoView {
    let api = use_api_client();
    let (deleting, set_deleting) = signal(false);
    let (delete_error, set_delete_error) = signal::<Option<String>>(None);
    let role_id = role.id.clone();
    let role_name = role.name.clone();
    let role_perms = role.permissions.len();

    let confirm = move |_| {
        let id = role_id.clone();
        set_deleting.set(true);
        set_delete_error.set(None);
        let api = api.clone();
        let on_confirmed = on_confirmed;
        spawn_local(async move {
            match api.iam_delete_role(&id).await {
                Ok(()) => {
                    set_deleting.set(false);
                    on_confirmed.run(());
                }
                Err(e) => {
                    set_deleting.set(false);
                    set_delete_error.set(Some(format!("Gagal menghapus peran: {}", e)));
                }
            }
        });
    };

    view! {
        <div
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <div class="w-full max-w-md bg-white rounded-xl shadow-xl p-6 space-y-4">
                <h2 class="text-lg font-semibold text-gray-900">"Hapus Peran"</h2>
                <p class="text-sm text-gray-600">
                    "Anda akan menghapus peran "
                    <strong class="text-gray-900">{role_name}</strong>
                    " yang memiliki "
                    {format!("{} permission", role_perms)}
                    ". Tindakan ini tidak dapat dibatalkan."
                </p>

                {move || delete_error.get().map(|msg| view! {
                    <div class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
                        {msg}
                    </div>
                })}

                <div class="flex justify-end gap-2">
                    <button
                        type="button"
                        class="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg"
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        prop:disabled=move || deleting.get()
                        class="px-4 py-2 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded-lg disabled:opacity-60 disabled:cursor-not-allowed"
                        on:click=confirm
                    >
                        {move || if deleting.get() { "Menghapus..." } else { "Hapus" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
