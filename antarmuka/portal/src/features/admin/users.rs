//! Users Management Page (Admin)
//!
//! CRUD operations for user accounts with search, pagination, and filters.
//! REQ-PORTAL-010

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel, SuccessBanner};
use crate::components::layout::main_layout::MainLayout;
use crate::features::admin::temp_password;
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
    // All pagination comparisons live here, NOT inline in `view!`. leptosfmt
    // parses `view!` as markup and splits an attribute expression at the first
    // `>`, which silently rewrites the markup into something that STILL
    // COMPILES (verified) but renders wrong. `is_last_page` was mangled twice
    // that way; `is_first_page`/`has_multiple_pages` are the same hazard class
    // (`<=`, `>`) and are extracted for the same reason. See antarmuka/AGENTS.md.
    let is_last_page = Memo::new(move |_| page.get() >= total_pages.get());
    let is_first_page = Memo::new(move |_| page.get() <= 1);
    let has_multiple_pages = Memo::new(move |_| total_pages.get() > 1);
    let (show_create_modal, set_show_create_modal) = signal(false);

    // Form fields for new user
    let (new_username, set_new_username) = signal(String::new());
    let (new_email, set_new_email) = signal(String::new());
    let (new_first_name, set_new_first_name) = signal(String::new());
    let (new_last_name, set_new_last_name) = signal(String::new());
    let (new_nip, set_new_nip) = signal(String::new());
    let (new_jabatan, set_new_jabatan) = signal(String::new());
    let (new_satker, set_new_satker) = signal(String::new());
    let (creating, set_creating) = signal(false);
    // The password of the account just created. It exists only here, in memory:
    // shown once, never stored, gone when the admin closes the panel or leaves.
    let (temp_credential, set_temp_credential) = signal(Option::<TempCredential>::None);

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

            // The panel holds the ONLY copy of the previous account's password. It
            // is never replaced behind the admin's back: a second account cannot be
            // created until they have confirmed they copied it (the Add button is
            // disabled meanwhile, and this guards the form that was already open).
            if temp_credential.get_untracked().is_some() {
                set_creating.set(false);
                set_error.set(Some(
                    "Tutup panel password sementara di atas (setelah menyalinnya) sebelum menambah pengguna lain."
                        .to_string(),
                ));
                return;
            }

            spawn_local(async move {
                // One random password per account. No browser randomness, no
                // account: see `temp_password` for why there is no fallback.
                let Some(password) = temp_password::generate() else {
                    set_error.set(Some(
                        "Gagal membuat pengguna: browser ini tidak menyediakan pengacak yang aman, \
                         jadi password sementara tidak dapat dibuat. Gunakan browser yang lebih baru."
                            .to_string(),
                    ));
                    set_creating.set(false);
                    return;
                };
                let username = new_username.get();
                let req = CreateUserRequest {
                    username: username.clone(),
                    email: new_email.get(),
                    password: password.clone(),
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
                    nip: non_empty(&new_nip.get()),
                    jabatan: non_empty(&new_jabatan.get()),
                    satker_code: non_empty(&new_satker.get()),
                };

                match api.iam_create_user(&req).await {
                    Ok(created) => {
                        // The account now has a password that only the admin has
                        // seen. Make it temporary: the user must pick their own
                        // at the first login.
                        let must_change = UpdateUserRequest {
                            require_password_change: Some(true),
                            ..Default::default()
                        };
                        let enforced = match api.iam_update_user(&created.id, &must_change).await {
                            Ok(_) => true,
                            Err(e) => {
                                set_error.set(Some(format!(
                                    "Pengguna dibuat, tetapi kewajiban ganti password gagal \
                                     ditetapkan ({e}). Nonaktifkan akun ini lalu buat ulang."
                                )));
                                false
                            }
                        };
                        set_temp_credential.set(Some(TempCredential {
                            username,
                            password,
                            enforced,
                        }));
                        set_success.set(Some("Pengguna berhasil dibuat".to_string()));
                        set_show_create_modal.set(false);
                        set_new_username.set(String::new());
                        set_new_email.set(String::new());
                        set_new_first_name.set(String::new());
                        set_new_last_name.set(String::new());
                        set_new_nip.set(String::new());
                        set_new_jabatan.set(String::new());
                        set_new_satker.set(String::new());
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
                        enabled: Some(enable),
                        ..Default::default()
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
                        disabled=move || temp_credential.get().is_some()
                        title=move || {
                            if temp_credential.get().is_some() {
                                "Tutup panel password sementara dulu"
                            } else {
                                ""
                            }
                        }
                        class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        "+ Tambah Pengguna"
                    </button>
                </div>

                {move || success.get().map(|msg| view! { <SuccessBanner message=msg /> })}
                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}
                {move || {
                    temp_credential
                        .get()
                        .map(|c| {
                            view! {
                                <TempPasswordPanel
                                    credential=c
                                    on_close=Callback::new(move |_| set_temp_credential.set(None))
                                />
                            }
                        })
                }}

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
                        <Show when=move || has_multiple_pages.get()>
                            <div class="flex items-center justify-between px-4 py-3 border-t bg-gray-50">
                                <button
                                    on:click=move |_| {
                                        set_page.set(page.get().saturating_sub(1).max(1))
                                    }
                                    disabled=is_first_page
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
                                    // `>=` inside an attribute closure trips
                                    // leptosfmt, which splits the markup at the
                                    // `>` and turns the rest into text nodes
                                    // (it did exactly that here). Keep the
                                    // comparison out of the view macro.
                                    disabled=is_last_page
                                    class="px-3 py-1 text-sm border rounded disabled:opacity-50"
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
                        <div class="bg-white rounded-xl shadow-xl w-full max-w-md mx-4 p-6 max-h-[90dvh] overflow-y-auto">
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
                                <div class="grid grid-cols-2 gap-4">
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "NIP"
                                        </label>
                                        <input
                                            type="text"
                                            inputmode="numeric"
                                            data-testid="new-user-nip"
                                            prop:value=new_nip
                                            on:input=move |ev| set_new_nip.set(event_target_value(&ev))
                                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "Jabatan"
                                        </label>
                                        <input
                                            type="text"
                                            data-testid="new-user-jabatan"
                                            prop:value=new_jabatan
                                            on:input=move |ev| {
                                                set_new_jabatan.set(event_target_value(&ev))
                                            }
                                            class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                        />
                                    </div>
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Kode Satker"
                                    </label>
                                    <input
                                        type="text"
                                        data-testid="new-user-satker"
                                        prop:value=new_satker
                                        on:input=move |ev| set_new_satker.set(event_target_value(&ev))
                                        class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500"
                                    />
                                    <p class="mt-1 text-xs text-gray-500">
                                        "Kode satuan kerja MySIMKARI. Kode ini menentukan data yang dapat "
                                        "dilihat akun; tanpa kode, akun tidak terkait satker mana pun."
                                    </p>
                                </div>
                                <p class="text-sm text-gray-500">
                                    "Password sementara dibuat otomatis dan ditampilkan sekali setelah "
                                    "pengguna dibuat. Pengguna wajib menggantinya saat login pertama."
                                </p>
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

/// A form field as an optional value: blank (or only spaces) means "not given", so
/// the backend never stores an empty string where it expects NULL.
pub(crate) fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// A form field as the value to send on an edit: only what the admin actually
/// changed. The form is filled from the profile as it was when the page opened, so
/// sending an untouched field would put that old value back over anything someone
/// else saved in the meantime (a satker move, say). Blank still means "leave as it
/// is", which also makes a field the admin emptied a no-op.
pub(crate) fn changed(edited: &str, original: &str) -> Option<String> {
    non_empty(edited).filter(|value| value != original.trim())
}

/// The account just created (or reset) and the password the admin has to pass on.
#[derive(Clone)]
pub struct TempCredential {
    pub username: String,
    pub password: String,
    /// Whether the account was marked "must change at first login".
    pub enforced: bool,
}

/// Shows the one-time password. Deliberately stays until the admin dismisses it:
/// a banner that fades out would take the only copy with it.
#[component]
pub fn TempPasswordPanel(credential: TempCredential, on_close: Callback<()>) -> impl IntoView {
    let TempCredential {
        username,
        password,
        enforced,
    } = credential;
    view! {
        <div
            class="mb-4 p-4 bg-amber-50 border border-amber-300 rounded-lg"
            role="alert"
            data-testid="temp-password-panel"
        >
            <p class="font-semibold text-amber-900">
                "Password sementara untuk " <span class="font-mono">{username}</span>
            </p>
            <p class="mt-2 text-sm text-amber-900">
                "Salin dan sampaikan kepada pengguna sekarang. Password ini hanya ditampilkan "
                "sekali dan tidak dapat dilihat lagi setelah panel ditutup."
            </p>
            <code
                data-testid="temp-password"
                class="mt-2 inline-block select-all px-3 py-2 bg-white border border-amber-300 rounded font-mono text-lg tracking-wide"
            >
                {password}
            </code>
            <p class="mt-2 text-sm text-amber-900">
                {if enforced {
                    "Pengguna wajib menggantinya saat login pertama."
                } else {
                    "Peringatan: kewajiban ganti password belum tertetapkan untuk akun ini."
                }}
            </p>
            <button
                type="button"
                on:click=move |_| on_close.run(())
                class="mt-3 px-3 py-1 text-sm border border-amber-400 rounded hover:bg-amber-100"
            >
                "Saya sudah menyalin, tutup"
            </button>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::{changed, non_empty};

    #[test]
    fn blank_form_fields_are_not_given() {
        assert_eq!(non_empty(""), None);
        assert_eq!(non_empty("   "), None);
        assert_eq!(non_empty("\t\n"), None);
    }

    #[test]
    fn an_untouched_field_is_not_sent_back() {
        assert_eq!(changed("006019999010001KD", "006019999010001KD"), None);
        // Same value up to surrounding spaces is still untouched.
        assert_eq!(changed(" 006019999010001KD ", "006019999010001KD"), None);
        assert_eq!(changed("006019999010001KD", " 006019999010001KD"), None);
    }

    #[test]
    fn a_changed_field_is_sent_trimmed() {
        assert_eq!(
            changed(" 006019999020001KD ", "006019999010001KD"),
            Some("006019999020001KD".to_string())
        );
        // A field that was empty and is now filled is a change.
        assert_eq!(
            changed("Kepala Seksi", ""),
            Some("Kepala Seksi".to_string())
        );
    }

    #[test]
    fn an_emptied_field_is_left_alone() {
        assert_eq!(changed("", "006019999010001KD"), None);
        assert_eq!(changed("   ", "Kepala Seksi"), None);
    }

    #[test]
    fn given_form_fields_are_trimmed() {
        assert_eq!(
            non_empty("  199203142014031001 "),
            Some("199203142014031001".to_string())
        );
        assert_eq!(non_empty("Kepala Seksi"), Some("Kepala Seksi".to_string()));
    }
}
