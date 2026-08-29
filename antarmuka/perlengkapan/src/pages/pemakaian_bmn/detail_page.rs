//! Pemakaian BMN detail — permit lifecycle surface.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_LEFT, PROHIBIT, REPEAT, WARNING, X};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlTextAreaElement};

use crate::api::error::AppError;
use crate::api::pemakaian_bmn::{
    self, IzinPemakaianDetailResponse, PemakaianWorkflowTransitionInfo, RenewPermitRequest,
    RevokePermitRequest,
};
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::components::status_badge::status_tone_classes;
use crate::routes::path;

#[component]
pub fn PemakaianBmnDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id_signal = Signal::derive(move || params.with(|p| p.get("id").unwrap_or_default()));

    let (detail, set_detail) = signal::<Option<IzinPemakaianDetailResponse>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);
    let (action_msg, set_action_msg) = signal::<Option<(bool, String)>>(None);
    let (show_revoke, set_show_revoke) = signal(false);
    let (show_renew, set_show_renew) = signal(false);
    let (revoke_reason, set_revoke_reason) = signal::<String>(String::new());
    let (renew_start, set_renew_start) = signal::<String>(String::new());
    let (renew_end, set_renew_end) = signal::<String>(String::new());
    let (renew_keperluan, set_renew_keperluan) = signal::<String>(String::new());
    let (submitting, set_submitting) = signal(false);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let id = id_signal.get();
        if id.is_empty() {
            set_loading.set(false);
            set_error.set(Some(AppError::not_found("Id izin tidak diberikan.")));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match pemakaian_bmn::fetch_pemakaian_bmn_detail(&id).await {
                Ok(resp) => set_detail.set(Some(resp.data)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let reload = move || set_reload_tick.update(|t| *t += 1);

    let open_revoke = move |_| {
        set_revoke_reason.set(String::new());
        set_show_revoke.set(true);
    };
    let close_revoke = move |_| set_show_revoke.set(false);

    let submit_revoke = move |_| {
        let id = id_signal.get();
        let reason = revoke_reason.get();
        if reason.trim().is_empty() {
            set_action_msg.set(Some((false, "Alasan pencabutan wajib diisi.".to_string())));
            return;
        }
        set_submitting.set(true);
        spawn_local(async move {
            match pemakaian_bmn::revoke_pemakaian_bmn(&id, RevokePermitRequest { alasan: reason })
                .await
            {
                Ok(_) => {
                    set_action_msg.set(Some((true, "Izin berhasil dicabut.".to_string())));
                    set_show_revoke.set(false);
                    set_reload_tick.update(|t| *t += 1);
                }
                Err(e) => {
                    set_action_msg.set(Some((false, e.user_message())));
                }
            }
            set_submitting.set(false);
        });
    };

    let open_renew = move |_| {
        set_renew_start.set(String::new());
        set_renew_end.set(String::new());
        set_renew_keperluan.set(String::new());
        set_show_renew.set(true);
    };
    let close_renew = move |_| set_show_renew.set(false);

    let submit_renew = move |_| {
        let id = id_signal.get();
        let start = renew_start.get();
        let end = renew_end.get();
        let keperluan = renew_keperluan.get();
        if start.is_empty() || end.is_empty() || keperluan.trim().is_empty() {
            set_action_msg.set(Some((
                false,
                "Tanggal mulai, selesai, dan keperluan wajib diisi.".to_string(),
            )));
            return;
        }
        set_submitting.set(true);
        spawn_local(async move {
            match pemakaian_bmn::renew_pemakaian_bmn(
                &id,
                RenewPermitRequest {
                    tanggal_mulai: start,
                    tanggal_selesai: end,
                    keperluan,
                },
            )
            .await
            {
                Ok(_) => {
                    set_action_msg.set(Some((true, "Izin berhasil diperpanjang.".to_string())));
                    set_show_renew.set(false);
                    set_reload_tick.update(|t| *t += 1);
                }
                Err(e) => {
                    set_action_msg.set(Some((false, e.user_message())));
                }
            }
            set_submitting.set(false);
        });
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Pemakaian BMN", path::PENGELOLAAN_PEMAKAIAN),
        PageBreadcrumb::leaf("Detail"),
    ];

    view! {
        <PageLayout
            title="Detail Izin Pemakaian"
            description="Kelola aksi lifecycle: perpanjang, cabut, dan pantau sisa waktu."
            icon="fas fa-handshake"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || {
                view! {
                    <A
                        href=path::PENGELOLAAN_PEMAKAIAN
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=ARROW_LEFT />
                        </span>
                        "Kembali"
                    </A>
                }
                    .into_any()
            })
        >
            {move || {
                if let Some((ok, msg)) = action_msg.get() {
                    let tone = if ok {
                        "border-success-500/30 bg-success-500/10 text-success-200"
                    } else {
                        "border-danger-500/30 bg-danger-500/10 text-danger-200"
                    };
                    view! {
                        <div class=format!(
                            "rounded-lg border px-4 py-2 text-xs {}",
                            tone,
                        )>{msg}</div>
                    }
                        .into_any()
                } else {
                    view! { <div class="hidden"></div> }.into_any()
                }
            }}

            {move || {
                if loading.get() && detail.get().is_none() {
                    view! { <LoadingState message="Memuat detail izin..." /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! { <ErrorState error=err on_retry=Box::new(move || reload()) /> }
                        .into_any()
                } else if let Some(d) = detail.get() {
                    view! {
                        <DetailBody
                            detail=d
                            set_action_msg=set_action_msg
                            set_reload_tick=set_reload_tick
                            on_renew=open_renew
                            on_revoke=open_revoke
                        />
                    }
                        .into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Izin tidak ditemukan"
                            description="Permohonan izin tidak tersedia dalam basis data."
                            icon="fas fa-folder-open"
                        />
                    }
                        .into_any()
                }
            }}

            <Show when=move || show_revoke.get()>
                <RevokeModal
                    reason=revoke_reason
                    set_reason=set_revoke_reason
                    submitting=submitting
                    on_close=close_revoke
                    on_submit=submit_revoke
                />
            </Show>

            <Show when=move || show_renew.get()>
                <RenewModal
                    start=renew_start
                    end=renew_end
                    keperluan=renew_keperluan
                    set_start=set_renew_start
                    set_end=set_renew_end
                    set_keperluan=set_renew_keperluan
                    submitting=submitting
                    on_close=close_renew
                    on_submit=submit_renew
                />
            </Show>
        </PageLayout>
    }
}

#[component]
fn DetailBody(
    detail: IzinPemakaianDetailResponse,
    set_action_msg: WriteSignal<Option<(bool, String)>>,
    set_reload_tick: WriteSignal<u32>,
    on_renew: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_revoke: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let izin = detail.izin.clone();
    let transitions = detail.allowed_transitions.clone();
    let permit_id = izin.id.clone();
    let permit_version = izin.version;
    let status = izin.status.clone();
    let is_active = status == "ACTIVE" || status == "APPROVED";
    let can_revoke = is_active;
    let can_renew = is_active;

    let nomor = izin.nomor_izin.clone().unwrap_or_else(|| "-".to_string());
    let periode = format!("{} → {}", izin.tanggal_mulai, izin.tanggal_selesai);
    let pemohon_nama = izin.pegawai_nama.clone();
    let pemohon_nip = izin.pegawai_nip.clone();
    let pemohon_satker = izin.pegawai_satker_nama.clone();
    let bmn_nama = izin.bmn_nama_barang.clone();
    let bmn_nup = izin.bmn_nup.clone();
    let bmn_kode = izin.bmn_kode_barang.clone();
    let keperluan = izin.keperluan.clone();
    let lokasi = izin
        .lokasi_pemakaian
        .clone()
        .unwrap_or_else(|| "-".to_string());

    let days = detail.days_until_expiry;
    let expiring_soon = detail.is_expiring_soon;

    view! {
        <LifecycleSummary
            status_label=izin.status_label.clone()
            status_tone=izin.status_tone.clone()
            days=days
            expiring_soon=expiring_soon
            periode=periode.clone()
            can_renew=can_renew
            can_revoke=can_revoke
            on_renew=on_renew
            on_revoke=on_revoke
        />

        <SatkerApprovalActions
            id=permit_id
            from_state=status.clone()
            version=permit_version
            transitions=transitions
            set_action_msg=set_action_msg
            set_reload_tick=set_reload_tick
        />

        <SectionCard
            title=format!("Izin #{}", nomor)
            description="Informasi umum permohonan."
            icon="fas fa-circle-info"
        >
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <InfoField label="Pemohon" value=pemohon_nama />
                <InfoField label="NIP" value=pemohon_nip />
                <InfoField label="Satker" value=pemohon_satker />
                <InfoField label="BMN" value=bmn_nama />
                <InfoField label="NUP" value=bmn_nup />
                <InfoField label="Kode Barang" value=bmn_kode />
                <InfoField label="Periode" value=periode.clone() />
                <InfoField label="Lokasi Pemakaian" value=lokasi />
            </div>
            <div class="mt-4 rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Keperluan"</p>
                <p class="mt-1 text-sm text-slate-200">{keperluan}</p>
            </div>
        </SectionCard>
    }
}

/// The satker-internal approval chain, rendered from what the backend says
/// this caller may do next.
///
/// `allowed_transitions` arrives already narrowed twice — by the workflow
/// config, then by `PemakaianBmnPolicy` against the caller's role — and each
/// entry carries its own imperative label. So there is deliberately no role
/// check here: duplicating the policy in the browser is how a UI ends up
/// offering a button the API answers 403 to.
///
/// Revoke and renew keep their own controls on [`LifecycleSummary`]; they are
/// lifecycle operations on an already-active permit, not approval steps.
#[component]
fn SatkerApprovalActions(
    id: String,
    from_state: String,
    version: i32,
    transitions: Vec<PemakaianWorkflowTransitionInfo>,
    set_action_msg: WriteSignal<Option<(bool, String)>>,
    set_reload_tick: WriteSignal<u32>,
) -> impl IntoView {
    let actionable: Vec<PemakaianWorkflowTransitionInfo> = transitions
        .into_iter()
        .filter(|t| t.status != "REVOKED")
        .collect();

    if actionable.is_empty() {
        return ().into_any();
    }

    let (catatan, set_catatan) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let needs_comment = actionable.iter().any(|t| t.requires_comment);

    view! {
        <SectionCard
            title="Tindakan Persetujuan"
            description="Alur persetujuan internal satuan kerja."
            icon="fas fa-user-check"
        >
            <Show when=move || needs_comment>
                <div class="mb-3">
                    <label
                        for="catatan-satker"
                        class="text-[0.65rem] uppercase tracking-wide text-slate-500"
                    >
                        "Catatan Revisi"
                    </label>
                    <textarea
                        id="catatan-satker"
                        rows="2"
                        placeholder="Minimal 10 karakter — jelaskan apa yang perlu diperbaiki Operator."
                        prop:value=move || catatan.get()
                        on:input=move |ev| {
                            let el = ev.target().unwrap().unchecked_into::<HtmlTextAreaElement>();
                            set_catatan.set(el.value());
                        }
                        class="focus-ring mt-1 w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder:text-slate-500"
                    />
                </div>
            </Show>

            <div class="flex flex-wrap gap-2">
                <For
                    each=move || actionable.clone()
                    key=|t| t.status.clone()
                    let:transition
                >
                    {
                        let id = id.clone();
                        let from_state = from_state.clone();
                        let to_state = transition.status.clone();
                        let requires_comment = transition.requires_comment;
                        let on_click = move |_| {
                            let catatan_now = catatan.get();
                            // The backend enforces a 10-char minimum; checking here
                            // turns a 400 round-trip into immediate feedback. The
                            // server stays the authority either way.
                            if requires_comment && catatan_now.trim().len() < 10 {
                                set_action_msg
                                    .set(
                                        Some((
                                            false,
                                            "Catatan revisi wajib diisi, minimal 10 karakter."
                                                .to_string(),
                                        )),
                                    );
                                return;
                            }
                            let catatan_opt = if catatan_now.trim().is_empty() {
                                None
                            } else {
                                Some(catatan_now)
                            };
                            let (id, from_state, to_state) = (
                                id.clone(),
                                from_state.clone(),
                                to_state.clone(),
                            );
                            set_submitting.set(true);
                            spawn_local(async move {
                                let result = dispatch_satker_action(
                                        &id,
                                        &from_state,
                                        &to_state,
                                        version,
                                        catatan_opt,
                                    )
                                    .await;
                                match result {
                                    Ok(msg) => {
                                        set_action_msg.set(Some((true, msg)));
                                        set_catatan.set(String::new());
                                        set_reload_tick.update(|t| *t += 1);
                                    }
                                    Err(e) => set_action_msg.set(Some((false, e.user_message()))),
                                }
                                set_submitting.set(false);
                            });
                        };
                        view! {
                            <button
                                type="button"
                                on:click=on_click
                                disabled=move || submitting.get()
                                class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-40"
                            >
                                {transition.action_label.clone()}
                            </button>
                        }
                    }
                </For>
            </div>
        </SectionCard>
    }
        .into_any()
}

/// Route a satker approval move to the endpoint that owns it.
///
/// Keyed on the (from, to) pair, mirroring `PemakaianBmnAction::for_transition`
/// on the server. The target alone is not enough: BOTH return paths land in
/// `REVISI_OPERATOR`, and it is the SOURCE state that says which role is
/// returning — validator from `SUBMITTED`, approver from
/// `SUBMITTED_APPROVER_SATKER`. Deciding from the pair keeps every error
/// meaningful; calling one endpoint and retrying the other on failure would
/// swallow a real rejection (a stale `expected_version`, say) and report it as
/// the wrong thing.
async fn dispatch_satker_action(
    id: &str,
    from_state: &str,
    to_state: &str,
    version: i32,
    catatan: Option<String>,
) -> Result<String, AppError> {
    match (from_state, to_state) {
        ("SUBMITTED", "SUBMITTED_APPROVER_SATKER") => {
            pemakaian_bmn::validator_satker_action(id, "forward", version, catatan).await?;
            Ok("Izin diteruskan ke Approver Satker.".to_string())
        }
        ("SUBMITTED", "REVISI_OPERATOR") => {
            pemakaian_bmn::validator_satker_action(id, "return", version, catatan).await?;
            Ok("Izin dikembalikan ke Operator untuk revisi.".to_string())
        }
        ("SUBMITTED_APPROVER_SATKER", "APPROVED") => {
            pemakaian_bmn::approver_satker_action(id, "approve", version, catatan).await?;
            Ok("Izin pemakaian BMN disetujui.".to_string())
        }
        ("SUBMITTED_APPROVER_SATKER", "REVISI_OPERATOR") => {
            pemakaian_bmn::approver_satker_action(id, "return", version, catatan).await?;
            Ok("Izin dikembalikan ke Operator untuk revisi.".to_string())
        }
        ("REVISI_OPERATOR", "SUBMITTED") => {
            pemakaian_bmn::resubmit_pemakaian_bmn(id, version).await?;
            Ok("Izin diajukan ulang ke Validator Satker.".to_string())
        }
        (from, to) => Err(AppError::Unknown(format!(
            "Transisi '{}' → '{}' tidak dikenali oleh antarmuka.",
            from, to
        ))),
    }
}

#[component]
fn LifecycleSummary(
    status_label: String,
    status_tone: String,
    days: Option<i64>,
    expiring_soon: bool,
    periode: String,
    can_renew: bool,
    can_revoke: bool,
    on_renew: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_revoke: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let status_tone = status_tone_classes(&status_tone);

    let countdown = match days {
        Some(d) if d < 0 => (format!("Lewat {} hari", -d), "text-danger-300"),
        Some(0) => ("Berakhir hari ini".to_string(), "text-warning-300"),
        Some(d) if d <= 7 => (format!("{} hari lagi", d), "text-warning-300"),
        Some(d) => (format!("{} hari lagi", d), "text-slate-300"),
        None => ("—".to_string(), "text-slate-500"),
    };

    let warning_banner = expiring_soon.then(|| view! {
        <div class="mt-3 flex items-start gap-2 rounded-lg border border-warning-500/30 bg-warning-500/10 px-3 py-2 text-xs text-warning-200">
            <span class="mt-0.5 text-warning-300">
                <AppIcon icon=WARNING />
            </span>
            <span>
                "Izin akan berakhir dalam 7 hari. Pertimbangkan perpanjangan sebelum masa aktif habis."
            </span>
        </div>
    });

    view! {
        <SectionCard
            title="Lifecycle"
            description="Status dan sisa waktu izin."
            icon="fas fa-gauge-high"
            actions=Box::new(move || {
                view! {
                    <button
                        type="button"
                        on:click=on_renew
                        disabled=!can_renew
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-40"
                    >
                        <span class="text-[0.6rem]">
                            <AppIcon icon=REPEAT />
                        </span>
                        "Perpanjang"
                    </button>
                    <button
                        type="button"
                        on:click=on_revoke
                        disabled=!can_revoke
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-1.5 text-xs font-semibold text-danger-200 transition hover:bg-danger-500/20 disabled:opacity-40"
                    >
                        <span class="text-[0.6rem]">
                            <AppIcon icon=PROHIBIT />
                        </span>
                        "Cabut Izin"
                    </button>
                }
                    .into_any()
            })
        >
            <div class="grid gap-4 sm:grid-cols-3">
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Status"</p>
                    <p class="mt-1">
                        <span class=format!(
                            "inline-flex rounded-full px-2.5 py-0.5 text-[0.7rem] font-semibold ring-1 {}",
                            status_tone,
                        )>{status_label}</span>
                    </p>
                </div>
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Periode"</p>
                    <p class="mt-1 text-sm text-slate-100">{periode}</p>
                </div>
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                        "Sisa Waktu"
                    </p>
                    <p class=format!("mt-1 text-sm font-semibold {}", countdown.1)>{countdown.0}</p>
                </div>
            </div>
            {warning_banner}
        </SectionCard>
    }
}

#[component]
fn InfoField(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
            <dt class="text-[0.65rem] uppercase tracking-wide text-slate-500">{label}</dt>
            <dd class="mt-1 text-sm text-slate-100">{value}</dd>
        </div>
    }
}

#[component]
fn RevokeModal(
    reason: ReadSignal<String>,
    set_reason: WriteSignal<String>,
    submitting: ReadSignal<bool>,
    on_close: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_submit: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let on_reason = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        {
            set_reason.set(target.value());
        }
    };
    view! {
        <div class="fixed inset-0 z-modal flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div class="w-[95vw] max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-5 shadow-2xl">
                <header class="flex items-start justify-between gap-3">
                    <div>
                        <h2 class="text-base font-semibold text-white">"Cabut Izin Pemakaian"</h2>
                        <p class="mt-1 text-xs text-slate-400">
                            "Berikan alasan pencabutan. Aksi ini mengubah status izin menjadi REVOKED."
                        </p>
                    </div>
                    <button
                        type="button"
                        on:click=on_close
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-2 py-1 text-xs text-slate-300 transition hover:bg-white/[0.08]"
                    >
                        <AppIcon icon=X />
                    </button>
                </header>
                <textarea
                    class="focus-ring mt-4 w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                    rows="4"
                    placeholder="Tuliskan alasan pencabutan..."
                    on:input=on_reason
                    prop:value=move || reason.get()
                ></textarea>
                <div class="mt-4 flex justify-end gap-2">
                    <button
                        type="button"
                        on:click=on_close
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08]"
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        on:click=on_submit
                        disabled=move || submitting.get()
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-danger-500/80 px-3 py-1.5 text-xs font-bold text-white shadow-sm transition hover:bg-danger-500 disabled:opacity-50"
                    >
                        <span class="text-[0.6rem]">
                            <AppIcon icon=PROHIBIT />
                        </span>
                        {move || if submitting.get() { "Memproses..." } else { "Cabut" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn RenewModal(
    start: ReadSignal<String>,
    end: ReadSignal<String>,
    keperluan: ReadSignal<String>,
    set_start: WriteSignal<String>,
    set_end: WriteSignal<String>,
    set_keperluan: WriteSignal<String>,
    submitting: ReadSignal<bool>,
    on_close: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_submit: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let on_start = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_start.set(target.value());
        }
    };
    let on_end = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_end.set(target.value());
        }
    };
    let on_keperluan = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
        {
            set_keperluan.set(target.value());
        }
    };
    view! {
        <div class="fixed inset-0 z-modal flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div class="w-[95vw] max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-5 shadow-2xl">
                <header class="flex items-start justify-between gap-3">
                    <div>
                        <h2 class="text-base font-semibold text-white">"Perpanjang Izin"</h2>
                        <p class="mt-1 text-xs text-slate-400">
                            "Tentukan periode perpanjangan dan keperluan lanjutannya."
                        </p>
                    </div>
                    <button
                        type="button"
                        on:click=on_close
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-2 py-1 text-xs text-slate-300 transition hover:bg-white/[0.08]"
                    >
                        <AppIcon icon=X />
                    </button>
                </header>
                <div class="mt-4 grid gap-3 sm:grid-cols-2">
                    <label class="flex flex-col gap-1">
                        <span class="text-[0.65rem] uppercase tracking-wide text-slate-400">
                            "Tanggal Mulai"
                        </span>
                        <input
                            type="date"
                            class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                            on:input=on_start
                            prop:value=move || start.get()
                        />
                    </label>
                    <label class="flex flex-col gap-1">
                        <span class="text-[0.65rem] uppercase tracking-wide text-slate-400">
                            "Tanggal Selesai"
                        </span>
                        <input
                            type="date"
                            class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                            on:input=on_end
                            prop:value=move || end.get()
                        />
                    </label>
                </div>
                <label class="mt-3 flex flex-col gap-1">
                    <span class="text-[0.65rem] uppercase tracking-wide text-slate-400">
                        "Keperluan"
                    </span>
                    <textarea
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        rows="3"
                        placeholder="Alasan perpanjangan izin..."
                        on:input=on_keperluan
                        prop:value=move || keperluan.get()
                    ></textarea>
                </label>
                <div class="mt-4 flex justify-end gap-2">
                    <button
                        type="button"
                        on:click=on_close
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08]"
                    >
                        "Batal"
                    </button>
                    <button
                        type="button"
                        on:click=on_submit
                        disabled=move || submitting.get()
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                    >
                        <span class="text-[0.6rem]">
                            <AppIcon icon=REPEAT />
                        </span>
                        {move || if submitting.get() { "Memproses..." } else { "Perpanjang" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
