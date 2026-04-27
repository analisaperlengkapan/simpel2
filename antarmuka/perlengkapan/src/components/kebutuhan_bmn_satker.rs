//! Kebutuhan BMN Satker Detail Component
//!
//! Displays satker-specific view with goods list, workflow, and analysis.

use crate::api::{
    AnalisisKelayakanResponse, CreateKebutuhanBmnBarangRequest, KebutuhanBmnStatus,
    KebutuhanValidatorWilayahActionRequest, PengajuanKebutuhanBmnAktivitas,
    PengajuanKebutuhanBmnBarang, SatkerWithBarangResponse, SubmitKebutuhanSatkerRequest,
    ValidatorPusatKeputusanRequest, WorkflowTransitionRequest, create_kebutuhan_bmn_barang,
    delete_kebutuhan_bmn_barang, fetch_satker_aktivitas, fetch_satker_analisis,
    fetch_satker_with_barang, kebutuhan_validator_pusat_keputusan,
    kebutuhan_validator_wilayah_action, submit_kebutuhan_satker_to_wilayah,
    transition_satker_status,
};
use crate::components::layout::{ErrorState, FormField, LoadingState, PageLayout, SectionCard};
use crate::features::auth::AuthService;
use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{ARROW_COUNTER_CLOCKWISE, ARROW_LEFT, ARROW_RIGHT, CHECK, CHECK_CIRCLE, DATABASE, FAST_FORWARD, INFO, LIST, PACKAGE, PAPER_PLANE_TILT, PLUS, TRASH, WARNING_CIRCLE, X, X_CIRCLE};
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

fn sync_state_badge_class(state: &str) -> &'static str {
    if state.contains("SUCCESS") {
        "bg-success-500/15 text-success-300 ring-1 ring-success-500/25"
    } else if state.contains("FAILED") {
        "bg-danger-500/15 text-danger-300 ring-1 ring-danger-500/25"
    } else if state.contains("RUNNING") {
        "bg-info-500/15 text-info-300 ring-1 ring-info-500/25"
    } else {
        "bg-slate-500/15 text-slate-300 ring-1 ring-slate-500/25"
    }
}

fn sync_state_label(state: &str) -> &'static str {
    if state.contains("SUCCESS") {
        "Sinkron Berhasil"
    } else if state.contains("FAILED") {
        "Sinkron Gagal"
    } else if state.contains("RUNNING") {
        "Sinkron Berjalan"
    } else if state.contains("IDLE") {
        "Idle"
    } else {
        "Tidak Diketahui"
    }
}

fn sync_state_is_risky(state: &str, error_message: Option<&str>) -> bool {
    state.contains("FAILED") || error_message.is_some_and(|e| !e.trim().is_empty())
}

fn is_override_reason_valid(reason: &str) -> bool {
    reason.trim().len() >= 20
}

/// Kondisi badge for SIMAN assets.
fn kondisi_badge_class(kondisi: &str) -> &'static str {
    match kondisi {
        "Baik" => "bg-success-500/15 text-success-300",
        "Rusak Ringan" => "bg-warning-500/15 text-warning-300",
        _ => "bg-danger-500/15 text-danger-300",
    }
}

/// Gap text color.
fn gap_class(gap: i32, jumlah: i32) -> &'static str {
    if gap <= 0 {
        "text-success-400"
    } else if gap < jumlah / 2 {
        "text-warning-400"
    } else {
        "text-danger-400"
    }
}

#[component]
pub fn KebutuhanBmnSatkerDetail() -> impl IntoView {
    let params = use_params_map();
    let satker_id = Memo::new(move |_| params.read().get("satker_id").clone().unwrap_or_default());

    // Data state
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<String>>(None);
    let (satker_data, set_satker_data) = signal::<Option<SatkerWithBarangResponse>>(None);
    let (aktivitas, set_aktivitas) = signal::<Vec<PengajuanKebutuhanBmnAktivitas>>(vec![]);
    let (analisis, set_analisis) = signal::<Option<AnalisisKelayakanResponse>>(None);

    // UI state
    let (active_tab, set_active_tab) = signal("barang");
    let (show_add_barang, set_show_add_barang) = signal(false);
    let (submitting, set_submitting) = signal(false);

    // New barang form state
    let (new_nama, set_new_nama) = signal(String::new());
    let (new_kode_barang, set_new_kode_barang) = signal::<Option<String>>(None);
    let (new_jumlah, set_new_jumlah) = signal(1i32);
    let (new_satuan, set_new_satuan) = signal("Unit".to_string());
    let (new_alasan, set_new_alasan) = signal::<Option<String>>(None);

    // Load satker data
    let load_data = move |sid: String| {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            match fetch_satker_with_barang(&sid).await {
                Ok(response) => {
                    set_satker_data.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(e.user_message()));
                }
            }
            set_loading.set(false);
        });
    };

    // Load aktivitas
    let load_aktivitas = move |sid: String| {
        spawn_local(async move {
            if let Ok(response) = fetch_satker_aktivitas(&sid).await {
                set_aktivitas.set(response.data);
            }
        });
    };

    // Load analisis
    let load_analisis = move |sid: String| {
        spawn_local(async move {
            if let Ok(response) = fetch_satker_analisis(&sid).await {
                set_analisis.set(Some(response.data));
            }
        });
    };

    // Initial load
    Effect::new(move || {
        let sid = satker_id.get();
        if !sid.is_empty() {
            load_data(sid.clone());
            load_aktivitas(sid.clone());
            load_analisis(sid);
        }
    });

    // Handle add barang
    let handle_add_barang = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        let sid = satker_id.get();
        let request = CreateKebutuhanBmnBarangRequest {
            nama: new_nama.get(),
            kode_barang: new_kode_barang.get(),
            jumlah: new_jumlah.get(),
            satuan: new_satuan.get(),
            alasan: new_alasan.get(),
            keterangan: None,
            file_pendukung: vec![],
        };

        spawn_local(async move {
            match create_kebutuhan_bmn_barang(&sid, request).await {
                Ok(_) => {
                    set_new_nama.set(String::new());
                    set_new_kode_barang.set(None);
                    set_new_jumlah.set(1);
                    set_new_alasan.set(None);
                    set_show_add_barang.set(false);
                    load_data(sid);
                }
                Err(e) => {
                    set_error.set(Some(e.user_message()));
                }
            }
            set_submitting.set(false);
        });
    };

    // Handle delete barang
    let handle_delete_barang = move |barang_id: String| {
        let sid = satker_id.get();
        spawn_local(async move {
            match delete_kebutuhan_bmn_barang(&barang_id).await {
                Ok(_) => load_data(sid),
                Err(e) => set_error.set(Some(e.user_message())),
            }
        });
    };

    // Workflow action signals
    let (show_return_modal, set_show_return_modal) = signal(false);
    let (show_reject_modal, set_show_reject_modal) = signal(false);
    let (return_catatan, set_return_catatan) = signal(String::new());
    let (reject_alasan, set_reject_alasan) = signal(String::new());
    let (action_loading, set_action_loading) = signal(false);
    let (override_enabled, set_override_enabled) = signal(false);
    let (override_reason, set_override_reason) = signal(String::new());

    let is_validator_decision_blocked = move || {
        analisis
            .get()
            .and_then(|a| a.integrasi_sync)
            .map(|sync| {
                sync_state_is_risky(
                    &sync.mysimkari.state,
                    sync.mysimkari.error_message.as_deref(),
                ) || sync_state_is_risky(&sync.siman.state, sync.siman.error_message.as_deref())
            })
            .unwrap_or(false)
    };

    let is_override_allowed = move || {
        AuthService::load_session()
            .map(|session| session.is_admin())
            .unwrap_or(false)
    };

    let can_validator_decide = move || {
        !is_validator_decision_blocked()
            || (is_override_allowed()
                && override_enabled.get()
                && is_override_reason_valid(&override_reason.get()))
    };

    // Submit satker to wilayah
    let handle_submit_to_wilayah = move |_| {
        let sid = satker_id.get();
        set_action_loading.set(true);
        spawn_local(async move {
            let request = SubmitKebutuhanSatkerRequest {
                catatan_satker: None,
                lampiran_surat_permohonan: String::new(),
                lampiran_pendukung: vec![],
            };
            match submit_kebutuhan_satker_to_wilayah(&sid, request).await {
                Ok(_) => {
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    // Validator wilayah: forward to pusat
    let handle_forward_to_pusat = move |_| {
        let sid = satker_id.get();
        set_action_loading.set(true);
        spawn_local(async move {
            let request = KebutuhanValidatorWilayahActionRequest {
                aksi: "forward".to_string(),
                catatan: None,
            };
            match kebutuhan_validator_wilayah_action(&sid, request).await {
                Ok(_) => {
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    // Validator wilayah: return to operator
    let handle_return_to_operator = move |_: leptos::ev::SubmitEvent| {
        let sid = satker_id.get();
        set_action_loading.set(true);
        spawn_local(async move {
            let request = KebutuhanValidatorWilayahActionRequest {
                aksi: "return".to_string(),
                catatan: Some(return_catatan.get()),
            };
            match kebutuhan_validator_wilayah_action(&sid, request).await {
                Ok(_) => {
                    set_show_return_modal.set(false);
                    set_return_catatan.set(String::new());
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    // Validator pusat: approve
    let handle_approve = move |_| {
        if !can_validator_decide() {
            set_error.set(Some(
                "Keputusan Validator Pusat dikunci sementara. Aktifkan override darurat (khusus admin) dan isi alasan minimal 20 karakter."
                    .to_string(),
            ));
            return;
        }

        let use_override =
            is_validator_decision_blocked() && is_override_allowed() && override_enabled.get();
        let override_reason_text = override_reason.get().trim().to_string();
        let sid = satker_id.get();
        set_action_loading.set(true);
        spawn_local(async move {
            let request = ValidatorPusatKeputusanRequest {
                is_approved: true,
                alasan: None,
                override_darurat: Some(use_override),
                override_reason: if use_override {
                    Some(override_reason_text)
                } else {
                    None
                },
            };
            match kebutuhan_validator_pusat_keputusan(&sid, request).await {
                Ok(_) => {
                    set_override_enabled.set(false);
                    set_override_reason.set(String::new());
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    // Validator pusat: reject
    let handle_reject = move |_: leptos::ev::SubmitEvent| {
        if !can_validator_decide() {
            set_error.set(Some(
                "Keputusan Validator Pusat dikunci sementara. Aktifkan override darurat (khusus admin) dan isi alasan minimal 20 karakter."
                    .to_string(),
            ));
            return;
        }

        let use_override =
            is_validator_decision_blocked() && is_override_allowed() && override_enabled.get();
        let override_reason_text = override_reason.get().trim().to_string();
        let reject_reason = reject_alasan.get();

        let sid = satker_id.get();
        set_action_loading.set(true);
        spawn_local(async move {
            let request = ValidatorPusatKeputusanRequest {
                is_approved: false,
                alasan: Some(reject_reason),
                override_darurat: Some(use_override),
                override_reason: if use_override {
                    Some(override_reason_text)
                } else {
                    None
                },
            };
            match kebutuhan_validator_pusat_keputusan(&sid, request).await {
                Ok(_) => {
                    set_show_reject_modal.set(false);
                    set_reject_alasan.set(String::new());
                    set_override_enabled.set(false);
                    set_override_reason.set(String::new());
                    load_data(sid.clone());
                    load_aktivitas(sid);
                }
                Err(e) => set_error.set(Some(e.user_message())),
            }
            set_action_loading.set(false);
        });
    };

    view! {
        <PageLayout
            title="Detail Satker — Kebutuhan BMN"
            icon="fas fa-building"
            description="Detail pengajuan kebutuhan BMN per satker"
        >
            // Back link
            <button
                onclick="history.back()"
                class="mb-4 inline-flex items-center gap-2 text-sm text-gold-400 transition hover:text-gold-300"
            >
                <span class="text-xs"><AppIcon icon=ARROW_LEFT /></span>
                "Kembali"
            </button>

            // Error banner
            <Show when=move || error.get().is_some()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Main content: loading / data
            {move || {
                if loading.get() {
                    return view! { <LoadingState message="Memuat data satker...".to_string() /> }.into_any();
                }

                let Some(data) = satker_data.get() else {
                    return view! {
                        <div class="py-8 text-center text-sm text-slate-500">"Data tidak tersedia"</div>
                    }.into_any();
                };

                let satker = data.satker.clone();
                let barang_list = data.barang_list.clone();
                let has_barang = !barang_list.is_empty();
                let barang_list_store = StoredValue::new(barang_list);
                let status = KebutuhanBmnStatus::from_code(satker.status_kode);
                let badge_class = status.map(|s| s.badge_class()).unwrap_or("bg-slate-500/15 text-slate-300 ring-1 ring-slate-500/25");
                let status_label = status.map(|s| s.label()).unwrap_or("Unknown");
                let sk = satker.status_kode;

                view! {
                    <div class="space-y-5">
                        // Header
                        <div class="flex flex-col items-start justify-between gap-4 lg:flex-row lg:items-center">
                            <div>
                                <h2 class="text-xl font-bold text-slate-100">
                                    {satker.nm_satker.clone().unwrap_or_else(|| satker.ms_satker_id.clone())}
                                </h2>
                                <p class="mt-1 text-sm text-slate-400">
                                    "Kode Satker: " <span class="font-mono text-slate-300">{satker.ms_satker_id.clone()}</span>
                                </p>
                            </div>
                            <span class=format!("inline-flex items-center rounded-full px-3 py-1 text-xs font-medium {}", badge_class)>
                                {status_label}
                            </span>
                        </div>

                        // Stats cards
                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
                            <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                <div class="text-xs font-medium text-info-400">"Total Barang"</div>
                                <div class="mt-1 text-2xl font-bold text-slate-100">{data.total_barang}</div>
                            </div>
                            <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                <div class="text-xs font-medium text-purple-400">"Total Jumlah Diminta"</div>
                                <div class="mt-1 text-2xl font-bold text-slate-100">{data.total_jumlah}</div>
                            </div>
                            <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                <div class="text-xs font-medium text-success-400">"Prioritas"</div>
                                <div class="mt-1 text-2xl font-bold text-slate-100">{satker.prioritas}</div>
                            </div>
                        </div>

                        // ── Workflow Action Panels ──

                        // Operator: Submit to Wilayah (Draft=2000 or RevisiSatker=2001)
                        <Show when=move || sk == 2000 || sk == 2001>
                            <div class="flex items-center justify-between rounded-xl border border-gold-500/30 bg-gold-500/[0.08] p-4">
                                <div>
                                    <p class="text-sm font-medium text-gold-300">"Siap diajukan?"</p>
                                    <p class="text-xs text-gold-400/70">"Kirim pengajuan ke Validator Wilayah untuk verifikasi."</p>
                                </div>
                                <button
                                    class="inline-flex items-center gap-2 rounded-lg bg-info-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-info-700 disabled:opacity-50"
                                    on:click=handle_submit_to_wilayah
                                    prop:disabled=move || action_loading.get()
                                >
                                    <span class="text-xs"><AppIcon icon=PAPER_PLANE_TILT /></span>
                                    "Submit ke Wilayah"
                                </button>
                            </div>
                        </Show>

                        // Validator Wilayah: Forward/Return (SubmitWilayah=2002)
                        <Show when=move || sk == 2002>
                            <SectionCard title="Tindakan Validator Wilayah">
                                <div class="flex gap-3">
                                    <button
                                        class="inline-flex items-center gap-2 rounded-lg bg-success-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-success-700 disabled:opacity-50"
                                        on:click=handle_forward_to_pusat
                                        prop:disabled=move || action_loading.get()
                                    >
                                        <span class="text-xs"><AppIcon icon=FAST_FORWARD /></span>
                                        "Teruskan ke Pusat"
                                    </button>
                                    <button
                                        class="inline-flex items-center gap-2 rounded-lg bg-warning-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-warning-700 disabled:opacity-50"
                                        on:click=move |_| set_show_return_modal.set(true)
                                        prop:disabled=move || action_loading.get()
                                    >
                                        <span class="text-xs"><AppIcon icon=ARROW_COUNTER_CLOCKWISE /></span>
                                        "Kembalikan ke Operator"
                                    </button>
                                </div>
                            </SectionCard>
                        </Show>

                        // Validator Pusat: Approve/Reject (AnalisisKelayakan=2005)
                        <Show when=move || sk == 2005>
                            <SectionCard title="Keputusan Validator Pusat">
                                // Blocked warning
                                <Show when=move || is_validator_decision_blocked()>
                                    <div class="mb-4 rounded-lg border border-danger-500/30 bg-danger-500/[0.08] p-3 text-sm text-danger-300">
                                        <div class="font-medium">"Aksi dikunci: kualitas data integrasi belum aman"</div>
                                        <div class="mt-1 text-xs text-danger-400/70">
                                            {move || {
                                                analisis
                                                    .get()
                                                    .and_then(|a| a.integrasi_sync)
                                                    .map(|sync| {
                                                        let mut failed: Vec<&str> = vec![];
                                                        if sync_state_is_risky(&sync.mysimkari.state, sync.mysimkari.error_message.as_deref()) {
                                                            failed.push("MySIMKARI");
                                                        }
                                                        if sync_state_is_risky(&sync.siman.state, sync.siman.error_message.as_deref()) {
                                                            failed.push("SIMAN");
                                                        }
                                                        if failed.is_empty() {
                                                            "Periksa status sinkronisasi sebelum melanjutkan.".to_string()
                                                        } else {
                                                            format!("Sumber bermasalah: {}. Lakukan sinkronisasi ulang atau verifikasi manual.", failed.join(", "))
                                                        }
                                                    })
                                                    .unwrap_or_else(|| "Status sinkronisasi belum tersedia.".to_string())
                                            }}
                                        </div>

                                        // Override darurat (admin only)
                                        <Show when=move || is_override_allowed()>
                                            <div class="mt-3 rounded-lg border border-white/[0.06] bg-surface-panel p-3">
                                                <label class="flex items-start gap-2 text-sm font-medium text-slate-200">
                                                    <input
                                                        type="checkbox"
                                                        class="mt-0.5"
                                                        checked=move || override_enabled.get()
                                                        on:change=move |ev| set_override_enabled.set(event_target_checked(&ev))
                                                    />
                                                    <span>"Aktifkan override darurat (khusus admin)"</span>
                                                </label>
                                                <textarea
                                                    rows="3"
                                                    class="focus-ring mt-2 w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                                    placeholder="Wajib diisi minimal 20 karakter. Contoh: keputusan mendesak karena tenggat operasional..."
                                                    on:input=move |ev| set_override_reason.set(event_target_value(&ev))
                                                    prop:value=move || override_reason.get()
                                                ></textarea>
                                                <div class="mt-1 text-xs text-slate-500">
                                                    {move || format!("Panjang alasan: {} karakter", override_reason.get().trim().len())}
                                                </div>
                                                <Show when=move || override_enabled.get() && !is_override_reason_valid(&override_reason.get())>
                                                    <div class="mt-1 text-xs text-danger-400">"Alasan override minimal 20 karakter."</div>
                                                </Show>
                                            </div>
                                        </Show>
                                    </div>
                                </Show>
                                <div class="flex gap-3">
                                    <button
                                        class="inline-flex items-center gap-2 rounded-lg bg-success-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-success-700 disabled:opacity-50"
                                        on:click=handle_approve
                                        prop:disabled=move || action_loading.get() || !can_validator_decide()
                                    >
                                        <span class="text-xs"><AppIcon icon=CHECK /></span>
                                        "Setujui"
                                    </button>
                                    <button
                                        class="inline-flex items-center gap-2 rounded-lg bg-danger-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-danger-700 disabled:opacity-50"
                                        on:click=move |_| set_show_reject_modal.set(true)
                                        prop:disabled=move || action_loading.get() || !can_validator_decide()
                                    >
                                        <span class="text-xs"><AppIcon icon=X /></span>
                                        "Tolak"
                                    </button>
                                </div>
                            </SectionCard>
                        </Show>

                        // Completed / Rejected banners
                        <Show when=move || sk == 2006>
                            <div class="flex items-center gap-3 rounded-xl border border-success-500/30 bg-success-500/[0.08] p-4">
                                <span class="text-xl text-success-400"><AppIcon icon=CHECK_CIRCLE /></span>
                                <div>
                                    <p class="text-sm font-medium text-success-300">"Pengajuan Disetujui"</p>
                                    <p class="text-xs text-success-400/70">"Pengajuan kebutuhan BMN telah disetujui oleh Validator Pusat."</p>
                                </div>
                            </div>
                        </Show>
                        <Show when=move || sk == 2007>
                            <div class="flex items-center gap-3 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] p-4">
                                <span class="text-xl text-danger-400"><AppIcon icon=X_CIRCLE /></span>
                                <div>
                                    <p class="text-sm font-medium text-danger-300">"Pengajuan Ditolak"</p>
                                    <p class="text-xs text-danger-400/70">"Pengajuan kebutuhan BMN ditolak oleh Validator Pusat."</p>
                                </div>
                            </div>
                        </Show>

                        // ── Tabs ──
                        <div class="border-b border-white/[0.06]">
                            <nav class="flex gap-1">
                                {["barang", "analisis", "aktivitas"].into_iter().map(|tab| {
                                    let icon = match tab {
                                        "barang" => "fas fa-boxes",
                                        "analisis" => "fas fa-chart-bar",
                                        _ => "fas fa-history",
                                    };
                                    let label = match tab {
                                        "barang" => "Daftar Barang",
                                        "analisis" => "Analisis Kelayakan",
                                        _ => "Riwayat Aktivitas",
                                    };
                                    view! {
                                        <button
                                            class=move || format!(
                                                "border-b-2 px-4 py-2.5 text-sm font-medium transition {}",
                                                if active_tab.get() == tab {
                                                    "border-gold-400 text-gold-400"
                                                } else {
                                                    "border-transparent text-slate-500 hover:text-slate-300"
                                                }
                                            )
                                            on:click=move |_| set_active_tab.set(tab)
                                        >
                                            <span class="mr-1.5 inline-flex">
                                                <AppIcon icon=icon_from_fa_class(icon) size=10 />
                                            </span>
                                            {label}
                                        </button>
                                    }
                                }).collect_view()}
                            </nav>
                        </div>

                        // ── Tab: Barang ──
                        <Show when=move || active_tab.get() == "barang">
                            <div>
                                <div class="mb-4 flex items-center justify-between">
                                    <h3 class="text-sm font-semibold text-slate-200">"Daftar Barang"</h3>
                                    <button
                                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                                        on:click=move |_| set_show_add_barang.set(true)
                                    >
                                        <span class="text-xs"><AppIcon icon=PLUS /></span>
                                        "Tambah Barang"
                                    </button>
                                </div>

                                <Show
                                    when=move || has_barang
                                    fallback=|| view! {
                                        <div class="py-8 text-center">
                                            <span class="mb-2 text-2xl text-slate-600"><AppIcon icon=PACKAGE /></span>
                                            <p class="text-sm text-slate-500">"Belum ada barang yang ditambahkan"</p>
                                        </div>
                                    }
                                >
                                    <div class="overflow-hidden rounded-xl border border-white/[0.06]">
                                        <div class="overflow-x-auto">
                                            <table class="min-w-full divide-y divide-white/[0.04]">
                                                <thead class="bg-white/[0.02]">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Barang"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Kode"</th>
                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Jumlah"</th>
                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Disetujui"</th>
                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Prioritas"</th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Alasan"</th>
                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Aksi"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || barang_list_store.get_value().into_iter()
                                                        key=|b| b.id.clone()
                                                        children=move |barang: PengajuanKebutuhanBmnBarang| {
                                                            let barang_id = barang.id.clone();
                                                            view! {
                                                                <tr class="border-b border-white/[0.04] transition hover:bg-white/[0.02]">
                                                                    <td class="px-4 py-3 text-sm font-medium text-slate-200">{barang.nama.clone()}</td>
                                                                    <td class="px-4 py-3 font-mono text-xs text-slate-400">{barang.kode_barang.clone().unwrap_or("-".into())}</td>
                                                                    <td class="px-4 py-3 text-center text-sm text-slate-300">{format!("{} {}", barang.jumlah, barang.satuan)}</td>
                                                                    <td class="px-4 py-3 text-center text-sm font-medium text-success-400">{barang.jml_setuju}</td>
                                                                    <td class="px-4 py-3 text-center">
                                                                        <span class="inline-flex items-center rounded-full bg-info-500/15 px-2 py-0.5 text-xs font-medium text-info-300 ring-1 ring-info-500/25">
                                                                            {barang.prioritas}
                                                                        </span>
                                                                    </td>
                                                                    <td class="max-w-xs truncate px-4 py-3 text-sm text-slate-400">{barang.alasan.clone().unwrap_or("-".into())}</td>
                                                                    <td class="px-4 py-3 text-center">
                                                                        <button
                                                                            class="text-danger-400 transition hover:text-danger-300"
                                                                            on:click=move |_| handle_delete_barang(barang_id.clone())
                                                                        >
                                                                            <span class="text-xs"><AppIcon icon=TRASH /></span>
                                                                        </button>
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }
                                                    />
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </Show>
                            </div>
                        </Show>

                        // ── Tab: Analisis ──
                        <Show when=move || active_tab.get() == "analisis">
                            <div>
                                {move || {
                                    analisis.get().map(|a| {
                                        let sync_data = a.integrasi_sync.clone();
                                        let barang_items = StoredValue::new(a.barang_list.clone());

                                        view! {
                                            <div class="space-y-5">
                                                // Summary stats
                                                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                                                    <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                        <div class="text-xs font-medium text-info-400">"Total Diminta"</div>
                                                        <div class="mt-1 text-2xl font-bold text-slate-100">{a.summary.total_diminta}</div>
                                                    </div>
                                                    <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                        <div class="text-xs font-medium text-success-400">"Total Existing"</div>
                                                        <div class="mt-1 text-2xl font-bold text-slate-100">{a.summary.total_existing}</div>
                                                    </div>
                                                    <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                        <div class="text-xs font-medium text-warning-400">"Gap Kebutuhan"</div>
                                                        <div class="mt-1 text-2xl font-bold text-slate-100">{a.summary.total_gap}</div>
                                                    </div>
                                                    <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                        <div class="text-xs font-medium text-purple-400">"% Kelayakan"</div>
                                                        <div class="mt-1 text-2xl font-bold text-slate-100">{format!("{:.1}%", a.summary.kelayakan_persen)}</div>
                                                    </div>
                                                </div>

                                                // Integrasi sync status
                                                {sync_data.map(|sync| {
                                                    let mysimkari = sync.mysimkari;
                                                    let siman = sync.siman;
                                                    let my_badge = sync_state_badge_class(&mysimkari.state);
                                                    let si_badge = sync_state_badge_class(&siman.state);
                                                    let my_label = sync_state_label(&mysimkari.state);
                                                    let si_label = sync_state_label(&siman.state);
                                                    let my_last = mysimkari.last_sync_at.clone().unwrap_or_else(|| "Belum tersedia".to_string());
                                                    let si_last = siman.last_sync_at.clone().unwrap_or_else(|| "Belum tersedia".to_string());
                                                    let my_err = mysimkari.error_message.clone();
                                                    let my_has_err = my_err.is_some();
                                                    let my_err_text = my_err.unwrap_or_default();
                                                    let si_err = siman.error_message.clone();
                                                    let si_has_err = si_err.is_some();
                                                    let si_err_text = si_err.unwrap_or_default();

                                                    view! {
                                                        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
                                                            // MySIMKARI sync card
                                                            <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                                <div class="flex items-start justify-between gap-3">
                                                                    <div>
                                                                        <div class="text-sm font-semibold text-slate-200">"MySIMKARI Sync"</div>
                                                                        <div class="mt-1 text-xs text-slate-400">
                                                                            {format!("Records tersinkron: {}", mysimkari.records_synced)}
                                                                        </div>
                                                                    </div>
                                                                    <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {}", my_badge)>
                                                                        {my_label}
                                                                    </span>
                                                                </div>
                                                                <div class="mt-3 space-y-1 text-xs text-slate-500">
                                                                    <div><span class="font-medium">"Last Sync:"</span> " " {my_last}</div>
                                                                    <Show when=move || my_has_err>
                                                                        <div class="text-danger-400">
                                                                            <span class="font-medium">"Error:"</span> " " {my_err_text.clone()}
                                                                        </div>
                                                                    </Show>
                                                                </div>
                                                            </div>
                                                            // SIMAN sync card
                                                            <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                                <div class="flex items-start justify-between gap-3">
                                                                    <div>
                                                                        <div class="text-sm font-semibold text-slate-200">"SIMAN Sync"</div>
                                                                        <div class="mt-1 text-xs text-slate-400">
                                                                            {format!("Records tersinkron: {}", siman.records_synced)}
                                                                        </div>
                                                                    </div>
                                                                    <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {}", si_badge)>
                                                                        {si_label}
                                                                    </span>
                                                                </div>
                                                                <div class="mt-3 space-y-1 text-xs text-slate-500">
                                                                    <div><span class="font-medium">"Last Sync:"</span> " " {si_last}</div>
                                                                    <Show when=move || si_has_err>
                                                                        <div class="text-danger-400">
                                                                            <span class="font-medium">"Error:"</span> " " {si_err_text.clone()}
                                                                        </div>
                                                                    </Show>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }
                                                })}

                                                // Gap analysis table
                                                <SectionCard title="Detail Analisis per Barang">
                                                    <div class="overflow-hidden rounded-xl border border-white/[0.06]">
                                                        <div class="overflow-x-auto">
                                                            <table class="min-w-full divide-y divide-white/[0.04]">
                                                                <thead class="bg-white/[0.02]">
                                                                    <tr>
                                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Nama Barang"</th>
                                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Diminta"</th>
                                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Existing (SIMAN)"</th>
                                                                        <th class="px-4 py-3 text-center text-xs font-semibold uppercase tracking-wide text-slate-400">"Gap"</th>
                                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">"Rekomendasi"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody>
                                                                    <For
                                                                        each=move || barang_items.get_value()
                                                                        key=|b| b.barang.id.clone()
                                                                        children=move |item| {
                                                                            let item_store = StoredValue::new(item.clone());
                                                                            let gc = gap_class(item.gap, item.barang.jumlah);
                                                                            let existing_count = item.existing_assets.len();
                                                                            let has_existing = existing_count > 0;

                                                                            view! {
                                                                                <tr class="border-b border-white/[0.04] transition hover:bg-white/[0.02]">
                                                                                    <td class="px-4 py-3">
                                                                                        <div class="text-sm font-medium text-slate-200">{item.barang.nama.clone()}</div>
                                                                                        <Show when=move || has_existing>
                                                                                            <div class="mt-0.5 text-xs text-slate-500">
                                                                                                <span class="mr-1"><AppIcon icon=DATABASE /></span>
                                                                                                {format!("{} aset ditemukan di SIMAN", existing_count)}
                                                                                            </div>
                                                                                        </Show>
                                                                                    </td>
                                                                                    <td class="px-4 py-3 text-center text-sm font-medium text-slate-200">{item.barang.jumlah}</td>
                                                                                    <td class="px-4 py-3 text-center text-sm font-medium text-info-400">{existing_count as i32}</td>
                                                                                    <td class=format!("px-4 py-3 text-center text-sm font-bold {}", gc)>
                                                                                        {if item.gap > 0 { format!("+{}", item.gap) } else { item.gap.to_string() }}
                                                                                    </td>
                                                                                    <td class="px-4 py-3 text-sm text-slate-300">{item.recommendation.clone()}</td>
                                                                                </tr>
                                                                                // Expandable existing assets
                                                                                <Show when=move || !item_store.with_value(|i| i.existing_assets.is_empty())>
                                                                                    <tr class="bg-white/[0.015]">
                                                                                        <td colspan="5" class="p-2">
                                                                                            <details class="cursor-pointer">
                                                                                                <summary class="text-xs font-medium text-info-400">
                                                                                                    <span class="mr-1"><AppIcon icon=LIST /></span>
                                                                                                    "Lihat aset existing dari SIMAN"
                                                                                                </summary>
                                                                                                <div class="mt-2 max-h-40 space-y-1 overflow-y-auto">
                                                                                                    <For
                                                                                                        each=move || item_store.get_value().existing_assets
                                                                                                        key=|ea| ea.no_aset.clone()
                                                                                                        children=move |ea| {
                                                                                                            let kc = kondisi_badge_class(&ea.kondisi);
                                                                                                            view! {
                                                                                                                <div class="flex items-center justify-between rounded-lg bg-white/[0.03] px-2 py-1 text-xs">
                                                                                                                    <div>
                                                                                                                        <span class="font-mono text-slate-500">{ea.no_aset.clone()}</span>
                                                                                                                        " - "
                                                                                                                        <span class="font-medium text-slate-300">{ea.nama_aset.clone()}</span>
                                                                                                                    </div>
                                                                                                                    <span class=format!("rounded-full px-2 py-0.5 text-xs {}", kc)>
                                                                                                                        {ea.kondisi.clone()}
                                                                                                                    </span>
                                                                                                                </div>
                                                                                                            }
                                                                                                        }
                                                                                                    />
                                                                                                </div>
                                                                                            </details>
                                                                                        </td>
                                                                                    </tr>
                                                                                </Show>
                                                                            }
                                                                        }
                                                                    />
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    </div>

                                                    // SIMAN note
                                                    <div class="mt-4 rounded-lg border border-info-500/20 bg-info-500/[0.06] p-3 text-xs text-info-300">
                                                        <span class="mr-1.5"><AppIcon icon=INFO /></span>
                                                        "Data aset existing diambil dari SIMAN. Gap dihitung berdasarkan jumlah diminta dikurangi aset sejenis."
                                                    </div>
                                                </SectionCard>

                                                // Pegawai data (MySIMKARI)
                                                {a.data_pegawai.clone().map(|dp| {
                                                    let total_pegawai = dp.total_pegawai;
                                                    let rekap_eselon_empty = dp.rekap_eselon.is_empty();
                                                    let rekap_eselon_data = StoredValue::new(dp.rekap_eselon);
                                                    let rekap_non_eselon_empty = dp.rekap_non_eselon.is_empty();
                                                    let rekap_non_eselon_data = StoredValue::new(dp.rekap_non_eselon);
                                                    view! {
                                                        <SectionCard title="Rekap Data Pegawai (MySIMKARI)">
                                                            <div class="mb-4 grid grid-cols-1 gap-4 sm:grid-cols-2">
                                                                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                                    <div class="text-xs font-medium text-success-400">"Total Pegawai"</div>
                                                                    <div class="mt-1 text-2xl font-bold text-slate-100">{total_pegawai}</div>
                                                                </div>
                                                                <div class="rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                                    <div class="text-xs font-medium text-info-400">"Sumber Data"</div>
                                                                    <div class="mt-1 text-sm font-medium text-slate-200">"MySIMKARI — Sistem Informasi Manajemen Kepegawaian"</div>
                                                                </div>
                                                            </div>

                                                            // Eselon table
                                                            <Show when=move || !rekap_eselon_empty>
                                                                <div class="mb-4">
                                                                    <h5 class="mb-2 text-xs font-medium text-slate-400">"Rekap per Eselon"</h5>
                                                                    <div class="overflow-hidden rounded-lg border border-white/[0.06]">
                                                                        <table class="min-w-full divide-y divide-white/[0.04] text-sm">
                                                                            <thead class="bg-white/[0.02]">
                                                                                <tr>
                                                                                    <th class="px-3 py-2 text-left text-xs font-semibold uppercase text-slate-400">"Eselon"</th>
                                                                                    <th class="px-3 py-2 text-center text-xs font-semibold uppercase text-slate-400">"Jumlah"</th>
                                                                                </tr>
                                                                            </thead>
                                                                            <tbody>
                                                                                <For
                                                                                    each=move || rekap_eselon_data.get_value()
                                                                                    key=|e| e.tingkat_eselon.clone()
                                                                                    children=move |item| view! {
                                                                                        <tr class="border-b border-white/[0.04]">
                                                                                            <td class="px-3 py-2 text-slate-300">{item.tingkat_eselon}</td>
                                                                                            <td class="px-3 py-2 text-center font-medium text-slate-200">{item.jumlah}</td>
                                                                                        </tr>
                                                                                    }
                                                                                />
                                                                            </tbody>
                                                                        </table>
                                                                    </div>
                                                                </div>
                                                            </Show>

                                                            // Non-eselon table
                                                            <Show when=move || !rekap_non_eselon_empty>
                                                                <div>
                                                                    <h5 class="mb-2 text-xs font-medium text-slate-400">"Rekap per Golongan (Non-Eselon)"</h5>
                                                                    <div class="overflow-hidden rounded-lg border border-white/[0.06]">
                                                                        <table class="min-w-full divide-y divide-white/[0.04] text-sm">
                                                                            <thead class="bg-white/[0.02]">
                                                                                <tr>
                                                                                    <th class="px-3 py-2 text-left text-xs font-semibold uppercase text-slate-400">"Golongan"</th>
                                                                                    <th class="px-3 py-2 text-center text-xs font-semibold uppercase text-slate-400">"Jumlah"</th>
                                                                                </tr>
                                                                            </thead>
                                                                            <tbody>
                                                                                <For
                                                                                    each=move || rekap_non_eselon_data.get_value()
                                                                                    key=|e| e.golongan.clone()
                                                                                    children=move |item| view! {
                                                                                        <tr class="border-b border-white/[0.04]">
                                                                                            <td class="px-3 py-2 text-slate-300">{item.golongan}</td>
                                                                                            <td class="px-3 py-2 text-center font-medium text-slate-200">{item.jumlah}</td>
                                                                                        </tr>
                                                                                    }
                                                                                />
                                                                            </tbody>
                                                                        </table>
                                                                    </div>
                                                                </div>
                                                            </Show>
                                                        </SectionCard>
                                                    }.into_any()
                                                }).unwrap_or_else(|| view! { <div></div> }.into_any())}
                                            </div>
                                        }.into_any()
                                    }).unwrap_or_else(|| view! {
                                        <div class="py-8 text-center text-sm text-slate-500">"Data analisis belum tersedia"</div>
                                    }.into_any())
                                }}
                            </div>
                        </Show>

                        // ── Tab: Aktivitas ──
                        <Show when=move || active_tab.get() == "aktivitas">
                            <div>
                                <Show
                                    when=move || !aktivitas.get().is_empty()
                                    fallback=|| view! {
                                        <div class="py-8 text-center text-sm text-slate-500">"Belum ada aktivitas"</div>
                                    }
                                >
                                    <div class="space-y-3">
                                        <For
                                            each=move || aktivitas.get()
                                            key=|a| a.id.clone()
                                            children=move |akt| {
                                                let to_status = KebutuhanBmnStatus::from_code(akt.to_status_kode);
                                                let has_komentar = akt.komentar.is_some();
                                                let komentar_text = akt.komentar.clone().unwrap_or_default();
                                                view! {
                                                    <div class="flex items-start gap-4 rounded-xl border border-white/[0.06] bg-surface-panel p-4">
                                                        <div class="flex h-10 w-10 items-center justify-center rounded-full bg-info-500/15">
                                                            <span class="text-info-400"><AppIcon icon=ARROW_RIGHT /></span>
                                                        </div>
                                                        <div class="flex-1">
                                                            <div class="text-sm font-medium text-slate-200">{akt.aksi.clone()}</div>
                                                            <div class="mt-1 text-xs text-slate-400">
                                                                "Ke status: " {to_status.map(|s| s.label()).unwrap_or("Unknown")}
                                                            </div>
                                                            <Show when=move || has_komentar>
                                                                <div class="mt-2 text-xs italic text-slate-500">
                                                                    "\"" {komentar_text.clone()} "\""
                                                                </div>
                                                            </Show>
                                                            <div class="mt-2 text-xs text-slate-600">
                                                                {format!("{} — {}", akt.nama.clone().unwrap_or("System".into()), akt.created_at)}
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </Show>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}

            // ── Modals ──

            // Add barang modal
            <Show when=move || show_add_barang.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="mx-4 w-full max-w-lg rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                        <h3 class="mb-4 text-lg font-bold text-slate-100">"Tambah Barang"</h3>
                        <form on:submit=handle_add_barang class="flex flex-col gap-4">
                            <FormField label="Nama Barang" required=true>
                                <input
                                    type="text"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    on:input=move |ev| set_new_nama.set(event_target_value(&ev))
                                    prop:value=move || new_nama.get()
                                />
                            </FormField>
                            <FormField label="Kode Barang">
                                <input
                                    type="text"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        set_new_kode_barang.set(if v.is_empty() { None } else { Some(v) });
                                    }
                                />
                            </FormField>
                            <div class="grid grid-cols-2 gap-4">
                                <FormField label="Jumlah" required=true>
                                    <input
                                        type="number"
                                        min="1"
                                        required
                                        class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                                        on:input=move |ev| {
                                            if let Ok(v) = event_target_value(&ev).parse() {
                                                set_new_jumlah.set(v);
                                            }
                                        }
                                        prop:value=move || new_jumlah.get()
                                    />
                                </FormField>
                                <FormField label="Satuan">
                                    <input
                                        type="text"
                                        class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                                        on:input=move |ev| set_new_satuan.set(event_target_value(&ev))
                                        prop:value=move || new_satuan.get()
                                    />
                                </FormField>
                            </div>
                            <FormField label="Alasan / Justifikasi">
                                <textarea
                                    rows="2"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        set_new_alasan.set(if v.is_empty() { None } else { Some(v) });
                                    }
                                ></textarea>
                            </FormField>
                            <div class="flex justify-end gap-3 border-t border-white/[0.04] pt-4">
                                <button
                                    type="button"
                                    class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                    on:click=move |_| set_show_add_barang.set(false)
                                >
                                    "Batal"
                                </button>
                                <button
                                    type="submit"
                                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                    disabled=move || submitting.get()
                                >
                                    {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>

            // Return to operator modal
            <Show when=move || show_return_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="mx-4 w-full max-w-lg rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                        <h3 class="mb-4 text-lg font-bold text-slate-100">"Kembalikan ke Operator"</h3>
                        <form on:submit=handle_return_to_operator class="flex flex-col gap-4">
                            <FormField label="Catatan / Alasan Pengembalian" required=true>
                                <textarea
                                    rows="3"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Jelaskan alasan pengembalian..."
                                    on:input=move |ev| set_return_catatan.set(event_target_value(&ev))
                                    prop:value=move || return_catatan.get()
                                ></textarea>
                            </FormField>
                            <div class="flex justify-end gap-3 border-t border-white/[0.04] pt-4">
                                <button
                                    type="button"
                                    class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                    on:click=move |_| set_show_return_modal.set(false)
                                >"Batal"</button>
                                <button
                                    type="submit"
                                    class="rounded-lg bg-warning-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-warning-700 disabled:opacity-50"
                                    disabled=move || action_loading.get()
                                >"Kembalikan"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>

            // Reject modal
            <Show when=move || show_reject_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="mx-4 w-full max-w-lg rounded-2xl border border-white/[0.06] bg-surface-panel p-6 shadow-xl">
                        <h3 class="mb-4 text-lg font-bold text-slate-100">"Tolak Pengajuan"</h3>
                        <form on:submit=handle_reject class="flex flex-col gap-4">
                            <FormField label="Alasan Penolakan" required=true>
                                <textarea
                                    rows="3"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Jelaskan alasan penolakan..."
                                    on:input=move |ev| set_reject_alasan.set(event_target_value(&ev))
                                    prop:value=move || reject_alasan.get()
                                ></textarea>
                            </FormField>
                            <div class="flex justify-end gap-3 border-t border-white/[0.04] pt-4">
                                <button
                                    type="button"
                                    class="rounded-lg border border-white/10 bg-white/[0.04] px-4 py-2 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                    on:click=move |_| set_show_reject_modal.set(false)
                                >"Batal"</button>
                                <button
                                    type="submit"
                                    class="rounded-lg bg-danger-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-danger-700 disabled:opacity-50"
                                    disabled=move || action_loading.get()
                                >"Tolak"</button>
                            </div>
                        </form>
                    </div>
                </div>
            </Show>
        </PageLayout>
    }
}
