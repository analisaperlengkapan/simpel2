//! Penghapusan BMN detail — review, konsep SK, unggah SK tertandatangan.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_LEFT, CLOCK, FILE_ARROW_UP, FILE_TEXT, MAGIC_WAND, WARNING, X};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};

use crate::api::common::{
    PenghapusanBmnDetailResponse, PenghapusanBmnWorkflow, UploadSignedSKRequest,
};
use crate::api::error::AppError;
use crate::api::penghapusan_bmn;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::path;

#[component]
pub fn PenghapusanBmnDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id_signal = Signal::derive(move || params.with(|p| p.get("id").unwrap_or_default()));

    let (detail, set_detail) = signal::<Option<PenghapusanBmnDetailResponse>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);
    let (action_msg, set_action_msg) = signal::<Option<(bool, String)>>(None);
    let (show_upload, set_show_upload) = signal(false);
    let (sk_url, set_sk_url) = signal::<String>(String::new());
    let (submitting, set_submitting) = signal(false);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let id = id_signal.get();
        if id.is_empty() {
            set_loading.set(false);
            set_error.set(Some(AppError::not_found(
                "Id usulan penghapusan tidak diberikan.",
            )));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match penghapusan_bmn::fetch_penghapusan_bmn_detail(&id).await {
                Ok(resp) => set_detail.set(Some(resp.data)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let reload = move || set_reload_tick.update(|t| *t += 1);

    let open_upload = move |_| {
        set_sk_url.set(String::new());
        set_show_upload.set(true);
    };
    let close_upload = move |_| set_show_upload.set(false);

    let submit_upload = move |_| {
        let id = id_signal.get();
        let url = sk_url.get();
        if url.trim().is_empty() {
            set_action_msg.set(Some((
                false,
                "URL SK tertandatangan wajib diisi.".to_string(),
            )));
            return;
        }
        set_submitting.set(true);
        spawn_local(async move {
            match penghapusan_bmn::upload_penghapusan_signed_sk(
                &id,
                UploadSignedSKRequest {
                    signed_sk_pdf_url: url,
                },
            )
            .await
            {
                Ok(_) => {
                    set_action_msg.set(Some((
                        true,
                        "SK tertandatangan berhasil diunggah.".to_string(),
                    )));
                    set_show_upload.set(false);
                    set_reload_tick.update(|t| *t += 1);
                }
                Err(e) => set_action_msg.set(Some((false, e.user_message()))),
            }
            set_submitting.set(false);
        });
    };

    let generate_sk = move |_| {
        let id = id_signal.get();
        set_submitting.set(true);
        spawn_local(async move {
            match penghapusan_bmn::generate_penghapusan_konsep_sk(&id).await {
                Ok(_) => {
                    set_action_msg.set(Some((true, "Konsep SK berhasil digenerate.".to_string())));
                    set_reload_tick.update(|t| *t += 1);
                }
                Err(e) => set_action_msg.set(Some((false, e.user_message()))),
            }
            set_submitting.set(false);
        });
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Penghapusan BMN", path::PENGELOLAAN_PENGHAPUSAN),
        PageBreadcrumb::leaf("Detail"),
    ];

    view! {
        <PageLayout
            title="Detail Usulan Penghapusan"
            description="Pantau workflow, konsep SK, dan status SK tertandatangan."
            icon="fas fa-trash-can"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || view! {
                <A
                    href=path::PENGELOLAAN_PENGHAPUSAN
                    attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                >
                    <span class="text-[0.7rem]"><AppIcon icon=ARROW_LEFT /></span>
                    "Kembali"
                </A>
            }.into_any())
        >
            {move || {
                if let Some((ok, msg)) = action_msg.get() {
                    let tone = if ok {
                        "border-success-500/30 bg-success-500/10 text-success-200"
                    } else {
                        "border-danger-500/30 bg-danger-500/10 text-danger-200"
                    };
                    view! {
                        <div class=format!("rounded-lg border px-4 py-2 text-xs {}", tone)>
                            {msg}
                        </div>
                    }.into_any()
                } else {
                    view! { <div class="hidden"></div> }.into_any()
                }
            }}

            {move || {
                if loading.get() && detail.get().is_none() {
                    view! { <LoadingState message="Memuat detail usulan..." /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <ErrorState
                            error=err
                            on_retry=Box::new(move || reload())
                        />
                    }.into_any()
                } else if let Some(d) = detail.get() {
                    view! {
                        <DetailBody
                            detail=d
                            submitting=submitting
                            on_generate=generate_sk
                            on_upload=open_upload
                        />
                    }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Usulan tidak ditemukan"
                            description="Data usulan penghapusan tidak tersedia dalam basis data."
                            icon="fas fa-folder-open"
                        />
                    }.into_any()
                }
            }}

            <Show when=move || show_upload.get()>
                <UploadSignedSKModal
                    url=sk_url
                    set_url=set_sk_url
                    submitting=submitting
                    on_close=close_upload
                    on_submit=submit_upload
                />
            </Show>
        </PageLayout>
    }
}

#[component]
fn DetailBody(
    detail: PenghapusanBmnDetailResponse,
    submitting: ReadSignal<bool>,
    on_generate: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_upload: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let p = detail.penghapusan.clone();
    let status = p.status.clone();
    let can_generate_sk = detail.can_generate_sk;
    let can_upload_signed_sk = detail.can_upload_signed_sk;

    let metode = p.metode_penghapusan.clone();
    let alasan = p.alasan.clone();
    let tanggal = p.tanggal_penghapusan.clone();
    let kode_barang = p.kode_barang.clone();
    let nama_barang = p.nama_barang.clone();
    let nup = p.nup.clone();
    let nilai_perolehan = p
        .nilai_perolehan
        .map(format_rupiah)
        .unwrap_or_else(|| "-".to_string());
    let nilai_perolehan_legacy = p.nilai_perolehan_dari_backfill;
    let catatan_operator = p
        .catatan_operator
        .clone()
        .unwrap_or_else(|| "-".to_string());

    view! {
        <StatusSummary status=status.clone() />

        <SectionCard
            title="Informasi Usulan"
            description="Data penghapusan dan konteks BMN."
            icon="fas fa-circle-info"
        >
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <InfoField label="Kode Barang" value=kode_barang />
                <InfoField label="Nama BMN" value=nama_barang />
                <InfoField label="NUP" value=nup />
                <InfoField label="Tanggal Rencana" value=tanggal />
                <InfoField label="Metode" value=metode />
                <InfoField label="Nilai Perolehan" value=nilai_perolehan />
            </div>
            {nilai_perolehan_legacy.then(|| view! {
                <div class="mt-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-2">
                    <p class="text-xs text-amber-200">
                        "Nilai perolehan diisi otomatis dari data lama (nilai residu) — perlu diverifikasi."
                    </p>
                </div>
            })}
            <div class="mt-4 rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Alasan Penghapusan"</p>
                <p class="mt-1 text-sm text-slate-200">{alasan}</p>
            </div>
            <div class="mt-3 rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Catatan Operator"</p>
                <p class="mt-1 text-sm text-slate-300">{catatan_operator}</p>
            </div>
        </SectionCard>

        <ValidatorPanel p=p.clone() />

        <SkPanel
            p=p.clone()
            status=status.clone()
            can_generate_sk=can_generate_sk
            can_upload_signed_sk=can_upload_signed_sk
            submitting=submitting
            on_generate=on_generate
            on_upload=on_upload
        />
    }
}

#[component]
fn StatusSummary(status: String) -> impl IntoView {
    let (status_label, status_tone) = status_descriptor(&status);
    let stage_hint = stage_hint(&status);

    view! {
        <SectionCard
            title="Status Workflow"
            description="Fase saat ini dan petunjuk tindakan berikutnya."
            icon="fas fa-gauge-high"
        >
            <div class="flex flex-wrap items-center gap-3">
                <span class=format!(
                    "inline-flex rounded-full px-3 py-1 text-xs font-semibold ring-1 {}",
                    status_tone
                )>
                    {status_label}
                </span>
                <p class="text-xs text-slate-400">{stage_hint}</p>
            </div>
        </SectionCard>
    }
}

#[component]
fn ValidatorPanel(p: PenghapusanBmnWorkflow) -> impl IntoView {
    let wilayah_catatan = p
        .catatan_validator_wilayah
        .clone()
        .unwrap_or_else(|| "Belum ada catatan.".to_string());
    let wilayah_tgl = p
        .tanggal_verifikasi_wilayah
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let pusat_catatan = p
        .catatan_validator_pusat
        .clone()
        .unwrap_or_else(|| "Belum ada catatan.".to_string());
    let pusat_tgl = p
        .tanggal_verifikasi_pusat
        .clone()
        .unwrap_or_else(|| "-".to_string());

    view! {
        <SectionCard
            title="Review Validator"
            description="Catatan validator wilayah dan pusat."
            icon="fas fa-user-shield"
        >
            <div class="grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Validator Wilayah"</p>
                    <p class="mt-1 text-sm text-slate-100">{wilayah_catatan}</p>
                    <p class="mt-2 text-[0.65rem] text-slate-500">
                        <span class="mr-1"><AppIcon icon=CLOCK /></span>{wilayah_tgl}
                    </p>
                </div>
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Validator Pusat"</p>
                    <p class="mt-1 text-sm text-slate-100">{pusat_catatan}</p>
                    <p class="mt-2 text-[0.65rem] text-slate-500">
                        <span class="mr-1"><AppIcon icon=CLOCK /></span>{pusat_tgl}
                    </p>
                </div>
            </div>
        </SectionCard>
    }
}

#[component]
fn SkPanel(
    p: PenghapusanBmnWorkflow,
    status: String,
    can_generate_sk: bool,
    can_upload_signed_sk: bool,
    submitting: ReadSignal<bool>,
    on_generate: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_upload: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let konsep_url = p.konsep_sk_url.clone();
    let konsep_generated_at = p
        .konsep_sk_generated_at
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let signed_url = p.signed_sk_pdf_url.clone();
    let signed_uploaded_at = p
        .signed_sk_pdf_uploaded_at
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let needs_upload = status == "KONSEP_SK_GENERATED" && signed_url.is_none();

    view! {
        <SectionCard
            title="Surat Keputusan"
            description="Alur khusus penghapusan: generate konsep SK, lalu unggah SK tertandatangan."
            icon="fas fa-file-signature"
            actions=Box::new(move || view! {
                <button
                    type="button"
                    on:click=on_generate
                    disabled=move || !can_generate_sk || submitting.get()
                    class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-semibold text-slate-100 transition hover:bg-white/[0.08] disabled:opacity-40"
                >
                    <span class="text-[0.6rem]"><AppIcon icon=MAGIC_WAND /></span>
                    {move || if submitting.get() { "Memproses..." } else { "Generate Konsep SK" }}
                </button>
                <button
                    type="button"
                    on:click=on_upload
                    disabled=!can_upload_signed_sk
                    class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-40"
                >
                    <span class="text-[0.6rem]"><AppIcon icon=FILE_ARROW_UP /></span>
                    "Unggah SK Tertandatangan"
                </button>
            }.into_any())
        >
            {if needs_upload {
                Some(view! {
                    <div class="mb-3 flex items-start gap-2 rounded-lg border border-warning-500/30 bg-warning-500/10 px-3 py-2 text-xs text-warning-200">
                        <span class="mt-0.5 text-warning-300"><AppIcon icon=WARNING /></span>
                        <span>"Konsep SK sudah digenerate. Silakan cetak, tandatangani, lalu unggah SK yang telah ditandatangani untuk melanjutkan ke fase penyelesaian."</span>
                    </div>
                })
            } else {
                None
            }}

            <div class="grid gap-4 md:grid-cols-2">
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Konsep SK"</p>
                    {match konsep_url {
                        Some(url) => view! {
                            <p class="mt-1">
                                <a
                                    href=url.clone()
                                    target="_blank"
                                    class="inline-flex items-center gap-1.5 text-sm font-medium text-gold-300 hover:underline"
                                >
                                    <span class="text-[0.6rem]"><AppIcon icon=FILE_TEXT /></span>
                                    "Lihat dokumen konsep"
                                </a>
                            </p>
                            <p class="mt-2 text-[0.65rem] text-slate-500">
                                <span class="mr-1"><AppIcon icon=CLOCK /></span>
                                "Digenerate: " {konsep_generated_at.clone()}
                            </p>
                        }.into_any(),
                        None => view! {
                            <p class="mt-1 text-sm text-slate-400">"Belum digenerate."</p>
                        }.into_any(),
                    }}
                </div>
                <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
                    <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"SK Tertandatangan"</p>
                    {match signed_url {
                        Some(url) => view! {
                            <p class="mt-1">
                                <a
                                    href=url.clone()
                                    target="_blank"
                                    class="inline-flex items-center gap-1.5 text-sm font-medium text-success-300 hover:underline"
                                >
                                    <span class="text-[0.6rem]"><AppIcon icon=FILE_TEXT /></span>
                                    "Lihat dokumen tertandatangan"
                                </a>
                            </p>
                            <p class="mt-2 text-[0.65rem] text-slate-500">
                                <span class="mr-1"><AppIcon icon=CLOCK /></span>
                                "Diunggah: " {signed_uploaded_at.clone()}
                            </p>
                        }.into_any(),
                        None => view! {
                            <p class="mt-1 text-sm text-slate-400">"Belum diunggah."</p>
                        }.into_any(),
                    }}
                </div>
            </div>
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
fn UploadSignedSKModal(
    url: ReadSignal<String>,
    set_url: WriteSignal<String>,
    submitting: ReadSignal<bool>,
    on_close: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
    on_submit: impl Fn(web_sys::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let on_url = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_url.set(target.value());
        }
    };
    view! {
        <div class="fixed inset-0 z-modal flex items-center justify-center bg-black/60 backdrop-blur-sm">
            <div class="w-[95vw] max-w-md rounded-2xl border border-white/[0.06] bg-surface-panel p-5 shadow-2xl">
                <header class="flex items-start justify-between gap-3">
                    <div>
                        <h2 class="text-base font-semibold text-white">"Unggah SK Tertandatangan"</h2>
                        <p class="mt-1 text-xs text-slate-400">
                            "Masukkan URL dokumen SK (PDF) yang sudah ditandatangani. Dokumen ini akan mengunci usulan ke fase SK_SIGNED."
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
                <label class="mt-4 flex flex-col gap-1">
                    <span class="text-[0.65rem] uppercase tracking-wide text-slate-400">"URL PDF"</span>
                    <input
                        type="url"
                        placeholder="https://storage.example.com/sk/..."
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        on:input=on_url
                        prop:value=move || url.get()
                    />
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
                        <span class="text-[0.6rem]"><AppIcon icon=FILE_ARROW_UP /></span>
                        {move || if submitting.get() { "Mengunggah..." } else { "Unggah" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

fn status_descriptor(status: &str) -> (&'static str, &'static str) {
    match status {
        "DRAFT" => ("Draft", "bg-slate-500/10 text-slate-300 ring-slate-500/20"),
        "SUBMIT_WILAYAH" | "SUBMITTED" => (
            "Review Wilayah",
            "bg-info-500/10 text-info-300 ring-info-500/20",
        ),
        "RETURNED_TO_OPERATOR" => (
            "Dikembalikan ke Operator",
            "bg-warning-500/10 text-warning-300 ring-warning-500/20",
        ),
        "SUBMIT_PUSAT" | "REVIEWED" => (
            "Review Pusat",
            "bg-info-500/10 text-info-300 ring-info-500/20",
        ),
        "VERIFIKASI_PUSAT" => (
            "Verifikasi Pusat",
            "bg-info-500/10 text-info-300 ring-info-500/20",
        ),
        "KONSEP_SK_GENERATED" => (
            "Konsep SK",
            "bg-warning-500/10 text-warning-300 ring-warning-500/20",
        ),
        "SK_SIGNED" | "APPROVED" => (
            "SK Ditandatangani",
            "bg-success-500/10 text-success-300 ring-success-500/20",
        ),
        "COMPLETED" => (
            "Selesai",
            "bg-success-500/10 text-success-300 ring-success-500/20",
        ),
        "REJECTED" => (
            "Ditolak",
            "bg-danger-500/10 text-danger-300 ring-danger-500/20",
        ),
        _ => (
            "Lainnya",
            "bg-slate-500/10 text-slate-300 ring-slate-500/20",
        ),
    }
}

fn stage_hint(status: &str) -> &'static str {
    match status {
        "DRAFT" => "Lengkapi data usulan lalu ajukan ke validator wilayah.",
        "SUBMIT_WILAYAH" | "SUBMITTED" => "Menunggu verifikasi validator wilayah.",
        "RETURNED_TO_OPERATOR" => "Usulan dikembalikan. Perbaiki sesuai catatan lalu ajukan ulang.",
        "SUBMIT_PUSAT" | "REVIEWED" => "Menunggu verifikasi validator pusat.",
        "VERIFIKASI_PUSAT" => "Validator pusat sedang memverifikasi dokumen.",
        "KONSEP_SK_GENERATED" => {
            "Konsep SK sudah digenerate — cetak, tandatangani, lalu unggah SK yang telah ditandatangani."
        }
        "SK_SIGNED" | "APPROVED" => "SK tertandatangan sudah tersimpan. Lanjutkan ke penyelesaian.",
        "COMPLETED" => "Usulan selesai. Penghapusan BMN telah tercatat.",
        "REJECTED" => "Usulan ditolak. Periksa catatan validator untuk alasan penolakan.",
        _ => "Fase tidak dikenal.",
    }
}

fn format_rupiah(n: f64) -> String {
    let truncated = n.round() as i128;
    let s = truncated.abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(*b as char);
    }
    if truncated < 0 {
        format!("-Rp {out}")
    } else {
        format!("Rp {out}")
    }
}
