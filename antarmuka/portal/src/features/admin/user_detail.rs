//! User Detail Page (Admin)
//!
//! Tabbed interface for viewing and managing a single user.
//! 4 tabs: Details, Attributes, Credentials, Role Mappings — the old
//! Groups/Sessions tabs rendered hard-coded "empty" claims about real
//! data and were deleted with the #45 IAM trim. Role Mappings is a real
//! manager: assign/remove against /api/v1/iam/users/{id}/roles/{role_id}.
//! REQ-PORTAL-016

use crate::components::feedback::{ErrorBanner, LoadingPanel, SuccessBanner};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::async_load::load_value_once;
use crate::utils::authenc_api::{IamUser, RoleInfo, UpdateUserRequest};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Active tab enum
#[derive(Clone, Copy, PartialEq, Eq)]
enum UserTab {
    Details,
    Attributes,
    Credentials,
    RoleMappings,
}

impl UserTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Details => "Detail",
            Self::Attributes => "Atribut",
            Self::Credentials => "Kredensial",
            Self::RoleMappings => "Pemetaan Peran",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Details => "👤",
            Self::Attributes => "🏷",
            Self::Credentials => "🔑",
            Self::RoleMappings => "🛡",
        }
    }

    fn all() -> &'static [UserTab] {
        &[
            Self::Details,
            Self::Attributes,
            Self::Credentials,
            Self::RoleMappings,
        ]
    }
}

/// User detail page — takes user ID from path
#[component]
pub fn UserDetailPage() -> impl IntoView {
    let api = use_api_client();

    // Extract user ID from URL path
    let params = leptos_router::hooks::use_params_map();
    let user_id = move || params.get().get("id").unwrap_or_default();

    let (user, set_user) = signal(Option::<IamUser>::None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (active_tab, set_active_tab) = signal(UserTab::Details);

    // All platform roles (for the Role Mappings manager) + in-flight flag
    // so the assign/remove buttons can't double-fire.
    let (all_roles, set_all_roles) = signal(Vec::<RoleInfo>::new());
    let (role_busy, set_role_busy) = signal(false);

    // Editable fields (synced from user signal on load)
    let (edit_email, set_edit_email) = signal(String::new());
    let (edit_first_name, set_edit_first_name) = signal(String::new());
    let (edit_last_name, set_edit_last_name) = signal(String::new());
    let (edit_enabled, set_edit_enabled) = signal(true);
    let (saving, set_saving) = signal(false);

    // Load user
    let load_user = {
        let api = api.clone();
        move || {
            let api = api.clone();
            let uid = user_id();
            load_value_once(
                move || {
                    let api = api.clone();
                    async move { api.iam_get_user(&uid).await.map(Some) }
                },
                set_user,
                set_error,
                set_loading,
                "Gagal memuat data pengguna",
            );
        }
    };

    // Keep form fields in sync with loaded user data.
    Effect::new(move || {
        if let Some(u) = user.get() {
            set_edit_email.set(u.email);
            set_edit_first_name.set(u.first_name.unwrap_or_default());
            set_edit_last_name.set(u.last_name.unwrap_or_default());
            set_edit_enabled.set(u.enabled);
        }
    });

    // Initial load
    {
        let load = load_user.clone();
        Effect::new(move || {
            let _ = user_id();
            load();
        });
    }

    // Save handler
    let (save_trigger, set_save_trigger) = signal(0u32);
    {
        let api = api.clone();
        let load = load_user.clone();
        Effect::new(move || {
            let count = save_trigger.get();
            if count == 0 {
                return;
            }
            let api = api.clone();
            let uid = user_id();
            let load = load.clone();
            set_saving.set(true);
            set_error.set(None);

            spawn_local(async move {
                let req = UpdateUserRequest {
                    email: Some(edit_email.get()),
                    first_name: Some(edit_first_name.get()),
                    last_name: Some(edit_last_name.get()),
                    enabled: Some(edit_enabled.get()),
                };
                match api.iam_update_user(&uid, &req).await {
                    Ok(_) => {
                        set_success.set(Some("Pengguna berhasil diperbarui".to_string()));
                        load();
                    }
                    Err(e) => set_error.set(Some(format!("Gagal memperbarui: {}", e))),
                }
                set_saving.set(false);
            });
        });
    }

    // Reset password handler
    let (reset_trigger, set_reset_trigger) = signal(0u32);
    {
        let api = api.clone();
        Effect::new(move || {
            let count = reset_trigger.get();
            if count == 0 {
                return;
            }
            let api = api.clone();
            let uid = user_id();
            spawn_local(async move {
                match api.iam_reset_user_password(&uid).await {
                    Ok(_) => set_success.set(Some("Password berhasil direset".to_string())),
                    Err(e) => set_error.set(Some(format!("Gagal reset password: {}", e))),
                }
            });
        });
    }

    // Load the platform role list once (for the Role Mappings manager).
    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.iam_list_roles().await {
                    Ok(roles) => set_all_roles.set(roles),
                    Err(e) => set_error.set(Some(format!("Gagal memuat daftar peran: {}", e))),
                }
            });
        });
    }

    // Assign/remove a role, then reload the user so the assigned set
    // reflects what the backend actually persisted. Callback so the view
    // closures stay `Fn` (Callback is Copy).
    let toggle_role = {
        let api = api.clone();
        let load = load_user.clone();
        Callback::new(move |(role_id, currently_assigned): (String, bool)| {
            if role_busy.get_untracked() {
                return;
            }
            let api = api.clone();
            let load = load.clone();
            let uid = user_id();
            set_role_busy.set(true);
            set_error.set(None);
            spawn_local(async move {
                let result = if currently_assigned {
                    api.iam_remove_role(&uid, &role_id).await
                } else {
                    api.iam_assign_role(&uid, &role_id).await
                };
                match result {
                    Ok(()) => {
                        set_success.set(Some(if currently_assigned {
                            "Peran berhasil dicabut".to_string()
                        } else {
                            "Peran berhasil ditetapkan".to_string()
                        }));
                        load();
                    }
                    Err(e) => set_error.set(Some(format!("Gagal mengubah peran: {}", e))),
                }
                set_role_busy.set(false);
            });
        })
    };

    // Derived signals for user info (avoids FnOnce issue)
    let display_name = move || {
        user.get()
            .map(|u| {
                let n = format!(
                    "{} {}",
                    u.first_name.clone().unwrap_or_default(),
                    u.last_name.clone().unwrap_or_default()
                )
                .trim()
                .to_string();
                if n.is_empty() { u.username.clone() } else { n }
            })
            .unwrap_or_default()
    };
    let user_username = move || user.get().map(|u| u.username.clone()).unwrap_or_default();
    let user_email = move || user.get().map(|u| u.email.clone()).unwrap_or_default();
    let user_id_str = move || user.get().map(|u| u.id.clone()).unwrap_or_default();
    let user_enabled = move || user.get().map(|u| u.enabled).unwrap_or(false);
    let user_mfa = move || user.get().map(|u| u.mfa_enabled).unwrap_or(false);
    let user_roles = move || user.get().map(|u| u.roles.clone()).unwrap_or_default();
    let user_roles_len = move || user_roles().len();
    let user_created = move || {
        lib_ui::utils::format_iso_local(
            &user.get().map(|u| u.created_at.clone()).unwrap_or_default(),
        )
    };
    let user_last_login = move || {
        user.get()
            .and_then(|u| u.last_login_at.clone())
            .map(|t| lib_ui::utils::format_iso_local(&t))
            .unwrap_or_else(|| "—".to_string())
    };
    let avatar_letter = move || {
        display_name()
            .chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .to_string()
    };

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-6xl mx-auto px-4 py-8">
                // Breadcrumb
                <nav class="text-sm text-gray-500 mb-4">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / "
                    <a href="/portal/admin/users" class="hover:text-primary-600">
                        "Pengguna"
                    </a>
                    " / Detail"
                </nav>

                {move || success.get().map(|msg| view! { <SuccessBanner message=msg /> })}
                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! { <LoadingPanel message="Memuat detail pengguna..." /> }
                >
                    <Show when=move || user.get().is_some()>
                        // User header card
                        <div class="bg-white rounded-xl border border-gray-200 overflow-hidden mb-6">
                            <div class="px-6 py-5 bg-gradient-to-r from-primary-50 via-blue-50 to-indigo-50">
                                <div class="flex items-center gap-4">
                                    <div class="w-14 h-14 rounded-full bg-primary-100 flex items-center justify-center text-2xl font-bold text-primary-700">
                                        {avatar_letter}
                                    </div>
                                    <div class="flex-1">
                                        <h1 class="text-xl font-bold text-gray-900">
                                            {display_name}
                                        </h1>
                                        <p class="text-sm text-gray-500">
                                            {user_username} " · " {user_email}
                                        </p>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        {move || {
                                            if user_enabled() {
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
                                        {move || {
                                            if user_mfa() {
                                                view! {
                                                    <span class="px-3 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-700">
                                                        "MFA"
                                                    </span>
                                                }
                                                    .into_any()
                                            } else {
                                                view! { <span></span> }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>

                            // Stats row
                            <div class="grid grid-cols-4 divide-x border-t">
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"ID Internal"</p>
                                    <p class="text-xs font-mono text-gray-700 truncate">
                                        {user_id_str}
                                    </p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Peran"</p>
                                    <p class="text-sm font-semibold text-gray-900">
                                        {user_roles_len}
                                    </p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Dibuat"</p>
                                    <p class="text-sm text-gray-700">{user_created}</p>
                                </div>
                                <div class="px-4 py-3 text-center">
                                    <p class="text-xs text-gray-500">"Login Terakhir"</p>
                                    <p class="text-sm text-gray-700">{user_last_login}</p>
                                </div>
                            </div>
                        </div>

                        // Tab navigation + content
                        <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                            <div class="border-b">
                                <nav class="flex overflow-x-auto px-2">
                                    {UserTab::all()
                                        .iter()
                                        .map(|tab| {
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
                                        })
                                        .collect::<Vec<_>>()}
                                </nav>
                            </div>

                            <div class="p-6">
                                // === Details Tab ===
                                <Show when=move || active_tab.get() == UserTab::Details>
                                    <form
                                        on:submit=move |ev: web_sys::SubmitEvent| {
                                            ev.prevent_default();
                                            set_save_trigger.set(save_trigger.get() + 1);
                                        }
                                        class="space-y-5"
                                    >
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                                    "Email"
                                                </label>
                                                <input
                                                    type="email"
                                                    prop:value=edit_email
                                                    on:input=move |ev| {
                                                        set_edit_email.set(event_target_value(&ev))
                                                    }
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                                    "Username"
                                                </label>
                                                <input
                                                    type="text"
                                                    prop:value=user_username
                                                    disabled=true
                                                    class="w-full px-3 py-2 border rounded-lg bg-gray-50 text-gray-500"
                                                />
                                            </div>
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                                    "Nama Depan"
                                                </label>
                                                <input
                                                    type="text"
                                                    prop:value=edit_first_name
                                                    on:input=move |ev| {
                                                        set_edit_first_name.set(event_target_value(&ev))
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
                                                    prop:value=edit_last_name
                                                    on:input=move |ev| {
                                                        set_edit_last_name.set(event_target_value(&ev))
                                                    }
                                                    class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                                />
                                            </div>
                                        </div>

                                        <div class="flex items-center gap-3">
                                            <label class="relative inline-flex items-center cursor-pointer">
                                                <input
                                                    type="checkbox"
                                                    prop:checked=edit_enabled
                                                    on:change=move |ev| {
                                                        let checked = event_target_checked(&ev);
                                                        set_edit_enabled.set(checked);
                                                    }
                                                    class="sr-only peer"
                                                />
                                                <div class="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
                                                <span class="ms-3 text-sm font-medium text-gray-700">
                                                    "Akun Aktif"
                                                </span>
                                            </label>
                                        </div>

                                        <div class="flex items-center gap-3 pt-2">
                                            <button
                                                type="submit"
                                                disabled=move || saving.get()
                                                class="px-5 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors"
                                            >
                                                {move || {
                                                    if saving.get() { "Menyimpan..." } else { "Simpan" }
                                                }}
                                            </button>
                                        </div>
                                    </form>
                                </Show>

                                // === Attributes Tab ===
                                <Show when=move || active_tab.get() == UserTab::Attributes>
                                    <div class="text-center py-8">
                                        <p class="text-3xl mb-3">"🏷"</p>
                                        <h3 class="text-lg font-semibold text-gray-700 mb-2">
                                            "Atribut Kustom"
                                        </h3>
                                        <p class="text-sm text-gray-500">
                                            "Atribut kustom pengguna untuk konfigurasi tambahan."
                                        </p>
                                        <div class="mt-6 bg-gray-50 rounded-lg p-4 inline-block">
                                            <p class="text-xs text-gray-400 font-mono">
                                                "Belum ada atribut kustom."
                                            </p>
                                        </div>
                                    </div>
                                </Show>

                                // === Credentials Tab ===
                                <Show when=move || active_tab.get() == UserTab::Credentials>
                                    <div class="space-y-6">
                                        <div class="bg-amber-50 border border-amber-200 rounded-lg p-4">
                                            <h4 class="font-semibold text-amber-800">
                                                "⚠ Manajemen Kredensial"
                                            </h4>
                                            <p class="text-sm text-amber-700 mt-1">
                                                "Perubahan kredensial akan berlaku segera. Pengguna akan diminta untuk login kembali."
                                            </p>
                                        </div>

                                        <div class="border rounded-lg p-5">
                                            <h4 class="font-medium text-gray-900 mb-3">
                                                "Reset Kata Sandi"
                                            </h4>
                                            <p class="text-sm text-gray-500 mb-4">
                                                "Kirim tautan reset kata sandi ke email pengguna atau tetapkan kata sandi baru secara langsung."
                                            </p>
                                            <button
                                                on:click=move |_| {
                                                    set_reset_trigger.set(reset_trigger.get() + 1)
                                                }
                                                class="px-4 py-2 bg-amber-500 text-white rounded-lg hover:bg-amber-600 transition-colors text-sm"
                                            >
                                                "🔄 Reset Kata Sandi"
                                            </button>
                                        </div>

                                        <div class="border rounded-lg p-5">
                                            <h4 class="font-medium text-gray-900 mb-3">
                                                "Multi-Factor Authentication"
                                            </h4>
                                            <div class="text-sm text-gray-500 mb-2 flex items-center gap-2">
                                                <span>"Status MFA:"</span>
                                                {move || {
                                                    if user_mfa() {
                                                        view! {
                                                            <span class="inline-flex px-2 py-0.5 text-xs rounded-full bg-green-100 text-green-700">
                                                                "Aktif"
                                                            </span>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        view! {
                                                            <span class="inline-flex px-2 py-0.5 text-xs rounded-full bg-gray-100 text-gray-600">
                                                                "Tidak Aktif"
                                                            </span>
                                                        }
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                </Show>

                                // === Role Mappings Tab ===
                                <Show when=move || active_tab.get() == UserTab::RoleMappings>
                                    <div class="space-y-4">
                                        <div class="flex items-center justify-between">
                                            <h4 class="font-medium text-gray-900">"Pemetaan Peran"</h4>
                                            <p class="text-xs text-gray-500">
                                                "Perubahan berlaku pada login berikutnya"
                                            </p>
                                        </div>
                                        {move || {
                                            let assigned = user_roles();
                                            let roles = all_roles.get();
                                            if roles.is_empty() {
                                                view! {
                                                    <div class="text-center py-8">
                                                        <p class="text-3xl mb-2">"🛡"</p>
                                                        <p class="text-sm text-gray-500">
                                                            "Daftar peran tidak tersedia."
                                                        </p>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-2">
                                                        {roles
                                                            .into_iter()
                                                            .map(|role| {
                                                                let is_assigned = assigned.contains(&role.name);
                                                                let role_id = role.id.clone();
                                                                view! {
                                                                    <div
                                                                        class="flex items-center justify-between px-4 py-3 bg-gray-50 rounded-lg"
                                                                        data-role-name=role.name.clone()
                                                                    >
                                                                        <div class="flex items-center gap-3">
                                                                            <span class="text-lg">"🛡"</span>
                                                                            <div>
                                                                                <span class="text-sm font-medium text-gray-900">
                                                                                    {role.name.clone()}
                                                                                </span>
                                                                                <p class="text-xs text-gray-500">
                                                                                    {role.description.clone().unwrap_or_default()}
                                                                                </p>
                                                                            </div>
                                                                        </div>
                                                                        <button
                                                                            class=if is_assigned {
                                                                                "px-3 py-1 text-sm rounded-lg border border-red-200 text-red-700 hover:bg-red-50 disabled:opacity-50"
                                                                            } else {
                                                                                "px-3 py-1 text-sm rounded-lg border border-primary-200 text-primary-700 hover:bg-primary-50 disabled:opacity-50"
                                                                            }
                                                                            disabled=move || role_busy.get()
                                                                            on:click=move |_| {
                                                                                toggle_role.run((role_id.clone(), is_assigned))
                                                                            }
                                                                        >
                                                                            {if is_assigned { "Cabut" } else { "Tetapkan" }}
                                                                        </button>
                                                                    </div>
                                                                }
                                                            })
                                                            .collect::<Vec<_>>()}
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                        }}
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
