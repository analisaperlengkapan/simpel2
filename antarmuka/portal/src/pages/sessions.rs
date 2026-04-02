//! Active Sessions Page
//!
//! View and manage active login sessions.
//! REQ-PORTAL-008

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::SessionInfo;
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Active sessions management page
#[component]
pub fn SessionsPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (sessions, set_sessions) = signal(Vec::<SessionInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (success, set_success) = signal(Option::<String>::None);
    let (terminating, set_terminating) = signal(Option::<String>::None);

    // Trigger signal for session termination
    let (terminate_trigger, set_terminate_trigger) = signal(Option::<String>::None);

    // Load sessions on mount
    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            spawn_local(async move {
                match api.list_sessions().await {
                    Ok(list) => set_sessions.set(list),
                    Err(e) => set_error.set(Some(format!("Gagal memuat sesi: {}", e))),
                }
                set_loading.set(false);
            });
        });
    }

    // Terminate session effect
    {
        let api = api.clone();
        Effect::new(move || {
            let session_id = terminate_trigger.get();
            if let Some(session_id) = session_id {
                let api = api.clone();
                set_terminating.set(Some(session_id.clone()));
                set_error.set(None);
                set_success.set(None);

                spawn_local(async move {
                    match api.terminate_session(&session_id).await {
                        Ok(()) => {
                            set_sessions.update(|list| list.retain(|s| s.id != session_id));
                            set_success.set(Some("Sesi berhasil dihentikan".to_string()));
                        }
                        Err(e) => set_error.set(Some(format!("Gagal menghentikan sesi: {}", e))),
                    }
                    set_terminating.set(None);
                });
            }
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
            <div class="max-w-4xl mx-auto px-4 py-8">
                <h1 class="text-2xl font-bold text-gray-900 mb-2">"Sesi Aktif"</h1>
                <p class="text-gray-600 mb-6">"Kelola perangkat dan sesi login yang aktif."</p>

                {move || success.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg text-green-700">
                        "✅ " {msg}
                    </div>
                })}
                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">
                        "❌ " {msg}
                    </div>
                })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! {
                        <div class="flex items-center justify-center py-12">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
                            <span class="ml-3 text-gray-500">"Memuat sesi..."</span>
                        </div>
                    }
                >
                    <Show
                        when=move || !sessions.get().is_empty()
                        fallback=|| view! {
                            <div class="text-center py-12 bg-white rounded-xl border border-gray-200">
                                <p class="text-4xl mb-3">"📱"</p>
                                <p class="text-gray-500">"Tidak ada sesi aktif ditemukan."</p>
                            </div>
                        }
                    >
                        <div class="space-y-3">
                            <For
                                each=move || sessions.get()
                                key=|s| s.id.clone()
                                children=move |session_info| {
                                    let sid = session_info.id.clone();
                                    let sid2 = session_info.id.clone();
                                    let is_current = session_info.is_current;
                                    let device = session_info.user_agent.clone().unwrap_or_else(|| "Perangkat tidak dikenal".to_string());
                                    let ip = session_info.ip_address.clone().unwrap_or_else(|| "-".to_string());
                                    let location = session_info.location.clone().unwrap_or_else(|| "Lokasi tidak diketahui".to_string());
                                    let last_active = session_info.last_active.clone();
                                    let created = session_info.created_at.clone();

                                    let device_icon = if device.to_lowercase().contains("mobile") || device.to_lowercase().contains("android") || device.to_lowercase().contains("iphone") {
                                        "📱"
                                    } else if device.to_lowercase().contains("tablet") || device.to_lowercase().contains("ipad") {
                                        "📱"
                                    } else {
                                        "💻"
                                    };

                                    view! {
                                        <div class={format!(
                                            "bg-white rounded-xl border {} p-4 flex items-start gap-4",
                                            if is_current { "border-primary-300 ring-1 ring-primary-100" } else { "border-gray-200" }
                                        )}>
                                            <span class="text-2xl mt-1">{device_icon}</span>
                                            <div class="flex-1 min-w-0">
                                                <div class="flex items-center gap-2">
                                                    <h3 class="font-medium text-gray-900 truncate">{device.clone()}</h3>
                                                    {is_current.then(|| view! {
                                                        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700">
                                                            "Sesi ini"
                                                        </span>
                                                    })}
                                                </div>
                                                <div class="text-sm text-gray-500 mt-1 space-y-0.5">
                                                    <p>"IP: " {ip}</p>
                                                    <p>"Lokasi: " {location}</p>
                                                    <p>"Login: " {created}</p>
                                                    <p>"Aktif terakhir: " {last_active}</p>
                                                </div>
                                            </div>
                                            <div class="flex-shrink-0">
                                                {if !is_current {
                                                    let sid2_disabled = sid2.clone();
                                                    let sid2_label = sid2;
                                                    Some(view! {
                                                        <button
                                                            on:click=move |_| {
                                                                set_terminate_trigger.set(Some(sid.clone()));
                                                            }
                                                            disabled=move || terminating.get().as_deref() == Some(&sid2_disabled)
                                                            class="px-3 py-1.5 text-sm text-red-600 border border-red-200 rounded-lg hover:bg-red-50 disabled:opacity-50 transition-colors"
                                                        >
                                                            {move || {
                                                                if terminating.get().as_deref() == Some(sid2_label.as_str()) {
                                                                    "..."
                                                                } else {
                                                                    "Hentikan"
                                                                }
                                                            }}
                                                        </button>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </Show>

                // Info box
                <div class="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg text-sm text-blue-700">
                    <p class="font-medium mb-1">"ℹ️ Tentang Sesi"</p>
                    <p>"Jika Anda melihat sesi yang tidak Anda kenali, segera hentikan sesi tersebut dan ubah kata sandi Anda."</p>
                </div>
            </div>
        </MainLayout>
    }
}
