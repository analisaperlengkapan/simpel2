//! Helpdesk — support contact page and ticket tracking.
//!
//! "Kirim Pesan" files a real ticket via `POST /bantuan/tiket` and the page
//! then lists the caller's tickets with their live status. It previously just
//! flipped a local `submitted` signal and told the user "Tim helpdesk akan
//! merespons dalam 1×24 jam kerja" while discarding the message — see #98.
//!
//! The same list doubles as the staff queue: the backend widens
//! `GET /bantuan/tiket` to *all* tickets for helpdesk staff, and staff get a
//! status-transition control per row (`PUT /bantuan/tiket/{id}/status`,
//! server-enforced). Every ticket row can expand into its comment thread.

use crate::api::bantuan::{
    CreateTicketRequest, PRIORITY_OPTIONS, SupportTicket, add_comment, create_ticket, get_ticket,
    list_comments, list_tickets, priority_label, status_label, update_status,
};
use crate::components::layout::{FormField, PageLayout, SectionCard};
use crate::components::role_switcher::use_active_role;
use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use lib_ui::utils::formatters::format_iso_local;
use phosphor_leptos::{
    ARROW_CLOCKWISE, CHAT_CIRCLE, CHECK_CIRCLE, CLOCK, ENVELOPE, PAPER_PLANE_TILT, PHONE,
    WARNING_CIRCLE,
};

/// Tailwind classes per status, so a ticket's state is legible at a glance.
fn status_class(status: &str) -> &'static str {
    match status {
        "open" => "bg-info-500/15 text-info-300",
        "in_progress" => "bg-gold-500/15 text-gold-300",
        "resolved" => "bg-success-500/15 text-success-300",
        "closed" => "bg-slate-500/15 text-slate-300",
        _ => "bg-slate-500/15 text-slate-400",
    }
}

/// Client-side mirror of the backend's `can_transition_to` — offered options
/// only; the server independently rejects anything else (403/422).
fn allowed_transitions(from: &str) -> &'static [&'static str] {
    match from {
        "open" => &["in_progress", "resolved", "closed"],
        "in_progress" => &["resolved", "closed", "open"],
        "resolved" => &["closed", "open"],
        _ => &[],
    }
}

#[component]
pub fn HelpdeskPage() -> impl IntoView {
    let subject = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let priority = RwSignal::new("normal".to_string());
    let submitting = RwSignal::new(false);
    let submit_error = RwSignal::new(None::<String>);
    let last_created = RwSignal::new(None::<SupportTicket>);

    let tickets = RwSignal::new(Vec::<SupportTicket>::new());
    let tickets_loading = RwSignal::new(true);
    let tickets_error = RwSignal::new(None::<String>);

    // Which ticket's comment thread is expanded (one at a time).
    let expanded = RwSignal::new(None::<String>);
    // Error from a staff status transition, shown above the table.
    let action_error = RwSignal::new(None::<String>);
    // Staff controls follow the active role; the server independently derives
    // staff-ness from the JWT, so this only shows/hides UI.
    let active_role = use_active_role();
    let is_staff = move || active_role.get() == "admin";

    let reload = move || {
        spawn_local(async move {
            tickets_loading.set(true);
            tickets_error.set(None);
            match list_tickets(None, 20, 0).await {
                Ok(rows) => tickets.set(rows),
                Err(e) => tickets_error.set(Some(e.to_string())),
            }
            tickets_loading.set(false);
        });
    };

    // Load the caller's tickets on mount.
    Effect::new(move |_| {
        reload();
    });

    let submit = move |_| {
        // Client-side guard mirrors the server's validation so the common
        // mistake costs no round-trip; the server still enforces it.
        if subject.get().trim().is_empty() {
            submit_error.set(Some("Subjek wajib diisi.".to_string()));
            return;
        }
        if message.get().trim().is_empty() {
            submit_error.set(Some("Pesan wajib diisi.".to_string()));
            return;
        }
        spawn_local(async move {
            submitting.set(true);
            submit_error.set(None);
            let req = CreateTicketRequest {
                subject: subject.get_untracked().trim().to_string(),
                description: message.get_untracked().trim().to_string(),
                priority: priority.get_untracked(),
            };
            match create_ticket(&req).await {
                Ok(ticket) => {
                    last_created.set(Some(ticket));
                    subject.set(String::new());
                    message.set(String::new());
                    priority.set("normal".to_string());
                    reload();
                }
                Err(e) => submit_error.set(Some(e.to_string())),
            }
            submitting.set(false);
        });
    };

    view! {
        <PageLayout
            title="Helpdesk"
            icon="fas fa-headset"
            description="Hubungi tim dukungan teknis SIMPEL"
        >
            // Contact cards
            <div class="mb-6 grid grid-cols-1 gap-3 sm:grid-cols-3">
                <SectionCard title="Email" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-info-400 text-lg">
                            <AppIcon icon=ENVELOPE />
                        </span>
                        <span class="text-sm text-slate-300">
                            "helpdesk-simpel@kejaksaan.go.id"
                        </span>
                    </div>
                </SectionCard>
                <SectionCard title="Telepon" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-success-400 text-lg">
                            <AppIcon icon=PHONE />
                        </span>
                        <span class="text-sm text-slate-300">"(021) 123-4567 ext. 890"</span>
                    </div>
                </SectionCard>
                <SectionCard title="Jam Kerja" dense=true>
                    <div class="flex items-start gap-3">
                        <span class="text-gold-400 text-lg">
                            <AppIcon icon=CLOCK />
                        </span>
                        <span class="text-sm text-slate-300">"Senin - Jumat, 08:00 - 16:00"</span>
                    </div>
                </SectionCard>
            </div>

            // Contact form
            <SectionCard title="Kirim Pesan">
                <div class="flex flex-col gap-5">
                    {move || {
                        last_created
                            .get()
                            .map(|t| {
                                view! {
                                    <div
                                        class="flex items-start gap-3 rounded-lg border border-success-500/30 bg-success-500/10 p-4"
                                        data-testid="helpdesk-success"
                                    >
                                        <span class="text-lg text-success-400">
                                            <AppIcon icon=CHECK_CIRCLE />
                                        </span>
                                        <div>
                                            <p class="text-sm font-semibold text-white">
                                                "Tiket Terkirim"
                                            </p>
                                            <p class="mt-1 text-sm text-slate-400">
                                                "Tiket \"" {t.subject.clone()}
                                                "\" telah tercatat dan tampil pada daftar di bawah."
                                            </p>
                                        </div>
                                    </div>
                                }
                            })
                    }}
                    {move || {
                        submit_error
                            .get()
                            .map(|e| {
                                view! {
                                    <div
                                        class="flex items-start gap-3 rounded-lg border border-danger-500/30 bg-danger-500/10 p-4"
                                        data-testid="helpdesk-error"
                                    >
                                        <span class="text-lg text-danger-400">
                                            <AppIcon icon=WARNING_CIRCLE />
                                        </span>
                                        <p class="text-sm text-danger-200">{e}</p>
                                    </div>
                                }
                            })
                    }} <FormField label="Subjek" required=true full_width=true>
                        <input
                            type="text"
                            placeholder="Judul pesan..."
                            data-testid="helpdesk-subject"
                            prop:value=move || subject.get()
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| subject.set(event_target_value(&ev))
                        />
                    </FormField> <FormField label="Prioritas" full_width=true>
                        <select
                            data-testid="helpdesk-priority"
                            prop:value=move || priority.get()
                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                            on:change=move |ev| priority.set(event_target_value(&ev))
                        >
                            {PRIORITY_OPTIONS
                                .iter()
                                .map(|(value, label)| {
                                    view! { <option value=*value>{label.to_string()}</option> }
                                })
                                .collect_view()}
                        </select>
                    </FormField> <FormField label="Pesan" required=true full_width=true>
                        <textarea
                            placeholder="Jelaskan kendala atau pertanyaan Anda..."
                            data-testid="helpdesk-message"
                            prop:value=move || message.get()
                            class="focus-ring min-h-[120px] w-full resize-y rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                            on:input=move |ev| message.set(event_target_value(&ev))
                        ></textarea>
                    </FormField> <div class="flex justify-end border-t border-white/[0.04] pt-4">
                        <button
                            type="button"
                            data-testid="helpdesk-submit"
                            disabled=move || submitting.get()
                            class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
                            on:click=submit
                        >
                            <span class="text-xs">
                                <AppIcon icon=PAPER_PLANE_TILT />
                            </span>
                            {move || {
                                if submitting.get() { "Mengirim..." } else { "Kirim Pesan" }
                            }}
                        </button>
                    </div>
                </div>
            </SectionCard>

            // Server-scoped ticket list: reporters see their own tickets,
            // helpdesk staff see every ticket (widening happens server-side).
            <div class="mt-6">
                <SectionCard
                    title="Tiket Saya"
                    description="Tiket bantuan yang pernah Anda kirim beserta statusnya. Staf helpdesk melihat semua tiket dan dapat mengubah statusnya."
                >
                    <div class="mb-3 flex justify-end">
                        <button
                            type="button"
                            data-testid="helpdesk-refresh"
                            class="inline-flex items-center gap-2 rounded-lg border border-white/10 px-3 py-1.5 text-xs font-semibold text-slate-300 transition hover:bg-white/[0.04]"
                            on:click=move |_| reload()
                        >
                            <AppIcon icon=ARROW_CLOCKWISE />
                            "Muat Ulang"
                        </button>
                    </div>
                    {move || {
                        action_error
                            .get()
                            .map(|e| {
                                view! {
                                    <p
                                        class="mb-2 text-sm text-danger-300"
                                        data-testid="staff-status-error"
                                    >
                                        {e}
                                    </p>
                                }
                            })
                    }}
                    {move || {
                        if let Some(e) = tickets_error.get() {
                            view! {
                                <p
                                    class="py-6 text-center text-sm text-danger-300"
                                    data-testid="helpdesk-list-error"
                                >
                                    {e}
                                </p>
                            }
                                .into_any()
                        } else if tickets_loading.get() {
                            view! {
                                <p class="py-6 text-center text-sm text-slate-400">
                                    "Memuat tiket..."
                                </p>
                            }
                                .into_any()
                        } else if tickets.get().is_empty() {
                            view! {
                                <p
                                    class="py-6 text-center text-sm text-slate-400"
                                    data-testid="helpdesk-empty"
                                >
                                    "Belum ada tiket bantuan."
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="overflow-x-auto">
                                    <table
                                        class="w-full text-left text-sm"
                                        data-testid="helpdesk-tickets"
                                    >
                                        <thead class="text-xs uppercase tracking-wide text-slate-400">
                                            <tr class="border-b border-white/[0.06]">
                                                <th class="px-3 py-2">"Subjek"</th>
                                                <th class="px-3 py-2">"Prioritas"</th>
                                                <th class="px-3 py-2">"Status"</th>
                                                <th class="px-3 py-2">"Satker"</th>
                                                <th class="px-3 py-2">"Dibuat"</th>
                                                <th class="px-3 py-2">"Aksi"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {tickets
                                                .get()
                                                .into_iter()
                                                .map(|t| {
                                                    let status = t.status.clone();
                                                    let status_for_select = t.status.clone();
                                                    let tid_toggle = t.id.clone();
                                                    let tid_status = t.id.clone();
                                                    let satker = t
                                                        .satker_code
                                                        .clone()
                                                        .unwrap_or_else(|| "-".to_string());
                                                    let transitions = allowed_transitions(&status);
                                                    view! {
                                                        <tr
                                                            class="border-b border-white/[0.04]"
                                                            data-testid="helpdesk-ticket-row"
                                                            data-ticket-id=t.id.clone()
                                                            data-status=status.clone()
                                                        >
                                                            <td class="px-3 py-2 text-slate-200">{t.subject}</td>
                                                            <td class="px-3 py-2 text-slate-400">{priority_label(&t.priority)}</td>
                                                            <td class="px-3 py-2">
                                                                <span class=format!(
                                                                    "rounded-full px-2 py-0.5 text-xs font-semibold {}",
                                                                    status_class(&status),
                                                                )>{status_label(&status)}</span>
                                                            </td>
                                                            <td class="px-3 py-2 text-slate-400">{satker}</td>
                                                            <td class="px-3 py-2 text-slate-400">{format_iso_local(&t.created_at)}</td>
                                                            <td class="px-3 py-2">
                                                                <div class="flex items-center gap-2">
                                                                    <button
                                                                        type="button"
                                                                        data-testid="ticket-comments-toggle"
                                                                        data-ticket-id=t.id.clone()
                                                                        class="inline-flex items-center gap-1 rounded-lg border border-white/10 px-2 py-1 text-xs text-slate-300 transition hover:bg-white/[0.04]"
                                                                        on:click=move |_| {
                                                                            expanded
                                                                                .update(|e| {
                                                                                    if e.as_deref() == Some(tid_toggle.as_str()) {
                                                                                        *e = None;
                                                                                    } else {
                                                                                        *e = Some(tid_toggle.clone());
                                                                                    }
                                                                                });
                                                                        }
                                                                    >
                                                                        <AppIcon icon=CHAT_CIRCLE />
                                                                        "Komentar"
                                                                    </button>
                                                                    <Show when=is_staff>
                                                                        {
                                                                            let tid = tid_status.clone();
                                                                            let cur = status_for_select.clone();
                                                                            view! {
                                                                                <select
                                                                                    data-testid="staff-status-select"
                                                                                    data-ticket-id=tid.clone()
                                                                                    class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-2 py-1 text-xs text-slate-200"
                                                                                    prop:value=""
                                                                                    on:change=move |ev| {
                                                                                        let next = event_target_value(&ev);
                                                                                        if next.is_empty() || next == cur {
                                                                                            return;
                                                                                        }
                                                                                        let tid = tid.clone();
                                                                                        spawn_local(async move {
                                                                                            action_error.set(None);
                                                                                            match update_status(&tid, &next).await {
                                                                                                Ok(_) => reload(),
                                                                                                Err(e) => action_error.set(Some(e.to_string())),
                                                                                            }
                                                                                        });
                                                                                    }
                                                                                >
                                                                                    <option value="">"— Ubah Status —"</option>
                                                                                    {transitions
                                                                                        .iter()
                                                                                        .map(|s| {
                                                                                            view! { <option value=*s>{status_label(s)}</option> }
                                                                                        })
                                                                                        .collect_view()}
                                                                                </select>
                                                                            }
                                                                        }
                                                                    </Show>
                                                                </div>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }
                                .into_any()
                        }
                    }}
                    {move || {
                        expanded
                            .get()
                            .map(|id| {
                                view! { <TicketComments ticket_id=id /> }
                            })
                    }}
                </SectionCard>
            </div>
        </PageLayout>
    }
}

/// Comment thread + fresh detail for one ticket. Reporters can only reach
/// this for their own tickets (the backend 404s otherwise); staff for any.
#[component]
fn TicketComments(ticket_id: String) -> impl IntoView {
    use crate::api::bantuan::TicketComment;

    let detail = RwSignal::new(None::<SupportTicket>);
    let comments = RwSignal::new(Vec::<TicketComment>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let new_comment = RwSignal::new(String::new());
    let sending = RwSignal::new(false);

    let tid_load = ticket_id.clone();
    let load = move || {
        let tid = tid_load.clone();
        spawn_local(async move {
            loading.set(true);
            error.set(None);
            match get_ticket(&tid).await {
                Ok(t) => detail.set(Some(t)),
                Err(e) => error.set(Some(e.to_string())),
            }
            match list_comments(&tid).await {
                Ok(c) => comments.set(c),
                Err(e) => error.set(Some(e.to_string())),
            }
            loading.set(false);
        });
    };
    let load_on_mount = load.clone();
    Effect::new(move |_| {
        load_on_mount();
    });

    let tid_submit = ticket_id.clone();
    let submit = move |_| {
        let content = new_comment.get().trim().to_string();
        if content.is_empty() {
            return;
        }
        let tid = tid_submit.clone();
        let load = load.clone();
        spawn_local(async move {
            sending.set(true);
            match add_comment(&tid, &content).await {
                Ok(_) => {
                    new_comment.set(String::new());
                    load();
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            sending.set(false);
        });
    };

    view! {
        <div
            class="mt-4 rounded-lg border border-white/[0.06] bg-white/[0.02] p-4"
            data-testid="ticket-comments"
        >
            {move || {
                error
                    .get()
                    .map(|e| {
                        view! {
                            <p class="mb-2 text-sm text-danger-300" data-testid="comments-error">
                                {e}
                            </p>
                        }
                    })
            }}
            {move || {
                detail
                    .get()
                    .map(|d| {
                        let pelapor: String = d.user_id.chars().take(8).collect();
                        let meta = format!(
                            "Pelapor {} · Satker {} · Diperbarui {}{}",
                            pelapor,
                            d.satker_code.unwrap_or_else(|| "-".to_string()),
                            d.updated_at,
                            d.closed_at.map(|c| format!(" · Ditutup {}", c)).unwrap_or_default(),
                        );
                        view! {
                            <div class="mb-3" data-testid="ticket-detail">
                                <p class="text-sm font-semibold text-white">{d.subject}</p>
                                <p class="mt-1 text-sm text-slate-300">
                                    {d.description.unwrap_or_default()}
                                </p>
                                <p class="mt-1 text-xs text-slate-500">{meta}</p>
                            </div>
                        }
                    })
            }}
            {move || {
                if loading.get() {
                    view! { <p class="text-sm text-slate-400">"Memuat komentar..."</p> }.into_any()
                } else if comments.get().is_empty() {
                    view! {
                        <p class="text-sm text-slate-400" data-testid="comments-empty">
                            "Belum ada komentar."
                        </p>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="flex flex-col gap-2">
                            {comments
                                .get()
                                .into_iter()
                                .map(|c| {
                                    let who: String = c.user_id.chars().take(8).collect();
                                    view! {
                                        <div
                                            class="rounded-lg bg-white/[0.03] p-3"
                                            data-testid="comment-row"
                                            data-comment-id=c.id.clone()
                                        >
                                            <p class="text-xs text-slate-500">
                                                {who} " · " {format_iso_local(&c.created_at)}
                                            </p>
                                            <p class="mt-1 text-sm text-slate-200">{c.content}</p>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}
            <div class="mt-3 flex gap-2">
                <input
                    type="text"
                    placeholder="Tulis komentar..."
                    data-testid="comment-input"
                    prop:value=move || new_comment.get()
                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                    on:input=move |ev| new_comment.set(event_target_value(&ev))
                />
                <button
                    type="button"
                    data-testid="comment-submit"
                    disabled=move || sending.get()
                    class="inline-flex items-center gap-2 rounded-lg border border-white/10 px-3 py-2 text-sm font-semibold text-slate-200 transition hover:bg-white/[0.04] disabled:cursor-not-allowed disabled:opacity-50"
                    on:click=submit
                >
                    {move || if sending.get() { "Mengirim..." } else { "Kirim" }}
                </button>
            </div>
        </div>
    }
}
