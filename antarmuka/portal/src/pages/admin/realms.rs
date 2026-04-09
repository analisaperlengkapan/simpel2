//! Realms Management Page (Admin)
//!
//! Manage multi-tenant realms (create, edit, delete).
//! REQ-PORTAL-011

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::{CreateRealmRequest, MASTER_REALM_ID, RealmInfo};
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
    let (success, set_success) = signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = signal(false);
    let (realm_to_delete, set_realm_to_delete) = signal(Option::<RealmInfo>::None);
    let (realm_to_edit, set_realm_to_edit) = signal(Option::<RealmInfo>::None);

    // Form fields for new realm
    let (new_name, set_new_name) = signal(String::new());
    let (new_display_name, set_new_display_name) = signal(String::new());
    let (creating, set_creating) = signal(false);
    let (deleting, set_deleting) = signal(false);

    // Client-side validation for realm name
    let new_name_error = move || {
        let name = new_name.get();
        if name.is_empty() {
            return None;
        }
        if name.len() < 3 {
            return Some("Minimal 3 karakter".to_string());
        }
        if name.len() > 64 {
            return Some("Maksimal 64 karakter".to_string());
        }
        if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Some("Harus diawali huruf kecil".to_string());
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
        {
            return Some("Hanya huruf kecil, angka, garis bawah, dan tanda hubung".to_string());
        }
        None
    };
    let is_name_valid = move || !new_name.get().is_empty() && new_name_error().is_none();

    // Form fields for editing realm
    let (edit_display_name, set_edit_display_name) = signal(String::new());
    let (edit_enabled, set_edit_enabled) = signal(true);
    let (updating, set_updating) = signal(false);

    let load_realms = StoredValue::new_local({
        let api = api.clone();
        move || {
            let api = api.clone();
            set_loading.set(true);
            spawn_local(async move {
                match api.iam_list_realms().await {
                    Ok(list) => set_realms.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat realm: {}", e))),
                }
                set_loading.set(false);
            });
        }
    });

    Effect::new(move |_| {
        load_realms.with_value(|f| f());
    });

    let handle_create = StoredValue::new_local({
        let api = api.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let api = api.clone();
            set_creating.set(true);
            set_error.set(None);
            set_success.set(None);

            spawn_local(async move {
                let req = CreateRealmRequest {
                    name: new_name.get(),
                    display_name: if new_display_name.get().is_empty() {
                        None
                    } else {
                        Some(new_display_name.get())
                    },
                };

                match api.iam_create_realm(&req).await {
                    Ok(_) => {
                        set_success.set(Some("Realm berhasil dibuat".to_string()));
                        set_show_create_modal.set(false);
                        set_new_name.set(String::new());
                        set_new_display_name.set(String::new());
                        load_realms.with_value(|f| f());
                    }
                    Err(e) => set_error.set(Some(format!("Gagal membuat realm: {}", e))),
                }
                set_creating.set(false);
            });
        }
    });

    let handle_delete = StoredValue::new_local({
        let api = api.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(realm) = realm_to_delete.get() {
                let api = api.clone();
                let id = realm.id.clone();
                set_deleting.set(true);
                set_error.set(None);
                set_success.set(None);

                spawn_local(async move {
                    match api.iam_delete_realm(&id).await {
                        Ok(_) => {
                            set_success.set(Some("Realm berhasil dihapus".to_string()));
                            set_realm_to_delete.set(None);
                            load_realms.with_value(|f| f());
                        }
                        Err(e) => set_error.set(Some(format!("Gagal menghapus realm: {}", e))),
                    }
                    set_deleting.set(false);
                });
            }
        }
    });

    let handle_update = StoredValue::new_local({
        let api = api.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            if let Some(realm) = realm_to_edit.get() {
                let api = api.clone();
                let id = realm.id.clone();
                set_updating.set(true);
                set_error.set(None);
                set_success.set(None);

                spawn_local(async move {
                    let req = crate::utils::authenc_api::UpdateRealmRequest {
                        display_name: if edit_display_name.get().is_empty() {
                            None
                        } else {
                            Some(edit_display_name.get())
                        },
                        enabled: Some(edit_enabled.get()),
                    };

                    match api.iam_update_realm(&id, &req).await {
                        Ok(_) => {
                            set_success.set(Some("Realm berhasil diperbarui".to_string()));
                            set_realm_to_edit.set(None);
                            load_realms.with_value(|f| f());
                        }
                        Err(e) => set_error.set(Some(format!("Gagal memperbarui realm: {}", e))),
                    }
                    set_updating.set(false);
                });
            }
        }
    });

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
                    " / Realm"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Realm"</h1>
                    <button
                        on:click=move |_| set_show_create_modal.set(true)
                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors flex items-center gap-2"
                    >
                        "＋ Buat Realm"
                    </button>
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
                                let r_clone = realm.clone();
                                view! {
                                    <div class="bg-white rounded-xl border border-gray-200 p-5 group relative">
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
                                        <div class="flex items-center justify-between">
                                            <div class="text-xs text-gray-400">
                                                "ID: " {realm.id.clone()}
                                            </div>
                                            <div class="flex gap-1">
                                                <button
                                                    on:click=move |_| {
                                                        set_edit_display_name.set(r_clone.display_name.clone().unwrap_or_default());
                                                        set_edit_enabled.set(r_clone.enabled);
                                                        set_realm_to_edit.set(Some(r_clone.clone()));
                                                    }
                                                    class="opacity-0 group-hover:opacity-100 p-1.5 text-primary-600 hover:bg-primary-50 rounded-lg transition-all"
                                                    title="Edit Realm"
                                                >
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                    </svg>
                                                </button>
                                                {if realm.id != MASTER_REALM_ID {
                                                    let r_del = realm.clone();
                                                    Some(view! {
                                                        <button
                                                            on:click=move |_| set_realm_to_delete.set(Some(r_del.clone()))
                                                            class="opacity-0 group-hover:opacity-100 p-1.5 text-red-600 hover:bg-red-50 rounded-lg transition-all"
                                                            title="Hapus Realm"
                                                        >
                                                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                            </svg>
                                                        </button>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
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

                // Create realm modal
                <Show when=move || show_create_modal.get()>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                            <h2 class="text-lg font-bold text-gray-900 mb-4">"Buat Realm Baru"</h2>
                            <form
                                on:submit=move |ev| handle_create.with_value(|f| f(ev))
                                class="space-y-4"
                            >
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Realm *"</label>
                                    <input
                                        type="text"
                                        prop:value=new_name
                                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                                        required=true
                                        placeholder="contoh: kejaksaan-agung"
                                        class=move || if new_name_error().is_some() {
                                            "w-full px-3 py-2 border border-red-300 rounded-lg focus:ring-2 focus:ring-red-500"
                                        } else {
                                            "w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                        }
                                    />
                                    {move || match new_name_error() {
                                        Some(msg) => view! { <p class="text-xs text-red-500 mt-1">{msg}</p> }.into_any(),
                                        None => view! { <p class="text-xs text-gray-400 mt-1">"Hanya huruf kecil, angka, garis bawah, dan tanda hubung."</p> }.into_any(),
                                    }}
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Tampilan"</label>
                                    <input
                                        type="text"
                                        prop:value=new_display_name
                                        on:input=move |ev| set_new_display_name.set(event_target_value(&ev))
                                        placeholder="contoh: Kejaksaan Agung RI"
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
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
                                        disabled=move || creating.get() || !is_name_valid()
                                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
                                    >
                                        {move || if creating.get() { "Membuat..." } else { "Buat Realm" }}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                </Show>

                // Delete confirmation modal
                <Show when=move || realm_to_delete.get().is_some()>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                            <div class="flex items-center gap-3 text-red-600 mb-4">
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                                <h2 class="text-lg font-bold">"Hapus Realm"</h2>
                            </div>
                            <p class="text-gray-600 mb-6">
                                "Apakah Anda yakin ingin menghapus realm "
                                <span class="font-bold text-gray-900">{move || realm_to_delete.get().map(|r| r.name).unwrap_or_default()}</span>
                                "? Realm akan dinonaktifkan dan tidak dapat diakses lagi. Data terkait akan diarsipkan dan dapat dipulihkan oleh administrator sistem jika diperlukan."
                            </p>
                            <div class="flex justify-end gap-3">
                                <button
                                    on:click=move |_| set_realm_to_delete.set(None)
                                    class="px-4 py-2 text-gray-700 border rounded-lg hover:bg-gray-50"
                                >
                                    "Batal"
                                </button>
                                <button
                                    on:click=move |ev| handle_delete.with_value(|f| f(ev))
                                    disabled=deleting
                                    class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50"
                                >
                                    {move || if deleting.get() { "Menghapus..." } else { "Ya, Hapus Realm" }}
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>

                // Edit realm modal
                <Show when=move || realm_to_edit.get().is_some()>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6">
                            <h2 class="text-lg font-bold text-gray-900 mb-4">"Edit Realm"</h2>
                            <form
                                on:submit=move |ev| handle_update.with_value(|f| f(ev))
                                class="space-y-4"
                            >
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Realm"</label>
                                    <input
                                        type="text"
                                        value=move || realm_to_edit.get().map(|r| r.name).unwrap_or_default()
                                        disabled=true
                                        class="w-full px-3 py-2 border rounded-lg bg-gray-50 text-gray-500 cursor-not-allowed"
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">"Nama Tampilan"</label>
                                    <input
                                        type="text"
                                        prop:value=edit_display_name
                                        on:input=move |ev| set_edit_display_name.set(event_target_value(&ev))
                                        placeholder="contoh: Kejaksaan Agung RI"
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
                                </div>
                                <div class="flex items-center gap-3">
                                    <label class="relative inline-flex items-center cursor-pointer">
                                        <input
                                            type="checkbox"
                                            prop:checked=edit_enabled
                                            on:change=move |ev| set_edit_enabled.set(event_target_checked(&ev))
                                            disabled=move || realm_to_edit.get().map(|r| r.id == MASTER_REALM_ID).unwrap_or(false)
                                            class="sr-only peer"
                                        />
                                        <div class="w-11 h-6 bg-gray-200 rounded-full peer peer-checked:bg-primary-600 after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-full peer-disabled:opacity-50 peer-disabled:cursor-not-allowed"></div>
                                        <span class="ms-3 text-sm font-medium text-gray-700">"Realm Aktif"</span>
                                    </label>
                                    <Show when=move || realm_to_edit.get().map(|r| r.id == MASTER_REALM_ID).unwrap_or(false)>
                                        <span class="text-xs text-amber-600">"Master realm tidak dapat dinonaktifkan"</span>
                                    </Show>
                                </div>
                                <div class="flex justify-end gap-3 pt-2">
                                    <button
                                        type="button"
                                        on:click=move |_| set_realm_to_edit.set(None)
                                        class="px-4 py-2 text-gray-700 border rounded-lg hover:bg-gray-50"
                                    >
                                        "Batal"
                                    </button>
                                    <button
                                        type="submit"
                                        disabled=updating
                                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
                                    >
                                        {move || if updating.get() { "Menyimpan..." } else { "Simpan Perubahan" }}
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
