//! Audit Logs Page (Admin)
//!
//! View, search, and filter security and activity audit logs.
//! REQ-PORTAL-015

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::authenc_api::{AuditLogEntry, AuditLogQuery};
use leptos::prelude::*;
use leptos::task::spawn_local;

/// Human-readable label for an audit `event_type`.
///
/// The backend emits SCREAMING_SNAKE enum tags (`AUTH_SUCCESS`, `TOKEN_REVOKE`,
/// …). Printing the tag straight into the table meant operators read raw wire
/// values — a defect the screenshot sweep flagged as `enum-mentah`. Unknown
/// values fall through unchanged so a newly added server event is still legible
/// rather than silently blanked.
fn event_label(event_type: &str) -> String {
    let label = match event_type {
        "AUTH_SUCCESS" | "LOGIN_SUCCESS" | "login" => "Login berhasil",
        "AUTH_FAILURE" | "LOGIN_FAILURE" | "login_failure" => "Login gagal",
        "AUTH_ATTEMPT" => "Percobaan login",
        "LOGOUT" | "logout" => "Logout",
        "TOKEN_REFRESH" => "Token diperbarui",
        "TOKEN_REVOKE" => "Token dicabut",
        "PASSWORD_CHANGE" => "Ganti kata sandi",
        "PASSWORD_RESET" => "Reset kata sandi",
        "MFA_ENROLL" => "MFA diaktifkan",
        "MFA_VERIFY" => "Verifikasi MFA",
        "USER_CREATE" | "user_create" => "Pengguna dibuat",
        "USER_UPDATE" => "Pengguna diubah",
        "USER_DELETE" | "user_delete" => "Pengguna dihapus",
        "ROLE_ASSIGN" => "Peran ditetapkan",
        "ROLE_REVOKE" => "Peran dicabut",
        "CLIENT_CREATE" => "Klien dibuat",
        "CLIENT_UPDATE" => "Klien diubah",
        "CLIENT_DELETE" => "Klien dihapus",
        "SESSION_TERMINATE" => "Sesi diakhiri",
        other => return other.to_string(),
    };
    label.to_string()
}

/// Render the audit `details` payload for a human.
///
/// The column holds a JSON object (`{"duration_ms":133,"ip":"…","method":"POST"}`)
/// rendered verbatim, which filled the table with punctuation and pushed the
/// meaningful fields off-screen. Flattened to `key: value` pairs separated by a
/// bullet, with the noise keys (byte-level request internals an operator does
/// not act on) dropped. Unparseable payloads are returned unchanged — the audit
/// trail is evidence, so nothing may be silently discarded.
fn event_details(details: Option<&str>) -> String {
    let Some(raw) = details.map(str::trim).filter(|s| !s.is_empty()) else {
        return "—".to_string();
    };
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw) else {
        return raw.to_string();
    };
    // Keys already shown in a dedicated column, or addressable only to a
    // machine. `user_id` is the UUID the USER cell resolves to a username, so
    // repeating it here is the same identifier twice in one row; `ip` likewise
    // has its own column. `suspicious_indicators` is retained for the risk
    // engine's own consumers, not for this table.
    const NOISE: &[&str] = &[
        "suspicious_indicators",
        "user_agent",
        "request_id",
        "user_id",
        "ip",
    ];
    let parts: Vec<String> = map
        .iter()
        .filter(|(k, v)| !NOISE.contains(&k.as_str()) && !v.is_null())
        .map(|(k, v)| {
            let value = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            format!("{k}: {value}")
        })
        .collect();
    if parts.is_empty() {
        return "—".to_string();
    }
    parts.join(" · ")
}

/// Name the actor behind an audit row.
///
/// The table rendered `user_id` — a UUID — which is unique but tells an
/// operator nothing; the API already joins `users` and ships `username`
/// alongside it. The id is kept only as a fallback for events whose user row
/// has since been deleted, where a UUID still beats an empty cell.
fn audit_actor(username: Option<&str>, user_id: Option<&str>) -> String {
    username
        .filter(|s| !s.trim().is_empty())
        .or(user_id)
        .unwrap_or("-")
        .to_string()
}

/// Audit logs page
#[component]
pub fn AuditLogsPage() -> impl IntoView {
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

            // Only explicitly load if we are already on page 1.
            // If page > 1, setting page to 1 will trigger the Effect above.
            if page.get() == 1 {
                load();
            } else {
                set_page.set(1);
            }
        }
    };

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / Audit Log"
                </nav>
                <h1 class="text-2xl font-bold text-gray-900 mb-6">"Audit Log"</h1>

                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

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
                    <button
                        type="submit"
                        class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 text-sm"
                    >
                        "🔍 Filter"
                    </button>
                </form>

                // Logs table
                <div class="bg-white rounded-xl border border-gray-200 overflow-hidden">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <LoadingPanel message="Memuat audit log..." /> }
                    >
                        <div class="overflow-x-auto">
                            <table class="w-full">
                                <thead class="bg-gray-50 border-b">
                                    <tr>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                            "Waktu"
                                        </th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                            "Event"
                                        </th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                            "User"
                                        </th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                            "IP"
                                        </th>
                                        <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                                            "Detail"
                                        </th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-gray-200">
                                    <For
                                        each=move || logs.get()
                                        key=|l| l.id.clone()
                                        children=move |log_entry| {
                                            let event_class = match log_entry.event_type.as_str() {
                                                "AUTH_FAILURE" | "LOGIN_FAILURE" | "login_failure"
                                                | "USER_DELETE" | "user_delete" => {
                                                    "bg-red-100 text-red-700"
                                                }
                                                "AUTH_SUCCESS" | "LOGIN_SUCCESS" | "login"
                                                | "USER_CREATE" | "user_create" => {
                                                    "bg-green-100 text-green-700"
                                                }
                                                "LOGOUT" | "logout" => "bg-gray-100 text-gray-700",
                                                _ => "bg-blue-100 text-blue-700",
                                            };
                                            view! {
                                                <tr class="hover:bg-gray-50 text-sm">
                                                    <td class="px-4 py-2.5 text-gray-500 whitespace-nowrap">
                                                        {lib_ui::utils::format_iso_local(
                                                            &log_entry.timestamp,
                                                        )}
                                                    </td>
                                                    <td class="px-4 py-2.5">
                                                        <span class=format!(
                                                            "text-xs px-2 py-0.5 rounded-full {}",
                                                            event_class,
                                                        )>{event_label(&log_entry.event_type)}</span>
                                                    </td>
                                                    <td class="px-4 py-2.5 text-gray-700">
                                                        {audit_actor(
                                                            log_entry.username.as_deref(),
                                                            log_entry.user_id.as_deref(),
                                                        )}
                                                    </td>
                                                    <td class="px-4 py-2.5 text-gray-500 font-mono text-xs">
                                                        {log_entry
                                                            .ip_address
                                                            .clone()
                                                            .unwrap_or_else(|| "-".to_string())}
                                                    </td>
                                                    <td class="px-4 py-2.5 text-gray-500 text-xs">
                                                        {event_details(
                                                            log_entry.details.as_deref(),
                                                        )}
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>

                        <Show when=move || logs.get().is_empty()>
                            <div class="m-4">
                                <EmptyPanel
                                    title="Tidak ada audit log"
                                    message="Tidak ditemukan data log untuk filter yang dipilih."
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
                                    disabled=move || { page.get() >= total_pages.get() }
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
