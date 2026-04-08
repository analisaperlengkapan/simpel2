//! Groups Management Page (Admin)
//!
//! Hierarchical group management with tree view, CRUD, and member management.
//! REQ-PORTAL-015

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::{CreateGroupApiRequest, GroupInfo};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Groups management page
#[component]
pub fn GroupsManagementPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (groups, set_groups) = signal(Vec::<GroupInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (search_query, set_search_query) = signal(String::new());
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (selected_group, set_selected_group) = signal(Option::<GroupInfo>::None);

    // Form fields for new group
    let (new_name, set_new_name) = signal(String::new());
    let (new_description, set_new_description) = signal(String::new());
    let (creating, set_creating) = signal(false);

    // Load groups
    let load_groups = {
        let api = api.clone();
        move || {
            let api = api.clone();
            set_loading.set(true);
            spawn_local(async move {
                match api.iam_list_groups().await {
                    Ok(list) => set_groups.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat grup: {}", e))),
                }
                set_loading.set(false);
            });
        }
    };

    // Initial load
    {
        let load = load_groups.clone();
        Effect::new(move || {
            load();
        });
    }

    let handle_search = {
        let load = load_groups.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            load();
        }
    };

    // Create group effect
    let (create_trigger, set_create_trigger) = signal(0u32);
    {
        let api = api.clone();
        let load = load_groups.clone();
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
                let req = CreateGroupApiRequest {
                    realm_id: "00000000-0000-0000-0000-000000000000".to_string(),
                    name: new_name.get(),
                    parent_id: None,
                    description: if new_description.get().is_empty() {
                        None
                    } else {
                        Some(new_description.get())
                    },
                };

                match api.iam_create_group(&req).await {
                    Ok(_) => {
                        set_success.set(Some("Grup berhasil dibuat".to_string()));
                        set_show_create_modal.set(false);
                        set_new_name.set(String::new());
                        set_new_description.set(String::new());
                        load();
                    }
                    Err(e) => set_error.set(Some(format!("Gagal membuat grup: {}", e))),
                }
                set_creating.set(false);
            });
        });
    }

    // Delete group effect
    let (delete_trigger, set_delete_trigger) = signal(Option::<String>::None);
    {
        let api = api.clone();
        let load = load_groups.clone();
        Effect::new(move || {
            let trigger = delete_trigger.get();
            if let Some(group_id) = trigger {
                let api = api.clone();
                let load = load.clone();
                spawn_local(async move {
                    match api.iam_delete_group(&group_id).await {
                        Ok(_) => {
                            set_success.set(Some("Grup berhasil dihapus".to_string()));
                            set_selected_group.set(None);
                            load();
                        }
                        Err(e) => set_error.set(Some(format!("Gagal menghapus grup: {}", e))),
                    }
                });
            }
        });
    }

    let on_logout = {
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    // Filter groups by search
    let filtered_groups = move || {
        let query = search_query.get().to_lowercase();
        let all = groups.get();
        if query.is_empty() {
            // Show only root groups (no parent_id)
            all.into_iter()
                .filter(|g| g.parent_id.is_none())
                .collect::<Vec<_>>()
        } else {
            all.into_iter()
                .filter(|g| g.name.to_lowercase().contains(&query))
                .collect::<Vec<_>>()
        }
    };

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <nav class="text-sm text-gray-500 mb-1">
                            <a href="/portal/admin" class="hover:text-primary-600">"Admin"</a>
                            " / Grup"
                        </nav>
                        <h1 class="text-2xl font-bold text-gray-900">"Manajemen Grup"</h1>
                        <p class="text-sm text-gray-500 mt-1">"Kelola hierarki grup dan keanggotaan pengguna"</p>
                    </div>
                    <button
                        on:click=move |_| set_show_create_modal.set(true)
                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors flex items-center gap-2"
                    >
                        "＋ Buat Grup"
                    </button>
                </div>

                {move || success.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">"✅ " {msg}</div>
                })}
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">"❌ " {msg}</div>
                })}

                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    // Left panel - Group tree
                    <div class="lg:col-span-1">
                        // Search bar
                        <form on:submit=handle_search class="mb-4">
                            <input
                                type="text"
                                prop:value=search_query
                                on:input=move |ev| set_search_query.set(event_target_value(&ev))
                                placeholder="Cari grup..."
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 text-sm"
                            />
                        </form>

                        <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                            <div class="px-4 py-3 bg-gray-50 border-b">
                                <h3 class="text-sm font-semibold text-gray-700">"📂 Grup"</h3>
                            </div>

                            <Show
                                when=move || !loading.get()
                                fallback=|| view! {
                                    <div class="p-8 text-center text-gray-500">
                                        <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-600 mx-auto mb-3"></div>
                                        "Memuat..."
                                    </div>
                                }
                            >
                                <div class="divide-y divide-gray-100 max-h-96 overflow-y-auto">
                                    <For
                                        each=move || filtered_groups()
                                        key=|g| g.id.clone()
                                        children=move |group| {
                                            let gid = group.id.clone();
                                            let g_clone = group.clone();
                                            let is_selected = move || {
                                                selected_group.get().as_ref().map(|s| s.id == gid).unwrap_or(false)
                                            };

                                            view! {
                                                <button
                                                    on:click=move |_| set_selected_group.set(Some(g_clone.clone()))
                                                    class=move || if is_selected() {
                                                        "w-full text-left px-4 py-3 bg-primary-50 border-l-4 border-primary-600 transition-colors"
                                                    } else {
                                                        "w-full text-left px-4 py-3 hover:bg-gray-50 border-l-4 border-transparent transition-colors"
                                                    }
                                                >
                                                    <div class="flex items-center justify-between">
                                                        <div>
                                                            <p class="font-medium text-gray-900 text-sm">{group.name.clone()}</p>
                                                            <p class="text-xs text-gray-500 mt-0.5">
                                                                {group.member_count} " anggota"
                                                                {if group.subgroup_count > 0 {
                                                                    format!(" · {} sub-grup", group.subgroup_count)
                                                                } else {
                                                                    String::new()
                                                                }}
                                                            </p>
                                                        </div>
                                                        <span class="text-gray-400 text-xs">"›"</span>
                                                    </div>
                                                </button>
                                            }
                                        }
                                    />
                                </div>

                                <Show when=move || filtered_groups().is_empty()>
                                    <div class="p-8 text-center">
                                        <p class="text-3xl mb-2">"📁"</p>
                                        <p class="text-sm text-gray-500">"Belum ada grup."</p>
                                        <p class="text-xs text-gray-400 mt-1">"Klik \"Buat Grup\" untuk menambahkan."</p>
                                    </div>
                                </Show>
                            </Show>
                        </div>
                    </div>

                    // Right panel - Group detail
                    <div class="lg:col-span-2">
                        <Show
                            when=move || selected_group.get().is_some()
                            fallback=|| view! {
                                <div class="bg-white rounded-xl border border-gray-200 p-12 text-center">
                                    <p class="text-5xl mb-4">"👥"</p>
                                    <h3 class="text-lg font-semibold text-gray-700 mb-2">"Pilih Grup"</h3>
                                    <p class="text-sm text-gray-500">"Pilih grup dari panel kiri untuk melihat detail dan anggota."</p>
                                </div>
                            }
                        >
                            {move || selected_group.get().map(|group| {
                                let gid = group.id.clone();
                                let gid2 = group.id.clone();

                                view! {
                                    <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                                        // Header
                                        <div class="px-6 py-4 bg-gradient-to-r from-primary-50 to-blue-50 border-b">
                                            <div class="flex items-center justify-between">
                                                <div>
                                                    <h2 class="text-lg font-bold text-gray-900">{group.name.clone()}</h2>
                                                    <p class="text-sm text-gray-500 mt-1">
                                                        {group.description.clone().unwrap_or_else(|| "Tidak ada deskripsi".to_string())}
                                                    </p>
                                                </div>
                                                <button
                                                    on:click=move |_| {
                                                        set_delete_trigger.set(Some(gid.clone()));
                                                    }
                                                    class="px-3 py-1.5 text-sm text-red-600 bg-red-50 rounded-lg hover:bg-red-100 transition-colors"
                                                >
                                                    "🗑 Hapus"
                                                </button>
                                            </div>
                                        </div>

                                        // Stats cards
                                        <div class="grid grid-cols-3 gap-4 p-6 border-b">
                                            <div class="bg-blue-50 rounded-lg p-4 text-center">
                                                <p class="text-2xl font-bold text-blue-700">{group.member_count}</p>
                                                <p class="text-xs text-blue-600 mt-1">"Anggota"</p>
                                            </div>
                                            <div class="bg-purple-50 rounded-lg p-4 text-center">
                                                <p class="text-2xl font-bold text-purple-700">{group.subgroup_count}</p>
                                                <p class="text-xs text-purple-600 mt-1">"Sub-grup"</p>
                                            </div>
                                            <div class="bg-green-50 rounded-lg p-4 text-center">
                                                <p class="text-2xl font-bold text-green-700">"—"</p>
                                                <p class="text-xs text-green-600 mt-1">"Peran"</p>
                                            </div>
                                        </div>

                                        // Tabs
                                        <div class="border-b">
                                            <nav class="flex px-6">
                                                <button class="px-4 py-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600">
                                                    "Detail"
                                                </button>
                                                <button class="px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700">
                                                    "Anggota"
                                                </button>
                                                <button class="px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700">
                                                    "Sub-grup"
                                                </button>
                                                <button class="px-4 py-3 text-sm font-medium text-gray-500 hover:text-gray-700">
                                                    "Atribut"
                                                </button>
                                            </nav>
                                        </div>

                                        // Detail content
                                        <div class="p-6">
                                            <dl class="space-y-4">
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"ID"</dt>
                                                    <dd class="text-sm font-mono text-gray-900">{gid2}</dd>
                                                </div>
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"Nama"</dt>
                                                    <dd class="text-sm font-semibold text-gray-900">{group.name.clone()}</dd>
                                                </div>
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"Realm"</dt>
                                                    <dd class="text-sm text-gray-700">{group.realm_id.clone()}</dd>
                                                </div>
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"Induk"</dt>
                                                    <dd class="text-sm text-gray-700">
                                                        {group.parent_id.clone().unwrap_or_else(|| "—  (root)".to_string())}
                                                    </dd>
                                                </div>
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"Dibuat"</dt>
                                                    <dd class="text-sm text-gray-700">{group.created_at.clone()}</dd>
                                                </div>
                                                <div class="flex justify-between">
                                                    <dt class="text-sm text-gray-500">"Diperbarui"</dt>
                                                    <dd class="text-sm text-gray-700">{group.updated_at.clone()}</dd>
                                                </div>
                                            </dl>
                                        </div>
                                    </div>
                                }
                            })}
                        </Show>
                    </div>
                </div>

                // Create group modal
                <Show when=move || show_create_modal.get()>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                            <h2 class="text-lg font-bold text-gray-900 mb-4">"Buat Grup Baru"</h2>
                            <form on:submit=move |ev: web_sys::SubmitEvent| { ev.prevent_default(); set_create_trigger.set(create_trigger.get() + 1); } class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Grup *"</label>
                                    <input
                                        type="text"
                                        prop:value=new_name
                                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                                        required=true
                                        placeholder="contoh: Divisi IT"
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Deskripsi"</label>
                                    <textarea
                                        prop:value=new_description
                                        on:input=move |ev| set_new_description.set(event_target_value(&ev))
                                        rows=3
                                        placeholder="Deskripsi opsional..."
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    ></textarea>
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
                                        {move || if creating.get() { "Membuat..." } else { "Buat Grup" }}
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
