//! Workflow Delegation admin page.
//!
//! Lists delegations the caller has either created or received, lets the
//! caller create a new delegation, and revoke an active/scheduled one
//! they delegated themselves. Backed by `/workflow/delegations` REST
//! endpoints.

use leptos::prelude::*;
use leptos::task::spawn_local;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_CLOCKWISE, PLUS, X};

use crate::api::error::AppError;
use crate::api::workflow::{
    CreateDelegationBody, Delegation, DelegationStatus, create_delegation, fetch_delegations,
    revoke_delegation,
};
use crate::components::layout::{EmptyState, ErrorState, LoadingState, PageLayout, SectionCard};

/// Truncate the first chunk of a UUID for display so the table stays
/// scannable; full IDs go in a `title=` tooltip via the cell title attr.
fn short_id(uuid: &str) -> String {
    uuid.split('-').next().unwrap_or(uuid).to_string()
}

/// `"2026-05-25T08:00:00Z"` → `"25 Mei 2026 08:00"`. Failures fall back to
/// the original string so we never hide data behind formatter errors.
fn format_datetime(iso: &str) -> String {
    use chrono::DateTime;
    DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%d %b %Y %H:%M").to_string())
        .unwrap_or_else(|_| iso.to_string())
}

fn status_badge_classes(status: &DelegationStatus) -> &'static str {
    match status {
        DelegationStatus::Active => "border-success-500/30 bg-success-500/10 text-success-300",
        DelegationStatus::Scheduled => "border-info-500/30 bg-info-500/10 text-info-300",
        DelegationStatus::Expired => "border-white/[0.06] bg-white/[0.02] text-slate-400",
        DelegationStatus::Revoked => "border-danger-500/30 bg-danger-500/10 text-danger-300",
    }
}

#[component]
pub fn WorkflowDelegationPage() -> impl IntoView {
    // false → "delegasi yang saya buat"; true → "delegasi untuk saya"
    let (as_delegate, set_as_delegate) = signal(false);
    let (rows, set_rows) = signal::<Vec<Delegation>>(Vec::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (show_create, set_show_create) = signal(false);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let tick = reload_tick.get();
        let is_delegate = as_delegate.get();
        let _ = tick;
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match fetch_delegations(is_delegate).await {
                Ok(resp) => set_rows.set(resp.data),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let refresh = move |_| set_reload_tick.update(|t| *t += 1);
    let open_create = move |_| set_show_create.set(true);
    let close_create = Callback::new(move |_| set_show_create.set(false));
    let on_created = Callback::new(move |_| {
        set_show_create.set(false);
        set_reload_tick.update(|t| *t += 1);
    });

    view! {
        <PageLayout
            title="Delegasi Workflow"
            description="Lihat dan kelola pendelegasian role workflow ke pengguna lain."
            icon="fas fa-user-shield"
            actions=Box::new(move || view! {
                <button
                    type="button"
                    class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105"
                    on:click=open_create
                >
                    <span class="mr-2"><AppIcon icon=PLUS /></span>
                    "Buat Delegasi"
                </button>
                <button
                    type="button"
                    class="focus-ring inline-flex items-center gap-2 rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-1.5 text-xs font-semibold text-info-300 transition hover:bg-info-500/20"
                    on:click=refresh
                >
                    <span class="text-[0.7rem]"><AppIcon icon=ARROW_CLOCKWISE /></span>
                    "Refresh"
                </button>
            }.into_any())
        >
            // Tab switcher between "saya delegasikan" vs "untuk saya"
            <div class="mb-4 inline-flex gap-1 rounded-lg border border-white/[0.06] bg-white/[0.02] p-1">
                <button
                    type="button"
                    class={move || tab_classes(!as_delegate.get())}
                    on:click=move |_| set_as_delegate.set(false)
                >
                    "Saya Delegasikan"
                </button>
                <button
                    type="button"
                    class={move || tab_classes(as_delegate.get())}
                    on:click=move |_| set_as_delegate.set(true)
                >
                    "Untuk Saya"
                </button>
            </div>

            <SectionCard title="Daftar Delegasi" icon="fas fa-list-check">
                {move || {
                    if loading.get() && rows.with(|r| r.is_empty()) {
                        view! { <LoadingState message="Memuat delegasi..." /> }.into_any()
                    } else if let Some(err) = error.get() {
                        let retry: Box<dyn Fn()> = Box::new(move || set_reload_tick.update(|t| *t += 1));
                        view! { <ErrorState error=err on_retry=retry /> }.into_any()
                    } else {
                        let list = rows.get();
                        if list.is_empty() {
                            view! {
                                <EmptyState
                                    title="Belum ada delegasi"
                                    description="Belum ada pendelegasian yang tercatat pada tab ini.".to_string()
                                />
                            }.into_any()
                        } else {
                            let mine = !as_delegate.get();
                            view! { <DelegationTable rows=list mine_can_revoke=mine on_revoked=Callback::new(move |_| set_reload_tick.update(|t| *t += 1)) /> }.into_any()
                        }
                    }
                }}
            </SectionCard>

            {move || show_create.get().then(|| view! {
                <CreateDelegationModal on_close=close_create on_save=on_created />
            })}
        </PageLayout>
    }
}

fn tab_classes(active: bool) -> &'static str {
    if active {
        "focus-ring rounded-md bg-gold-gradient px-3 py-1.5 text-xs font-semibold text-navy-950"
    } else {
        "focus-ring rounded-md px-3 py-1.5 text-xs font-semibold text-slate-300 transition hover:bg-white/[0.04]"
    }
}

#[component]
fn DelegationTable(
    rows: Vec<Delegation>,
    mine_can_revoke: bool,
    #[prop(into)] on_revoked: Callback<()>,
) -> impl IntoView {
    let body_rows = rows
        .into_iter()
        .map(|d| {
            let can_revoke = mine_can_revoke
                && matches!(
                    d.status,
                    DelegationStatus::Active | DelegationStatus::Scheduled
                );
            let id_for_revoke = d.id.clone();
            let id_full = d.id.clone();
            let delegate_full = d.delegate_user_id.clone();
            let delegator_full = d.delegator_user_id.clone();
            let role = d.role.clone();
            let status_text = d.status.as_label().to_string();
            let status_classes = status_badge_classes(&d.status);
            let valid_from = format_datetime(&d.valid_from);
            let valid_until = format_datetime(&d.valid_until);
            let reason = d.reason.clone().unwrap_or_else(|| "—".to_string());

            let on_revoked = on_revoked;
            let revoke_action = move |_| {
                let id = id_for_revoke.clone();
                spawn_local(async move {
                    if let Err(e) = revoke_delegation(&id).await {
                        web_sys::console::error_1(
                            &format!("revoke delegation failed: {}", e).into(),
                        );
                    }
                    on_revoked.run(());
                });
            };

            view! {
                <tr class="border-b border-white/[0.04]">
                    <td class="px-3 py-3 font-mono text-xs text-slate-300" title=id_full>{short_id(&d.id)}</td>
                    <td class="px-3 py-3 font-mono text-xs text-slate-300" title=delegator_full.clone()>{short_id(&delegator_full)}</td>
                    <td class="px-3 py-3 font-mono text-xs text-slate-300" title=delegate_full.clone()>{short_id(&delegate_full)}</td>
                    <td class="px-3 py-3 text-xs text-white">{role}</td>
                    <td class="px-3 py-3 text-xs text-slate-300">{valid_from}</td>
                    <td class="px-3 py-3 text-xs text-slate-300">{valid_until}</td>
                    <td class="px-3 py-3 text-xs text-slate-400">{reason}</td>
                    <td class="px-3 py-3">
                        <span class=format!("inline-flex items-center rounded-md border px-2 py-0.5 text-[0.65rem] font-semibold {}", status_classes)>
                            {status_text}
                        </span>
                    </td>
                    <td class="px-3 py-3 text-right">
                        {if can_revoke {
                            view! {
                                <button
                                    type="button"
                                    class="focus-ring rounded-md border border-danger-500/30 bg-danger-500/10 px-2.5 py-1 text-[0.7rem] font-semibold text-danger-300 transition hover:bg-danger-500/20"
                                    on:click=revoke_action
                                >
                                    "Cabut"
                                </button>
                            }.into_any()
                        } else {
                            view! { <span class="text-[0.65rem] text-slate-500">"—"</span> }.into_any()
                        }}
                    </td>
                </tr>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="overflow-x-auto rounded-xl border border-white/[0.06]">
            <table class="min-w-full border-collapse">
                <thead>
                    <tr class="bg-white/[0.02]">
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"ID"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Delegator"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Delegate"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Role"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Mulai"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Berakhir"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Alasan"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-left text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Status"</th>
                        <th class="border-b border-white/[0.06] px-3 py-2 text-right text-[0.65rem] font-semibold uppercase tracking-wider text-slate-400">"Aksi"</th>
                    </tr>
                </thead>
                <tbody>{body_rows}</tbody>
            </table>
        </div>
    }
}

/// `datetime-local` form input → RFC3339 string. Browser hands us
/// `"2026-05-25T08:00"` (no timezone); we append the local TZ offset so the
/// backend interprets it consistently with what the user saw on screen.
fn local_input_to_rfc3339(local: &str) -> Option<String> {
    use chrono::Local;
    if local.is_empty() {
        return None;
    }
    let parsed = chrono::NaiveDateTime::parse_from_str(local, "%Y-%m-%dT%H:%M").ok()?;
    let with_tz = parsed.and_local_timezone(Local).single()?;
    Some(with_tz.to_rfc3339())
}

#[component]
fn CreateDelegationModal(
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let (delegate_id, set_delegate_id) = signal(String::new());
    let (role, set_role) = signal(String::new());
    let (valid_from, set_valid_from) = signal(String::new());
    let (valid_until, set_valid_until) = signal(String::new());
    let (reason, set_reason) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal::<Option<String>>(None);

    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        // Validate cheap inputs synchronously — uuid format + required end date
        let delegate_value = delegate_id.get();
        let role_value = role.get();
        let until_value = valid_until.get();
        if delegate_value.trim().is_empty()
            || role_value.trim().is_empty()
            || until_value.trim().is_empty()
        {
            set_submit_error.set(Some(
                "Delegate, role, dan tanggal berakhir wajib diisi.".to_string(),
            ));
            return;
        }

        let valid_until_iso = match local_input_to_rfc3339(&until_value) {
            Some(s) => s,
            None => {
                set_submit_error.set(Some("Format tanggal berakhir tidak valid.".to_string()));
                return;
            }
        };
        let valid_from_iso = local_input_to_rfc3339(&valid_from.get());
        let reason_value = reason.get();
        let reason_opt = if reason_value.trim().is_empty() {
            None
        } else {
            Some(reason_value)
        };

        let body = CreateDelegationBody {
            delegate_user_id: delegate_value.trim().to_string(),
            role: role_value.trim().to_string(),
            valid_from: valid_from_iso,
            valid_until: valid_until_iso,
            reason: reason_opt,
        };
        set_submitting.set(true);
        set_submit_error.set(None);
        let on_save = on_save;
        spawn_local(async move {
            match create_delegation(body).await {
                Ok(_) => {
                    set_submitting.set(false);
                    on_save.run(());
                }
                Err(e) => {
                    set_submitting.set(false);
                    set_submit_error.set(Some(format!("Gagal membuat delegasi: {}", e)));
                }
            }
        });
    };

    view! {
        <div
            class="fixed inset-0 z-modal flex items-start justify-center overflow-y-auto bg-black/60 p-4 backdrop-blur-sm sm:p-8"
            on:click=move |e| {
                if e.target() == e.current_target() {
                    on_close.run(());
                }
            }
        >
            <form
                class="my-auto flex w-full max-w-lg flex-col gap-4 rounded-2xl border border-white/[0.08] bg-surface-panel p-6 shadow-panel"
                on:submit=submit
            >
                <header class="flex items-start justify-between gap-3">
                    <div>
                        <div class="text-xs font-semibold uppercase tracking-wider text-gold-300">
                            "Delegasi Baru"
                        </div>
                        <h2 class="mt-1 text-xl font-bold text-white">"Buat Delegasi"</h2>
                    </div>
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.04] p-2 text-slate-300 transition hover:bg-white/[0.08]"
                        on:click=move |_| on_close.run(())
                    >
                        <AppIcon icon=X />
                    </button>
                </header>

                <label class="flex flex-col gap-1">
                    <span class="text-xs font-semibold text-slate-300">"Delegate User ID (UUID)"</span>
                    <input
                        class="rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white focus:border-gold-400 focus:outline-none"
                        placeholder="00000000-0000-0000-0000-000000000000"
                        prop:value=move || delegate_id.get()
                        on:input=move |e| set_delegate_id.set(event_target_value(&e))
                    />
                </label>

                <label class="flex flex-col gap-1">
                    <span class="text-xs font-semibold text-slate-300">"Role"</span>
                    <input
                        class="rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white focus:border-gold-400 focus:outline-none"
                        placeholder="validator_wilayah"
                        prop:value=move || role.get()
                        on:input=move |e| set_role.set(event_target_value(&e))
                    />
                </label>

                <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    <label class="flex flex-col gap-1">
                        <span class="text-xs font-semibold text-slate-300">"Mulai (opsional)"</span>
                        <input
                            type="datetime-local"
                            class="rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white focus:border-gold-400 focus:outline-none"
                            prop:value=move || valid_from.get()
                            on:input=move |e| set_valid_from.set(event_target_value(&e))
                        />
                    </label>
                    <label class="flex flex-col gap-1">
                        <span class="text-xs font-semibold text-slate-300">"Berakhir"</span>
                        <input
                            type="datetime-local"
                            class="rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white focus:border-gold-400 focus:outline-none"
                            prop:value=move || valid_until.get()
                            on:input=move |e| set_valid_until.set(event_target_value(&e))
                        />
                    </label>
                </div>

                <label class="flex flex-col gap-1">
                    <span class="text-xs font-semibold text-slate-300">"Alasan (opsional)"</span>
                    <textarea
                        class="rounded-lg border border-white/[0.08] bg-white/[0.04] px-3 py-2 text-sm text-white focus:border-gold-400 focus:outline-none"
                        rows="3"
                        prop:value=move || reason.get()
                        on:input=move |e| set_reason.set(event_target_value(&e))
                    />
                </label>

                {move || submit_error.get().map(|msg| view! {
                    <div class="rounded-md border border-danger-500/30 bg-danger-500/10 px-3 py-2 text-xs text-danger-300">
                        {msg}
                    </div>
                })}

                <div class="mt-2 flex justify-end gap-2">
                    <button
                        type="button"
                        class="focus-ring rounded-lg border border-white/[0.08] bg-white/[0.02] px-4 py-2 text-sm font-semibold text-slate-300 transition hover:bg-white/[0.06]"
                        on:click=move |_| on_close.run(())
                    >
                        "Batal"
                    </button>
                    <button
                        type="submit"
                        prop:disabled=move || submitting.get()
                        class="focus-ring rounded-lg bg-gold-gradient px-4 py-2 text-sm font-semibold text-navy-950 shadow-card transition hover:brightness-105 disabled:cursor-not-allowed disabled:opacity-60"
                    >
                        {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                    </button>
                </div>
            </form>
        </div>
    }
}
