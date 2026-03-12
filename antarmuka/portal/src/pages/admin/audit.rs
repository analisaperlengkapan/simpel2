//! Audit Logs Page (Admin)
//!
//! View, search, and filter security and activity audit logs.
//! REQ-PORTAL-015

use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{AppState, use_api_client, use_app_state};
use crate::utils::authenc_api::{AuditLogEntry, AuditLogQuery};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Audit logs page
#[component]
pub fn AuditLogsPage() -> impl IntoView {
    let state = use_app_state();
    let api = use_api_client();

    let (logs, set_logs) = signal(Vec::<AuditLogEntry>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);
    let (event_type_filter, set_event_type_filter) = signal(String::new());
    let (user_filter, set_user_filter) = signal(String::new());
    let (page, set_page) = signal(1u32);
    let (total_pages, set_total_pages) = signal(1u32);

    let load_logs = {
        let api = api.clone();
        move || {
            let api = api.clone();
            set_loading.set(true);
            spawn_local(async move {
                let query = AuditLogQuery {
                    event_type: if event_type_filter.get().is_empty() {
                        None
                    } else {
                        Some(event_type_filter.get())
                    },
                    user_id: if user_filter.get().is_empty() {
                        None
                    } else {
                        Some(user_filter.get())
                    },
                    from_date: None,
                    to_date: None,
                    page: Some(page.get()),
                    per_page: Some(50),
                };

                match api.iam_query_audit_logs(&query).await {
                    Ok(paginated) => {
                        set_logs.set(paginated.data);
                        set_total_pages.set(paginated.total_pages);
                    }
                    Err(e) => set_error.set(Some(format!("Gagal memuat audit log: {}", e))),
                }
                set_loading.set(false);
            });
        }
    };

    {
        let load = load_logs.clone();
        Effect::new(move || {
            let _ = page.get();
            load();
        });
    }

    let handle_filter = {
        let load = load_logs.clone();
        move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            set_page.set(1);
            load();
        }
    };

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
                    " / Audit Log"
                </nav>
                <h1 class="text-2xl font-bold text-gray-900 mb-6">"Audit Log"</h1>

                {move || error.get().map(|msg| view! {
                    <div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg text-red-700">"❌ " {msg}</div>
                })}

                // Filters
                <form on:submit=handle_filter class="mb-6 flex flex-wrap gap-3">
                    <select
                        on:change=move |ev| set_event_type_filter.set(event_target_value(&ev))
                        class="px-3 py-2 border border-gray-300 rounded-lg text-sm"
                    >
                        <option value="">"Semua Event"</option>
                        <option value="login">"Login"</option>
                        <option value="logout">"Logout"</option>
                        <option value="login_failure">"Login Gagal"</option>
                        <option value="token_refresh">"Token Refresh"</option>
                        <option value="password_change">"Ubah Password"</option>
                        <option value="mfa_enable">"MFA Enable"</option>
                        <option value="mfa_disable">"MFA Disable"</option>
                        <option value="user_create">"User Create"</option>
                        <option value="user_update">"User Update"</option>
                        <option value="user_delete">"User Delete"</option>
                        <option value="webauthn_register">"WebAuthn Register"</option>
                    </select>
                    <input
                        type="text"
                        prop:value=user_filter
                        on:input=move |ev| set_user_filter.set(event_target_value(&ev))
                        placeholder="Filter by User ID..."
                        class="px-3 py-2 border border-gray-300 rounded-lg text-sm w-64"
                    />
                    <button type="submit" class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 text-sm">
                        "🔍 Filter"
                    </button>
                </form>

                // Logs table
                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! {
                            <div class="p-8 text-center text-gray-500">
                                <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600 mx-auto mb-3"></div>
                                "Memuat..."
                            </div>
                        }
                    >
                        <div class="overflow-x-auto">
                            <table class="w-full">
                                <thead class="bg-gray-50 border-b">
                                    <tr>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Waktu"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Event"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"User"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"IP"</th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Detail"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-200">
                                    <For
                                        each=move || logs.get()
                                        key=|l| l.id.clone()
                                        children=move |log_entry| {
                                            let event_class = match log_entry.event_type.as_str() {
                                                "login_failure" | "user_delete" => "bg-red-100 text-red-700",
                                                "login" | "user_create" => "bg-green-100 text-green-700",
                                                "logout" => "bg-gray-100 text-gray-700",
                                                _ => "bg-blue-100 text-blue-700",
                                            };
                                            view! {
                                                <tr class="hover:bg-gray-50 text-sm">
                                                    <td class="px-4 py-2.5 text-gray-500 whitespace-nowrap">{log_entry.timestamp.clone()}</td>
                                                    <td class="px-4 py-2.5">
                                                        <span class={format!("text-xs px-2 py-0.5 rounded-full {}", event_class)}>
                                                            {log_entry.event_type.clone()}
                                                        </span>
                                                    </td>
                                                    <td class="px-4 py-2.5 text-gray-700">{log_entry.user_id.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td class="px-4 py-2.5 text-gray-500 font-mono text-xs">{log_entry.ip_address.clone().unwrap_or_else(|| "-".to_string())}</td>
                                                    <td class="px-4 py-2.5 text-gray-500 text-xs max-w-xs truncate">{log_entry.details.clone().unwrap_or_default()}</td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>

                        <Show when=move || logs.get().is_empty()>
                            <div class="text-center py-8 text-gray-500">
                                "Tidak ada log ditemukan."
                            </div>
                        </Show>

                        // Pagination
                        <Show when=move || { total_pages.get() > 1 }>
                            <div class="flex items-center justify-between px-4 py-3 border-t bg-gray-50">
                                <button
                                    on:click=move |_| set_page.set(page.get().saturating_sub(1).max(1))
                                    disabled=move || page.get() <= 1
                                    class="px-3 py-1 text-sm border rounded disabled:opacity-50"
                                >
                                    "← Sebelumnya"
                                </button>
                                <span class="text-sm text-gray-600">
                                    "Halaman " {move || page.get()} " dari " {move || total_pages.get()}
                                </span>
                                <button
                                    on:click=move |_| set_page.set((page.get() + 1).min(total_pages.get()))
                                    disabled=move || page.get() >= total_pages.get()
                                    class="px-3 py-1 text-sm border rounded disabled:opacity-50"
                                >
                                    "Selanjutnya →"
                                </button>
                            </div>
                        </Show>
                    </Show>
                </div>
            </div>
        </MainLayout>
    }
}
