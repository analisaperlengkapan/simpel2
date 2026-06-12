//! Users Management Page (Admin)
//!
//! CRUD operations for user accounts with search, pagination, and filters.
//! REQ-PORTAL-010

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel, SuccessBanner};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::authenc_api::{CreateUserRequest, IamUser, UpdateUserRequest};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Users management page
#[component]
pub fn UsersManagementPage() -> impl IntoView {
    let api = use_api_client();

    let (users, set_users) = signal(Vec::<IamUser>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (search_query, set_search_query) = signal(String::new());
    let (page, set_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(1u32);
    let (show_create_modal, set_show_create_modal) = signal(false);

    // Form fields for new user
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_first_name, set_new_first_name) = signal(String::new());
    let (new_last_name, set_new_last_name) = signal(String::new());
    let (creating, set_creating) = signal(false);

    // Load users
    let load_users = {
        let api = api.clone();
        move || {
            let api = api.clone();
            set_loading.set(true);
            spawn_local(async move {
                let q = search_query.get();
                let search = if q.is_empty() { None } else { Some(q.as_str()) };
                match api.iam_list_users(page.get(), 20, search).await {
                    Ok(paginated) => {
                        set_users.set(paginated.data);
                        set_total_pages.set(paginated.total_pages);
                    }
                    Err(e) => set_error.set(Some(format!("Gagal memuat pengguna: {}", e))),
                }
                set_loading.set(false);
            });
        }
    };

    // Initial load
    {
        let load = load_users.clone();
        Effect::new(move || {
            let _ = page.get();
            load();
        });
    }

    let handle_search = {
        let load = load_users.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();

            // Only explicitly load if we are already on page 1.
            // If page > 1, setting page to 1 will trigger the Effect above.
            if page.get() == 1 {
                load();
            } else {
                set_page.set(1);
            }
        }
    };

    // Trigger signals for create and toggle
    let (create_trigger, set_create_trigger) = signal(0u32);
    let (toggle_trigger, set_toggle_trigger) = signal(Option::<(String, bool)>::None);

    // Create user effect
    {
        let api = api.clone();
        let load = load_users.clone();
        Effect::new(move || {
            let count = create_trigger.get();
            if count == 0 {
                return;
            }
            let api = api.clone();
            let load = load.clone();
            set_creating.set(true);
            set_error.set(None);

            spawn_local(async move {
                let req = CreateUserRequest {
                    username: new_username.get(),
                    email: new_email.get(),
                    password: "TempPass123!".to_string(),
                    first_name: if new_first_name.get().is_empty() {
                        None
                    } else {
                        Some(new_first_name.get())
                    },
                    last_name: if new_last_name.get().is_empty() {
                        None
                    } else {
                        Some(new_last_name.get())
                    },
                    enabled: true,
                };

                match api.iam_create_user(&req).await {
                    Ok(_) => {
                        set_success.set(Some("Pengguna berhasil dibuat".to_string()));
                        set_show_create_modal.set(false);
                        set_new_username.set(String::new());
                        set_new_email.set(String::new());
                        set_new_first_name.set(String::new());
                        set_new_last_name.set(String::new());
                        load();
                    }
                    Err(e) => set_error.set(Some(format!("Gagal membuat pengguna: {}", e))),
                }
                set_creating.set(false);
            });
        });
    }

    // Toggle user effect
    {
        let api = api.clone();
        let load = load_users.clone();
        Effect::new(move || {
            let trigger = toggle_trigger.get();
            if let Some((user_id, enable)) = trigger {
                let api = api.clone();
                let load = load.clone();
                spawn_local(async move {
                    let req = UpdateUserRequest {
                        email: None,
                        first_name: None,
                        last_name: None,
                        enabled: Some(enable),
                    };
                    match api.iam_update_user(&user_id, &req).await {
                        Ok(_) => {
                            set_success.set(Some(if enable {
                                "Pengguna diaktifkan".to_string()
                            } else {
                                "Pengguna dinonaktifkan".to_string()
                            }));
                            load();
                        }
                        Err(e) => set_error.set(Some(format!("Gagal: {}", e))),
                    }
                });
            }
        });
    }

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <nav class="text-sm text-gray-500 mb-1">
                            <a href="/portal/admin" class="hover:text-primary-600">
                                "Admin"
                            </a>
                            " / Pengguna"
                        </nav>
                        <h1 class="text-2xl font-bold text-gray-900">"Manajemen Pengguna"</h1>
                    </div>
                    <button
                        on:click=move |_| set_show_create_modal.set(true)
                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
                    >
                        "+ Tambah Pengguna"
                    </button>
                </div>

                {move || success.get().map(|msg| view! { <SuccessBanner message=msg /> })}
                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

                // Search bar
                <form on:submit=handle_search class="mb-6">
                    <div class="flex gap-2">
                        <input
                            type="text"
                            prop:value=search_query
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            placeholder="Cari pengguna..."
                            class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
                        />
                        <button
                            type="submit"
                            class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200"
                        >
                            "🔍 Cari"
                        </button>
                    </div>
                </form>

                // Users table
                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <LoadingPanel message="Memuat data pengguna..." /> }
                    >
                        <table class="w-full">
                            <thead class="bg-gray-50 border-b">
                                <tr>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Username"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Email"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Nama"
                                    </th>
                                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                        "Status"
                                    </th>
                                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                                        "Aksi"
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-200">
                                <For
                                    each=move || users.get()
                                    key=|u| u.id.clone()
                                    children=move |user| {
                                        let uid = user.id.clone();
                                        let uid2 = user.id.clone();
                                        let is_enabled = user.enabled;

                                        view! {
                                            <tr class="hover:bg-gray-50">
                                                <td class="px-4 py-3 font-medium text-gray-900">
                                                    {user.username.clone()}
                                                </td>
                                                <td class="px-4 py-3 text-gray-600">
                                                    {user.email.clone()}
                                                </td>
                                                <td class="px-4 py-3 text-gray-600">
                                                    {format!(
                                                        "{} {}",
                                                        user.first_name.as_deref().unwrap_or(""),
                                                        user.last_name.as_deref().unwrap_or(""),
                                                    )
                                                        .trim()
                                                        .to_string()}
                                                </td>
                                                <td class="px-4 py-3">
                                                    <span class=if is_enabled {
                                                        "inline-flex px-2 py-0.5 text-xs rounded-full bg-green-100 text-green-700"
                                                    } else {
                                                        "inline-flex px-2 py-0.5 text-xs rounded-full bg-red-100 text-red-700"
                                                    }>{if is_enabled { "Aktif" } else { "Nonaktif" }}</span>
                                                </td>
                                                <td class="px-4 py-3 text-right">
                                                    <button
                                                        on:click=move |_| {
                                                            set_toggle_trigger.set(Some((uid.clone(), !is_enabled)));
                                                        }
                                                        class="text-sm text-primary-600 hover:text-primary-700 mr-3"
                                                    >
                                                        {if is_enabled { "Nonaktifkan" } else { "Aktifkan" }}
                                                    </button>
                                                    <a
                                                        href=format!("/portal/admin/users/{}", uid2)
                                                        class="text-sm text-gray-500 hover:text-gray-700"
                                                    >
                                                        "Detail"
                                                    </a>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>

                        <Show when=move || users.get().is_empty()>
                            <div class="m-4">
                                <EmptyPanel
                                    title="Belum ada pengguna"
                                    message="Belum ditemukan data pengguna untuk filter saat ini."
                                />
                            </div>
                        </Show>

                        // Pagination
                        <Show when=move || { total_pages.get() > 1 }>
                            <div class="flex items-center justify-between px-4 py-3 border-t bg-gray-50">
                                <button
                                    on:click=move |_| {
                                        set_page.set(page.get().saturating_sub(1).max(1))
                                    }
                                    disabled=move || page.get() <= 1
                                    class="px-3 py-1 text-sm border rounded disabled:opacity-50"
                                >
                                    "← Sebelumnya"
                                </button>
                                <span class="text-sm text-gray-600">
                                    "Halaman " {move || page.get()} " dari "
                                    {move || total_pages.get()}
                                </span>
                                <button
                                    on:click=move |_| {
                                        set_page.set((page.get() + 1).min(total_pages.get()))
                                    }
                                    disabled=move || page.get()
                                >
                                    = total_pages.get()
                                    class=
                                    "px-3 py-1 text-sm border rounded disabled:opacity-50"
                                    >
                                    "Selanjutnya →"
                                </button>
                            </div>
                        </Show>
                    </Show>
                </div>

                // Create user modal
                <Show when=move || show_create_modal.get()>
                    <div class="fixed inset-0 z-modal flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                            <h2 class="text-lg font-bold text-gray-900 mb-4">
                                "Tambah Pengguna Baru"
                            </h2>
                            <form
                                on:submit=move |ev: web_sys::SubmitEvent| {
                                    ev.prevent_default();
                                    set_create_trigger.set(create_trigger.get() + 1);
                                }
                                class="space-y-4"
                            >
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Username *"
                                    </label>
                                    <input
                                        type="text"
                                        prop:value=new_username
                                        on:input=move |ev| {
                                            set_new_username.set(event_target_value(&ev))
                                        }
                                        required=true
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Email *"
                                    </label>
                                    <input
                                        type="email"
                                        prop:value=new_email
                                        on:input=move |ev| {
                                            set_new_email.set(event_target_value(&ev))
                                        }
                                        required=true
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
                                </div>
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Nama Depan"
                                        </label>
                                        <input
                                            type="text"
                                            prop:value=new_first_name
                                            on:input=move |ev| {
                                                set_new_first_name.set(event_target_value(&ev))
                                            }
                                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Nama Belakang"
                                        </label>
                                        <input
                                            type="text"
                                            prop:value=new_last_name
                                            on:input=move |ev| {
                                                set_new_last_name.set(event_target_value(&ev))
                                            }
                                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                        />
                                    </div>
                                </div>
                                <div class="flex justify-end gap-3 pt-2">
                                    <button
                                        type="button"
                                        on:click=move |_| set_show_create_modal.set(false)
                                        class="px-4 py-2 text-gray-700 border rounded-lg hover:bg-gray-50"
                                    >
                                        "Batal"
                                    </button>
                                    <button
                                        type="submit"
                                        disabled=move || creating.get()
                                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
                                    >
                                        {move || {
                                            if creating.get() { "Membuat..." } else { "Buat Pengguna" }
                                        }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </Show>
            </div>
        </MainLayout>
    }
}
