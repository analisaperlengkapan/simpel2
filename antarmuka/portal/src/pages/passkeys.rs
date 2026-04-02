//! Passkeys Management Page (MANDATORY)
//!
//! Allows users to view, add, delete, and rename their WebAuthn passkeys.
//! This is a PRIMARY authentication method per REQ-PORTAL-009 / REQ-WEBAUTHN-003.

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_app_state};
use crate::utils::authenc_api::{PasskeyInfo, UpdatePasskeyRequest};
use crate::utils::webauthn;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Passkeys management page
#[component]
pub fn PasskeysPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (passkeys, set_passkeys) = signal(Vec::<PasskeyInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (registering, set_registering) = signal(false);
    let (editing_id, set_editing_id) = signal(Option::<String>::None);
    let (edit_nickname, set_edit_nickname) = signal(String::new());
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (confirm_delete, set_confirm_delete) = signal(Option::<String>::None);

    let webauthn_supported = webauthn::is_webauthn_supported();

    // Trigger signals to replace closures that cause FnOnce issues
    let (register_trigger, set_register_trigger) = signal(0u32);
    let (delete_trigger, set_delete_trigger) = signal(Option::<String>::None);
    let (save_nick_trigger, set_save_nick_trigger) = signal(Option::<String>::None);

    // Load passkeys
    let api_load = api.clone();
    let load_passkeys = move || {
        let api = api_load.clone();
        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);
            match api.webauthn_list_credentials().await {
                Ok(creds) => set_passkeys.set(creds),
                Err(e) => set_error.set(Some(format!("Gagal memuat passkey: {}", e))),
            }
            set_loading.set(false);
        });
    };

    // Initial load
    load_passkeys();

    // Register effect - triggered by register_trigger signal
    {
        let api = api.clone();
        let reload = load_passkeys.clone();
        Effect::new(move || {
            let count = register_trigger.get();
            if count == 0 {
                return;
            } // skip initial
            let api = api.clone();
            let reload = reload.clone();
            set_registering.set(true);
            set_error.set(None);
            set_success_msg.set(None);

            spawn_local(async move {
                match api.webauthn_register_start().await {
                    Ok(start_resp) => {
                        // Pass challenge to browser WebAuthn API
                        match webauthn::create_credential(&start_resp.challenge).await {
                            Ok(credential) => {
                                // Send session_id + credential to finish endpoint
                                match api
                                    .webauthn_register_finish(&start_resp.session_id, &credential)
                                    .await
                                {
                                    Ok(info) => {
                                        let name =
                                            info.nickname.as_deref().unwrap_or("Passkey baru");
                                        set_success_msg.set(Some(format!(
                                            "Passkey '{}' berhasil didaftarkan!",
                                            name
                                        )));
                                        reload();
                                    }
                                    Err(e) => {
                                        set_error.set(Some(format!(
                                            "Gagal mendaftarkan passkey: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                            Err(e) => {
                                set_error.set(Some(format!("Pendaftaran dibatalkan: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Gagal memulai pendaftaran: {}", e)));
                    }
                }
                set_registering.set(false);
            });
        });
    }

    // Delete effect - triggered by delete_trigger signal
    {
        let api = api.clone();
        let reload = load_passkeys.clone();
        Effect::new(move || {
            let id = delete_trigger.get();
            if let Some(id) = id {
                let api = api.clone();
                let reload = reload.clone();
                set_confirm_delete.set(None);
                set_error.set(None);

                spawn_local(async move {
                    match api.webauthn_delete_credential(&id).await {
                        Ok(()) => {
                            set_success_msg.set(Some("Passkey berhasil dihapus".to_string()));
                            reload();
                        }
                        Err(e) => set_error.set(Some(format!("Gagal menghapus passkey: {}", e))),
                    }
                });
            }
        });
    }

    // Save nickname effect - triggered by save_nick_trigger signal
    {
        let api = api.clone();
        let reload = load_passkeys.clone();
        Effect::new(move || {
            let id = save_nick_trigger.get();
            if let Some(id) = id {
                let api = api.clone();
                let reload = reload.clone();
                let nickname = edit_nickname.get();
                set_editing_id.set(None);
                set_error.set(None);

                spawn_local(async move {
                    match api
                        .webauthn_update_credential(&id, &UpdatePasskeyRequest { nickname })
                        .await
                    {
                        Ok(()) => {
                            set_success_msg
                                .set(Some("Nama passkey berhasil diperbarui".to_string()));
                            reload();
                        }
                        Err(e) => set_error.set(Some(format!("Gagal memperbarui nama: {}", e))),
                    }
                });
            }
        });
    }

    let on_logout = {
        let state = state;
        Box::new(move || {
            crate::features::auth::AuthService::logout();
            state.set(crate::utils::app_state::AppState::default());
        }) as Box<dyn Fn()>
    };

    let session = state.get().user.unwrap_or_default();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-4xl mx-auto px-4 py-8">
                // Header
                <div class="flex items-center justify-between mb-6">
                    <div>
                        <h1 class="text-2xl font-bold text-gray-900">"Kelola Passkey"</h1>
                        <p class="text-gray-600 mt-1">
                            "Passkey memungkinkan Anda login dengan sidik jari, wajah, atau kunci keamanan tanpa kata sandi."
                        </p>
                    </div>
                    {if webauthn_supported {
                        view! {
                            <button
                                on:click=move |_| set_register_trigger.set(register_trigger.get() + 1)
                                disabled=registering
                                class="inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            >
                                {move || if registering.get() {
                                    view! {
                                        <span class="animate-spin inline-block w-4 h-4 border-2 border-white border-t-transparent rounded-full"></span>
                                        <span>"Mendaftarkan..."</span>
                                    }.into_any()
                                } else {
                                    view! {
                                        <span>"🔑"</span>
                                        <span>"Tambah Passkey"</span>
                                    }.into_any()
                                }}
                            </button>
                        }.into_any()
                    } else {
                        view! {
                            <div class="text-sm text-yellow-600 bg-yellow-50 border border-yellow-200 rounded-lg px-3 py-2">
                                "⚠️ Browser Anda tidak mendukung WebAuthn/Passkey"
                            </div>
                        }.into_any()
                    }}
                </div>

                // Success message
                {move || success_msg.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700 flex items-center gap-2">
                        <span>"✅"</span>
                        <span>{msg}</span>
                        <button
                            on:click=move |_| set_success_msg.set(None)
                            class="ml-auto text-green-500 hover:text-green-700"
                        >"✕"</button>
                    </div>
                })}

                // Error message
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 flex items-center gap-2">
                        <span>"❌"</span>
                        <span>{msg}</span>
                        <button
                            on:click=move |_| set_error.set(None)
                            class="ml-auto text-red-500 hover:text-red-700"
                        >"✕"</button>
                    </div>
                })}

                // Passkey list
                <div class="bg-white rounded-xl shadow-sm border border-gray-200">
                    {move || {
                        if loading.get() {
                            view! {
                                <div class="p-8 text-center">
                                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-3"></div>
                                    <p class="text-gray-500">"Memuat passkey..."</p>
                                </div>
                            }.into_any()
                        } else if passkeys.get().is_empty() {
                            view! {
                                <div class="p-12 text-center">
                                    <div class="text-5xl mb-4">"🔐"</div>
                                    <h3 class="text-lg font-semibold text-gray-900 mb-2">"Belum Ada Passkey"</h3>
                                    <p class="text-gray-600 max-w-md mx-auto mb-6">
                                        "Passkey memungkinkan login yang lebih aman dan cepat menggunakan "
                                        "sidik jari, pengenalan wajah, atau kunci keamanan fisik."
                                    </p>
                                    {if webauthn_supported {
                                        view! {
                                            <button
                                                on:click=move |_| set_register_trigger.set(register_trigger.get() + 1)
                                                disabled=registering
                                                class="inline-flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 transition-colors"
                                            >
                                                <span>"🔑"</span>
                                                <span>"Daftarkan Passkey Pertama"</span>
                                            </button>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            let keys = passkeys.get();
                            view! {
                                <div class="divide-y divide-gray-200">
                                    {keys.into_iter().map(|pk| {
                                        let pk_id = pk.id.clone();
                                        let pk_id_del = pk.id.clone();
                                        let pk_id_edit = pk.id.clone();
                                        let pk_id_save = pk.id.clone();
                                        let pk_nickname = pk.nickname.clone().unwrap_or_else(|| "Passkey".to_string());
                                        let pk_nickname_edit = pk_nickname.clone();
                                        let is_editing = {
                                            let pk_id = pk_id.clone();
                                            move || editing_id.get().as_deref() == Some(&pk_id)
                                        };

                                        view! {
                                            <div class="flex items-center justify-between p-4 hover:bg-gray-50 transition-colors">
                                                <div class="flex items-center gap-4">
                                                    // Icon: platform vs security key
                                                    <div class="flex-shrink-0 w-10 h-10 rounded-full bg-primary-50 flex items-center justify-center">
                                                        {if pk.is_platform {
                                                            view! { <span class="text-xl">"👆"</span> }.into_any()
                                                        } else {
                                                            view! { <span class="text-xl">"🔑"</span> }.into_any()
                                                        }}
                                                    </div>

                                                    <div>
                                                        {move || {
                                                            if is_editing() {
                                                                view! {
                                                                    <div class="flex items-center gap-2">
                                                                        <input
                                                                            type="text"
                                                                            prop:value=edit_nickname
                                                                            on:input=move |ev| {
                                                                                set_edit_nickname.set(event_target_value(&ev));
                                                                            }
                                                                            class="px-2 py-1 border border-gray-300 rounded text-sm focus:ring-primary-500 focus:border-primary-500"
                                                                            autofocus=true
                                                                        />
                                                                        <button
                                                                            on:click={
                                                                                let pk_id = pk_id_save.clone();
                                                                                move |_| set_save_nick_trigger.set(Some(pk_id.clone()))
                                                                            }
                                                                            class="text-primary-600 hover:text-primary-700 text-sm font-medium"
                                                                        >"Simpan"</button>
                                                                        <button
                                                                            on:click=move |_| set_editing_id.set(None)
                                                                            class="text-gray-500 hover:text-gray-700 text-sm"
                                                                        >"Batal"</button>
                                                                    </div>
                                                                }.into_any()
                                                            } else {
                                                                let nick = pk_nickname.clone();
                                                                view! {
                                                                    <h4 class="font-medium text-gray-900">{nick}</h4>
                                                                }.into_any()
                                                            }
                                                        }}
                                                        <div class="flex items-center gap-3 text-xs text-gray-500 mt-1">
                                                            <span>
                                                                {if pk.is_platform { "Biometrik" } else { "Kunci keamanan" }}
                                                            </span>
                                                            <span>"•"</span>
                                                            <span>"Dibuat: " {pk.created_at.clone()}</span>
                                                            {pk.last_used.clone().map(|lu| view! {
                                                                <span>"•"</span>
                                                                <span>"Terakhir: " {lu}</span>
                                                            })}
                                                        </div>
                                                    </div>
                                                </div>

                                                // Actions
                                                <div class="flex items-center gap-2">
                                                    <button
                                                        on:click={
                                                            let pk_id = pk_id_edit.clone();
                                                            let nick = pk_nickname_edit.clone();
                                                            move |_| {
                                                                set_editing_id.set(Some(pk_id.clone()));
                                                                set_edit_nickname.set(nick.clone());
                                                            }
                                                        }
                                                        class="p-2 text-gray-400 hover:text-primary-600 rounded-lg hover:bg-primary-50 transition-colors"
                                                        title="Ubah nama"
                                                    >
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        on:click={
                                                            let pk_id = pk_id_del.clone();
                                                            move |_| set_confirm_delete.set(Some(pk_id.clone()))
                                                        }
                                                        class="p-2 text-gray-400 hover:text-red-600 rounded-lg hover:bg-red-50 transition-colors"
                                                        title="Hapus passkey"
                                                    >
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                                        </svg>
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Info box
                <div class="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
                    <h4 class="font-medium text-blue-900 mb-2">"ℹ️ Tentang Passkey"</h4>
                    <ul class="text-sm text-blue-800 space-y-1 list-disc list-inside">
                        <li>"Passkey menggantikan kata sandi dengan autentikasi biometrik atau kunci keamanan"</li>
                        <li>"Lebih aman dari kata sandi - tahan terhadap phishing"</li>
                        <li>"Tersinkronisasi otomatis melalui iCloud, Google Password Manager, dsb."</li>
                        <li>"Disarankan mendaftarkan setidaknya 2 passkey untuk cadangan"</li>
                    </ul>
                </div>

                // Delete confirmation modal
                {move || confirm_delete.get().map(|id| {
                    let id_for_delete = id.clone();
                    view! {
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                            <div class="bg-white rounded-xl shadow-2xl p-6 max-w-sm w-full mx-4">
                                <div class="text-center">
                                    <div class="text-4xl mb-3">"⚠️"</div>
                                    <h3 class="text-lg font-semibold text-gray-900 mb-2">"Hapus Passkey?"</h3>
                                    <p class="text-gray-600 text-sm mb-6">
                                        "Passkey ini akan dihapus secara permanen. "
                                        "Anda tidak akan bisa menggunakannya untuk login lagi."
                                    </p>
                                    <div class="flex gap-3 justify-center">
                                        <button
                                            on:click=move |_| set_confirm_delete.set(None)
                                            class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                                        >"Batal"</button>
                                        <button
                                            on:click=move |_| set_delete_trigger.set(Some(id_for_delete.clone()))
                                            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                                        >"Hapus"</button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                })}
            </div>
        </MainLayout>
    }
}
