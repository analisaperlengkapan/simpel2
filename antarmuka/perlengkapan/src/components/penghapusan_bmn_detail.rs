//! Usulan SK Penghapusan BMN Detail Component
//!
//! Displays full detail of an Usulan SK Penghapusan BMN workflow item with action buttons
//! based on the current workflow status.

use leptos::prelude::*;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::{
    ApiResponse, PenghapusanBmnDetailResponse, PenghapusanBmnLampiran, PenghapusanBmnWorkflow,
    PenghapusanValidatorWilayahActionRequest, SimanAssetVerification, UploadSignedSKRequest,
    fetch_penghapusan_bmn_detail, fetch_penghapusan_lampiran, fetch_penghapusan_verifikasi_siman,
    generate_penghapusan_konsep_sk, penghapusan_validator_wilayah_action,
    submit_penghapusan_to_wilayah, upload_penghapusan_lampiran, upload_penghapusan_signed_sk,
};
use crate::components::workflow_ui::{
    ActionTone, ApprovalDialog, StepStatus, WorkflowAction, WorkflowActions, WorkflowStep,
    WorkflowTimeline,
};
use crate::routes;
use lib_ui::components::forms::FileUpload;

/// leptos-fetch query — keyed by `(usulan_id, refresh_trigger)`.
///
/// `refresh_trigger` is folded into the cache key so post-mutation
/// code (submit, forward, return, generate SK, upload signed SK)
/// can force a real network re-fetch by bumping the trigger.
/// Returns `Result` so the existing render branches (Loading /
/// Error / Detail) keep their three-arm shape.
async fn query_penghapusan_bmn_detail(
    key: (String, i32),
) -> Result<PenghapusanBmnDetailResponse, crate::api::AppError> {
    let (id, _trigger) = key;
    if id.is_empty() {
        return Err(crate::api::AppError::Unknown(
            "ID tidak ditemukan".to_string(),
        ));
    }
    fetch_penghapusan_bmn_detail(&id)
        .await
        .map(|r| r.data)
        .map_err(|e| crate::api::AppError::Unknown(format!("{:?}", e)))
}

/// leptos-fetch query — verifikasi aset SIMAN (Fase 2.3). Keyed by
/// `(usulan_id, refresh_trigger)` agar ikut menyegar setelah mutasi.
async fn query_penghapusan_verifikasi_siman(key: (String, i32)) -> Option<SimanAssetVerification> {
    let (id, _trigger) = key;
    if id.is_empty() {
        return None;
    }
    fetch_penghapusan_verifikasi_siman(&id)
        .await
        .ok()
        .map(|r| r.data)
}

/// leptos-fetch query — daftar lampiran pendukung (Fase 0.6 / #15). Keyed by
/// `(usulan_id, refresh_trigger)` agar ikut menyegar setelah unggah.
async fn query_penghapusan_lampiran(
    key: (String, i32),
) -> Result<ApiResponse<Vec<PenghapusanBmnLampiran>>, crate::api::AppError> {
    let (id, _trigger) = key;
    fetch_penghapusan_lampiran(&id).await
}

/// Status badge color helper
fn status_badge_class(status_kode: i32) -> &'static str {
    match status_kode {
        4000 => "bg-gray-100 text-gray-800",     // Draft
        4001 => "bg-blue-100 text-blue-800",     // SubmitWilayah
        4002 => "bg-yellow-100 text-yellow-800", // ReturnedToOperator
        4003 => "bg-indigo-100 text-indigo-800", // SubmitPusat
        4004 => "bg-purple-100 text-purple-800", // VerifikasiPusat
        4005 => "bg-cyan-100 text-cyan-800",     // KonsepSKGenerated
        4006 => "bg-teal-100 text-teal-800",     // SKSigned
        4007 => "bg-green-100 text-green-800",   // Completed
        4008 => "bg-red-100 text-red-800",       // Rejected
        _ => "bg-gray-100 text-gray-800",
    }
}

fn status_label(status_kode: i32) -> &'static str {
    match status_kode {
        4000 => "Draft",
        4001 => "Diajukan ke Validator Wilayah",
        4002 => "Dikembalikan ke Operator",
        4003 => "Diajukan ke Validator Pusat",
        4004 => "Verifikasi Pusat",
        4005 => "Konsep SK Digenerate",
        4006 => "SK Ditandatangani",
        4007 => "Selesai",
        4008 => "Ditolak",
        _ => "Tidak Diketahui",
    }
}

/// Rakit langkah-langkah timeline dari milestone yang benar-benar terjadi
/// (berbasis timestamp), lalu satu langkah penutup sesuai status terkini.
/// Pendekatan berbasis timestamp ini aman terhadap percabangan kewenangan
/// PUSAT vs WILAYAH: tahap yang dilewati tidak punya timestamp → tidak muncul.
fn build_penghapusan_timeline(p: &PenghapusanBmnWorkflow) -> Vec<WorkflowStep> {
    let mut steps: Vec<WorkflowStep> = Vec::new();

    steps.push(
        WorkflowStep::new("Usulan Dibuat", StepStatus::Done)
            .with_timestamp(p.created_at.clone())
            .with_note(p.catatan_operator.clone()),
    );
    let mut milestone = |label: &str, ts: &Option<String>, note: Option<String>| {
        if let Some(ts) = ts {
            steps.push(
                WorkflowStep::new(label, StepStatus::Done)
                    .with_timestamp(ts.clone())
                    .with_note(note),
            );
        }
    };
    milestone(
        "Diajukan ke Validator Wilayah",
        &p.tanggal_submit_wilayah,
        None,
    );
    milestone(
        "Ditinjau Validator Wilayah",
        &p.tanggal_verifikasi_wilayah,
        p.catatan_validator_wilayah.clone(),
    );
    milestone("Diajukan ke Validator Pusat", &p.tanggal_submit_pusat, None);
    milestone(
        "Ditinjau Validator Pusat",
        &p.tanggal_verifikasi_pusat,
        p.catatan_validator_pusat.clone(),
    );
    milestone("Konsep SK Digenerate", &p.konsep_sk_generated_at, None);
    milestone(
        "SK Ditandatangani Diunggah",
        &p.signed_sk_pdf_uploaded_at,
        None,
    );

    if p.is_completed {
        steps.push(WorkflowStep::new("Selesai", StepStatus::Done));
    } else if p.status_kode == 4008 {
        steps.push(
            WorkflowStep::new("Pengajuan Ditolak", StepStatus::Rejected).with_note(
                p.catatan_validator_pusat
                    .clone()
                    .or_else(|| p.catatan_validator_wilayah.clone()),
            ),
        );
    } else {
        steps.push(WorkflowStep::new(
            status_label(p.status_kode),
            StepStatus::Current,
        ));
    }
    steps
}

#[component]
pub fn PenghapusanBmnDetail() -> impl IntoView {
    let params = use_params_map();
    let (error_msg, set_error_msg) = signal(Option::<String>::None);
    let (success_msg, set_success_msg) = signal(Option::<String>::None);
    let (loading_action, set_loading_action) = signal(false);
    let (catatan_input, set_catatan_input) = signal(String::new());
    let (signed_sk_url, set_signed_sk_url) = signal(String::new());
    let (show_return_modal, set_show_return_modal) = signal(false);
    let (show_upload_modal, set_show_upload_modal) = signal(false);
    // #15: lampiran upload in-flight flag.
    let uploading = RwSignal::new(false);

    // Bumped after every successful mutation so the keyer below
    // emits a fresh cache slot and leptos-fetch issues a real
    // network request instead of serving stale data. Calling
    // `.refetch()` on a leptos-fetch resource with an unchanged
    // key just returns the stale cached value.
    let refresh_trigger = RwSignal::new(0i32);

    let client: QueryClient = expect_context();
    let detail_resource = client.local_resource(query_penghapusan_bmn_detail, move || {
        (
            params.get().get("id").unwrap_or_default(),
            refresh_trigger.get(),
        )
    });
    let lampiran_resource = client.local_resource(query_penghapusan_lampiran, move || {
        (
            params.get().get("id").unwrap_or_default(),
            refresh_trigger.get(),
        )
    });
    let verifikasi_resource =
        client.local_resource(query_penghapusan_verifikasi_siman, move || {
            (
                params.get().get("id").unwrap_or_default(),
                refresh_trigger.get(),
            )
        });

    // Action: Submit to Validator Wilayah (Draft → SubmitWilayah)
    let on_submit_wilayah = Callback::new(move |_: ()| {
        let id = params.get().get("id").unwrap_or_default();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match submit_penghapusan_to_wilayah(&id).await {
                Ok(_) => {
                    set_success_msg.set(Some("Berhasil diajukan ke Validator Wilayah".to_string()));
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    });

    // Action: Validator Wilayah forwards to Pusat
    let on_forward_pusat = Callback::new(move |_: ()| {
        let id = params.get().get("id").unwrap_or_default();
        let catatan = catatan_input.get();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let req = PenghapusanValidatorWilayahActionRequest {
                aksi: "forward".to_string(),
                catatan: if catatan.is_empty() {
                    None
                } else {
                    Some(catatan)
                },
            };
            match penghapusan_validator_wilayah_action(&id, req).await {
                Ok(_) => {
                    set_success_msg.set(Some("Berhasil diteruskan ke Validator Pusat".to_string()));
                    set_catatan_input.set(String::new());
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    });

    // Action: Validator Wilayah returns to Operator.
    // Catatan dipasok oleh ApprovalDialog (require_note) lewat callback.
    let on_return_operator = Callback::new(move |catatan: String| {
        let id = params.get().get("id").unwrap_or_default();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let req = PenghapusanValidatorWilayahActionRequest {
                aksi: "return".to_string(),
                catatan: Some(catatan),
            };
            match penghapusan_validator_wilayah_action(&id, req).await {
                Ok(_) => {
                    set_success_msg
                        .set(Some("Berhasil dikembalikan ke Operator Satker".to_string()));
                    set_show_return_modal.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    });

    // Action: Generate Konsep SK (Validator Pusat)
    let on_generate_sk = Callback::new(move |_: ()| {
        let id = params.get().get("id").unwrap_or_default();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match generate_penghapusan_konsep_sk(&id).await {
                Ok(_) => {
                    set_success_msg.set(Some("Konsep SK berhasil digenerate".to_string()));
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    });

    // Action: Upload Signed SK PDF
    let on_upload_signed_sk = move |_| {
        let id = params.get().get("id").unwrap_or_default();
        let url = signed_sk_url.get();
        if url.is_empty() {
            set_error_msg.set(Some(
                "URL file SK yang ditandatangani wajib diisi".to_string(),
            ));
            return;
        }
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            let req = UploadSignedSKRequest {
                signed_sk_pdf_url: url,
            };
            match upload_penghapusan_signed_sk(&id, req).await {
                Ok(_) => {
                    set_success_msg.set(Some(
                        "Usulan SK Penghapusan BMN berhasil diupload. Proses selesai.".to_string(),
                    ));
                    set_signed_sk_url.set(String::new());
                    set_show_upload_modal.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            // Messages
            {move || {
                error_msg
                    .get()
                    .map(|msg| {
                        view! {
                            <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
                                <p class="font-medium">{msg}</p>
                            </div>
                        }
                    })
            }}
            {move || {
                success_msg
                    .get()
                    .map(|msg| {
                        view! {
                            <div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg">
                                <p class="font-medium">{msg}</p>
                            </div>
                        }
                    })
            }}
            <Suspense fallback=move || {
                view! {
                    <div class="flex items-center justify-center py-12">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                        <span class="ml-3 text-gray-600">"Memuat detail..."</span>
                    </div>
                }
            }>
                {move || {
                    detail_resource
                        .get()
                        .map(|result| match result {
                            Err(e) => {
                                view! {
                                    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
                                        <p>{format!("Error: {}", e)}</p>
                                    </div>
                                }
                                    .into_any()
                            }
                            Ok(detail) => {
                                let d = detail.clone();
                                let status_kode = d.penghapusan.status_kode;

                                view! {
                                    <div class="space-y-6">
                                        // Header
                                        <div class="flex items-center justify-between">
                                            <div>
                                                <h2 class="text-2xl font-bold text-gray-900">
                                                    "Usulan SK Penghapusan BMN"
                                                </h2>
                                                <p class="text-gray-500">
                                                    {format!(
                                                        "{} - {}",
                                                        d.penghapusan.nama_barang,
                                                        d.penghapusan.nup,
                                                    )}
                                                </p>
                                            </div>
                                            <span class=format!(
                                                "px-3 py-1 rounded-full text-sm font-medium {}",
                                                status_badge_class(status_kode),
                                            )>{status_label(status_kode)}</span>
                                        </div>

                                        // Info Cards
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                            // Informasi BMN
                                            <div class="bg-white rounded-lg shadow p-4">
                                                <h3 class="font-semibold text-gray-700 mb-3">
                                                    "Informasi BMN"
                                                </h3>
                                                <dl class="space-y-2 text-sm">
                                                    <div>
                                                        <dt class="text-gray-500">"Kode Barang"</dt>
                                                        <dd class="font-medium">
                                                            {d.penghapusan.kode_barang.clone()}
                                                        </dd>
                                                    </div>
                                                    <div>
                                                        <dt class="text-gray-500">"Nama Barang"</dt>
                                                        <dd class="font-medium">
                                                            {d.penghapusan.nama_barang.clone()}
                                                        </dd>
                                                    </div>
                                                    <div>
                                                        <dt class="text-gray-500">"NUP"</dt>
                                                        <dd class="font-medium">{d.penghapusan.nup.clone()}</dd>
                                                    </div>
                                                    <div>
                                                        <dt class="text-gray-500">"Nilai Perolehan"</dt>
                                                        <dd class="font-medium">
                                                            {d
                                                                .penghapusan
                                                                .nilai_perolehan
                                                                .map(|v| format!("Rp {:.2}", v))
                                                                .unwrap_or_else(|| "-".to_string())}
                                                        </dd>
                                                        {d
                                                            .penghapusan
                                                            .nilai_perolehan_dari_backfill
                                                            .then(|| {
                                                                view! {
                                                                    <p class="mt-1 text-xs text-amber-600">
                                                                        "Diisi otomatis dari data lama (nilai residu) — perlu diverifikasi."
                                                                    </p>
                                                                }
                                                            })}
                                                    </div>
                                                </dl>
                                            </div>

                                            // Detail Penghapusan
                                            <div class="bg-white rounded-lg shadow p-4">
                                                <h3 class="font-semibold text-gray-700 mb-3">
                                                    "Detail Penghapusan"
                                                </h3>
                                                <dl class="space-y-2 text-sm">
                                                    <div>
                                                        <dt class="text-gray-500">"Tanggal Penghapusan"</dt>
                                                        <dd class="font-medium">
                                                            {d.penghapusan.tanggal_penghapusan.clone()}
                                                        </dd>
                                                    </div>
                                                    <div>
                                                        <dt class="text-gray-500">"Metode"</dt>
                                                        <dd class="font-medium">
                                                            {d.penghapusan.metode_penghapusan.clone()}
                                                        </dd>
                                                    </div>
                                                    <div>
                                                        <dt class="text-gray-500">"Alasan"</dt>
                                                        <dd class="font-medium">{d.penghapusan.alasan.clone()}</dd>
                                                    </div>
                                                </dl>
                                            </div>

                                            // Lampiran & Catatan
                                            <div class="bg-white rounded-lg shadow p-4">
                                                <h3 class="font-semibold text-gray-700 mb-3">
                                                    "Lampiran & Catatan"
                                                </h3>
                                                <dl class="space-y-2 text-sm">
                                                    {d
                                                        .penghapusan
                                                        .lampiran_persyaratan
                                                        .clone()
                                                        .map(|url| {
                                                            view! {
                                                                <div>
                                                                    <dt class="text-gray-500">"Lampiran Persyaratan"</dt>
                                                                    <dd>
                                                                        <a
                                                                            href=url.clone()
                                                                            target="_blank"
                                                                            class="text-blue-600 hover:underline"
                                                                        >
                                                                            "Lihat Dokumen"
                                                                        </a>
                                                                    </dd>
                                                                </div>
                                                            }
                                                        })}
                                                    {d
                                                        .penghapusan
                                                        .catatan_operator
                                                        .clone()
                                                        .map(|c| {
                                                            view! {
                                                                <div>
                                                                    <dt class="text-gray-500">"Catatan Operator"</dt>
                                                                    <dd>{c}</dd>
                                                                </div>
                                                            }
                                                        })}
                                                    {d
                                                        .penghapusan
                                                        .catatan_validator_wilayah
                                                        .clone()
                                                        .map(|c| {
                                                            view! {
                                                                <div>
                                                                    <dt class="text-gray-500">"Catatan Validator Wilayah"</dt>
                                                                    <dd>{c}</dd>
                                                                </div>
                                                            }
                                                        })}
                                                    {d
                                                        .penghapusan
                                                        .catatan_validator_pusat
                                                        .clone()
                                                        .map(|c| {
                                                            view! {
                                                                <div>
                                                                    <dt class="text-gray-500">"Catatan Validator Pusat"</dt>
                                                                    <dd>{c}</dd>
                                                                </div>
                                                            }
                                                        })}
                                                </dl>
                                            </div>
                                        </div>

                                        // Daftar Item BMN (Fase 2.8) — multi-item
                                        {(!d.items.is_empty())
                                            .then(|| {
                                                let items = d.items.clone();
                                                view! {
                                                    <div class="bg-white rounded-lg shadow p-4">
                                                        <h3 class="font-semibold text-gray-700 mb-3">
                                                            "Daftar Item BMN (" {items.len()} ")"
                                                        </h3>
                                                        <div class="overflow-x-auto">
                                                            <table class="min-w-full text-sm">
                                                                <thead class="bg-gray-50 text-left text-xs text-gray-500">
                                                                    <tr>
                                                                        <th class="px-3 py-2">"No"</th>
                                                                        <th class="px-3 py-2">"Kode Barang"</th>
                                                                        <th class="px-3 py-2">"Nama Barang"</th>
                                                                        <th class="px-3 py-2">"NUP"</th>
                                                                        <th class="px-3 py-2">"Kondisi"</th>
                                                                        <th class="px-3 py-2 text-right">"Nilai Perolehan"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody class="divide-y">
                                                                    {items
                                                                        .into_iter()
                                                                        .enumerate()
                                                                        .map(|(i, it)| {
                                                                            view! {
                                                                                <tr>
                                                                                    <td class="px-3 py-2">{i + 1}</td>
                                                                                    <td class="px-3 py-2 font-mono">{it.kode_barang}</td>
                                                                                    <td class="px-3 py-2">{it.nama_barang}</td>
                                                                                    <td class="px-3 py-2 font-mono">{it.nup}</td>
                                                                                    <td class="px-3 py-2">
                                                                                        {it.kondisi.unwrap_or_else(|| "-".to_string())}
                                                                                    </td>
                                                                                    <td class="px-3 py-2 text-right">
                                                                                        {it
                                                                                            .nilai_perolehan
                                                                                            .map(|v| format!("Rp {:.0}", v))
                                                                                            .unwrap_or_else(|| "-".to_string())}
                                                                                    </td>
                                                                                </tr>
                                                                            }
                                                                        })
                                                                        .collect_view()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    </div>
                                                }
                                            })}

                                        // Lampiran Pendukung (Fase 0.6 / #15) — daftar + unggah
                                        {
                                            let can_upload = status_kode == 4000 || status_kode == 4002;
                                            let upload_surat = Box::new(move |
                                                files: Vec<web_sys::File>|
                                            {
                                                let Some(file) = files.into_iter().next() else { return };
                                                let id = params.get().get("id").unwrap_or_default();
                                                uploading.set(true);
                                                set_error_msg.set(None);
                                                leptos::task::spawn_local(async move {
                                                    match upload_penghapusan_lampiran(&id, Some(file), vec![])
                                                        .await
                                                    {
                                                        Ok(_) => {
                                                            set_success_msg
                                                                .set(Some("Surat Usulan berhasil diunggah".to_string()));
                                                            refresh_trigger.update(|v| *v += 1);
                                                        }
                                                        Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
                                                    }
                                                    uploading.set(false);
                                                });
                                            }) as Box<dyn Fn(Vec<web_sys::File>)>;
                                            let upload_lampiran = Box::new(move |
                                                files: Vec<web_sys::File>|
                                            {
                                                if files.is_empty() {
                                                    return;
                                                }
                                                let id = params.get().get("id").unwrap_or_default();
                                                uploading.set(true);
                                                set_error_msg.set(None);
                                                leptos::task::spawn_local(async move {
                                                    match upload_penghapusan_lampiran(&id, None, files).await {
                                                        Ok(_) => {
                                                            set_success_msg
                                                                .set(Some("Lampiran berhasil diunggah".to_string()));
                                                            refresh_trigger.update(|v| *v += 1);
                                                        }
                                                        Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
                                                    }
                                                    uploading.set(false);
                                                });
                                            }) as Box<dyn Fn(Vec<web_sys::File>)>;
                                            view! {
                                                <div class="bg-white rounded-lg shadow p-4">
                                                    <h3 class="font-semibold text-gray-700 mb-3">
                                                        "Lampiran Pendukung"
                                                    </h3>
                                                    <Suspense fallback=move || {
                                                        view! {
                                                            <p class="text-sm text-gray-400">"Memuat lampiran..."</p>
                                                        }
                                                    }>
                                                        {move || match lampiran_resource.get() {
                                                            None => {
                                                                view! {
                                                                    <p class="text-sm text-gray-400">"Memuat lampiran..."</p>
                                                                }
                                                                    .into_any()
                                                            }
                                                            Some(Err(_)) => {
                                                                view! {
                                                                    <p class="text-sm text-gray-400">
                                                                        "Gagal memuat daftar lampiran."
                                                                    </p>
                                                                }
                                                                    .into_any()
                                                            }
                                                            Some(Ok(resp)) => {
                                                                let items = resp.data;
                                                                if items.is_empty() {
                                                                    view! {
                                                                        <p class="text-sm text-gray-400">
                                                                            "Belum ada lampiran diunggah."
                                                                        </p>
                                                                    }
                                                                        .into_any()
                                                                } else {
                                                                    view! {
                                                                        <ul class="space-y-1.5 text-sm">
                                                                            {items
                                                                                .into_iter()
                                                                                .map(|l| {
                                                                                    view! {
                                                                                        <li class="flex items-center justify-between gap-2">
                                                                                            <a
                                                                                                href=l.file_url.clone()
                                                                                                target="_blank"
                                                                                                class="text-blue-600 hover:underline truncate"
                                                                                            >
                                                                                                {l.nama.clone()}
                                                                                            </a>
                                                                                            <span class="text-xs text-gray-400 shrink-0">
                                                                                                {l
                                                                                                    .size_bytes
                                                                                                    .map(|b| format!("{} KB", (b + 1023) / 1024))
                                                                                                    .unwrap_or_default()}
                                                                                            </span>
                                                                                        </li>
                                                                                    }
                                                                                })
                                                                                .collect_view()}
                                                                        </ul>
                                                                    }
                                                                        .into_any()
                                                                }
                                                            }
                                                        }}
                                                    </Suspense>

                                                    {can_upload
                                                        .then(move || {
                                                            view! {
                                                                <div class="mt-4 space-y-3 border-t pt-4">
                                                                    <FileUpload
                                                                        label="Surat Usulan (1 file)".to_string()
                                                                        accept=".pdf,.doc,.docx".to_string()
                                                                        disabled=uploading.get()
                                                                        on_change=upload_surat
                                                                    />
                                                                    <FileUpload
                                                                        label="Lampiran Pendukung (boleh lebih dari satu)"
                                                                            .to_string()
                                                                        accept=".pdf,.doc,.docx,.jpg,.jpeg,.png".to_string()
                                                                        multiple=true
                                                                        disabled=uploading.get()
                                                                        on_change=upload_lampiran
                                                                    />
                                                                    {move || {
                                                                        uploading
                                                                            .get()
                                                                            .then(|| {
                                                                                view! {
                                                                                    <p class="text-xs text-gray-500">"Mengunggah berkas..."</p>
                                                                                }
                                                                            })
                                                                    }}
                                                                </div>
                                                            }
                                                        })}
                                                </div>
                                            }
                                        }

                                        // Verifikasi Aset SIMAN (Fase 2.3) — tampil di tahap validator
                                        {(status_kode == 4001 || status_kode == 4003
                                            || status_kode == 4004)
                                            .then(|| {
                                                view! {
                                                    <div class="bg-white rounded-lg shadow p-4">
                                                        <h3 class="font-semibold text-gray-700 mb-3">
                                                            "Verifikasi Aset di SIMAN"
                                                        </h3>
                                                        <Suspense fallback=move || {
                                                            view! {
                                                                <p class="text-sm text-gray-400">
                                                                    "Memeriksa aset di SIMAN..."
                                                                </p>
                                                            }
                                                        }>
                                                            {move || {
                                                                verifikasi_resource
                                                                    .get()
                                                                    .flatten()
                                                                    .map(|v| {
                                                                        let (badge_class, badge_text) = if !v.ditemukan {
                                                                            ("bg-red-100 text-red-800", "Tidak Ditemukan")
                                                                        } else if v.layak_lanjut {
                                                                            ("bg-green-100 text-green-800", "Terverifikasi")
                                                                        } else {
                                                                            ("bg-yellow-100 text-yellow-800", "Perlu Pengecekan")
                                                                        };
                                                                        view! {
                                                                            <div>
                                                                                <div class="flex items-center gap-2 mb-2">
                                                                                    <span class=format!(
                                                                                        "px-2 py-0.5 rounded text-xs font-medium {}",
                                                                                        badge_class,
                                                                                    )>{badge_text}</span>
                                                                                    <span class="text-sm text-gray-600">{v.pesan.clone()}</span>
                                                                                </div>
                                                                                {v
                                                                                    .ditemukan
                                                                                    .then(|| {
                                                                                        view! {
                                                                                            <dl class="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                                                                                                <dt class="text-gray-500">"NUP"</dt>
                                                                                                <dd class="font-medium">{v.nup.clone()}</dd>
                                                                                                <dt class="text-gray-500">"Nama (SIMAN)"</dt>
                                                                                                <dd class="font-medium">
                                                                                                    {v
                                                                                                        .nama_barang_siman
                                                                                                        .clone()
                                                                                                        .unwrap_or_else(|| "-".to_string())}
                                                                                                </dd>
                                                                                                <dt class="text-gray-500">"Kondisi"</dt>
                                                                                                <dd class="font-medium">
                                                                                                    {v.kondisi.clone().unwrap_or_else(|| "-".to_string())}
                                                                                                </dd>
                                                                                                <dt class="text-gray-500">"Kode Barang (SIMAN)"</dt>
                                                                                                <dd class=if v.kode_barang_cocok {
                                                                                                    "font-medium"
                                                                                                } else {
                                                                                                    "font-medium text-red-600"
                                                                                                }>
                                                                                                    {v
                                                                                                        .kode_barang_siman
                                                                                                        .clone()
                                                                                                        .unwrap_or_else(|| "-".to_string())}
                                                                                                </dd>
                                                                                            </dl>
                                                                                        }
                                                                                    })}
                                                                            </div>
                                                                        }
                                                                            .into_any()
                                                                    })
                                                                    .unwrap_or_else(|| {
                                                                        view! {
                                                                            <p class="text-sm text-gray-400">
                                                                                "Data verifikasi tidak tersedia"
                                                                            </p>
                                                                        }
                                                                            .into_any()
                                                                    })
                                                            }}
                                                        </Suspense>
                                                    </div>
                                                }
                                            })}

                                        // Riwayat Proses (Fase 2.5 — WorkflowTimeline reusable)
                                        <div class="bg-white rounded-lg shadow p-4">
                                            <h3 class="font-semibold text-gray-700 mb-3">
                                                "Riwayat Proses"
                                            </h3>
                                            <WorkflowTimeline steps=build_penghapusan_timeline(
                                                &d.penghapusan,
                                            ) />
                                        </div>

                                        // SK Document Section — konsep is produced as both DOCX
                                        // (editable) and PDF (final) side-by-side, signed PDF is
                                        // the post-signature upload.
                                        {(d.penghapusan.konsep_sk_url.is_some()
                                            || d.penghapusan.konsep_sk_pdf_url.is_some()
                                            || d.penghapusan.signed_sk_pdf_url.is_some())
                                            .then(|| {
                                                let konsep_docx = d.penghapusan.konsep_sk_url.clone();
                                                let konsep_pdf = d.penghapusan.konsep_sk_pdf_url.clone();
                                                let signed = d.penghapusan.signed_sk_pdf_url.clone();
                                                view! {
                                                    <div class="bg-white rounded-lg shadow p-4">
                                                        <h3 class="font-semibold text-gray-700 mb-3">
                                                            "Dokumen SK"
                                                        </h3>
                                                        <div class="flex flex-wrap gap-4">
                                                            {konsep_docx
                                                                .map(|url| {
                                                                    view! {
                                                                        <a
                                                                            href=url
                                                                            target="_blank"
                                                                            class="inline-flex items-center px-4 py-2 bg-cyan-50 text-cyan-700 rounded-lg hover:bg-cyan-100"
                                                                        >
                                                                            <svg
                                                                                class="w-4 h-4 mr-2"
                                                                                fill="none"
                                                                                stroke="currentColor"
                                                                                viewBox="0 0 24 24"
                                                                            >
                                                                                <path
                                                                                    stroke-linecap="round"
                                                                                    stroke-linejoin="round"
                                                                                    stroke-width="2"
                                                                                    d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
                                                                                />
                                                                            </svg>
                                                                            "Konsep SK (DOCX)"
                                                                        </a>
                                                                    }
                                                                })}
                                                            {konsep_pdf
                                                                .map(|url| {
                                                                    view! {
                                                                        <a
                                                                            href=url
                                                                            target="_blank"
                                                                            class="inline-flex items-center px-4 py-2 bg-rose-50 text-rose-700 rounded-lg hover:bg-rose-100"
                                                                        >
                                                                            <svg
                                                                                class="w-4 h-4 mr-2"
                                                                                fill="none"
                                                                                stroke="currentColor"
                                                                                viewBox="0 0 24 24"
                                                                            >
                                                                                <path
                                                                                    stroke-linecap="round"
                                                                                    stroke-linejoin="round"
                                                                                    stroke-width="2"
                                                                                    d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"
                                                                                />
                                                                            </svg>
                                                                            "Konsep SK (PDF)"
                                                                        </a>
                                                                    }
                                                                })}
                                                            {signed
                                                                .map(|url| {
                                                                    view! {
                                                                        <a
                                                                            href=url
                                                                            target="_blank"
                                                                            class="inline-flex items-center px-4 py-2 bg-green-50 text-green-700 rounded-lg hover:bg-green-100"
                                                                        >
                                                                            <svg
                                                                                class="w-4 h-4 mr-2"
                                                                                fill="none"
                                                                                stroke="currentColor"
                                                                                viewBox="0 0 24 24"
                                                                            >
                                                                                <path
                                                                                    stroke-linecap="round"
                                                                                    stroke-linejoin="round"
                                                                                    stroke-width="2"
                                                                                    d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                                                                />
                                                                            </svg>
                                                                            "SK Ditandatangani (PDF)"
                                                                        </a>
                                                                    }
                                                                })}
                                                        </div>
                                                    </div>
                                                }
                                            })}

                                        // Action Buttons based on status
                                        <div class="bg-white rounded-lg shadow p-4">
                                            <h3 class="font-semibold text-gray-700 mb-3">"Aksi"</h3>

                                            // Catatan input for validator actions
                                            {(status_kode == 4001 || status_kode == 4003)
                                                .then(|| {
                                                    view! {
                                                        <div class="mb-4">
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Catatan"
                                                            </label>
                                                            <textarea
                                                                class="w-full border rounded-lg px-3 py-2 text-sm"
                                                                rows="2"
                                                                placeholder="Tambahkan catatan (opsional untuk meneruskan, wajib untuk mengembalikan)"
                                                                prop:value=move || catatan_input.get()
                                                                on:input=move |ev| {
                                                                    set_catatan_input.set(event_target_value(&ev))
                                                                }
                                                            />
                                                        </div>
                                                    }
                                                })}

                                            // Tombol aksi — WorkflowActions reusable (Fase 2.5).
                                            // Vec dirakit kondisional sesuai status & kapabilitas.
                                            {
                                                let mut actions: Vec<WorkflowAction> = Vec::new();
                                                if status_kode == 4000 || status_kode == 4002 {
                                                    actions
                                                        .push(
                                                            WorkflowAction::new(
                                                                "Ajukan ke Validator Wilayah",
                                                                ActionTone::Primary,
                                                                on_submit_wilayah,
                                                            ),
                                                        );
                                                }
                                                if status_kode == 4001 {
                                                    actions
                                                        .push(
                                                            WorkflowAction::new(
                                                                "Teruskan ke Validator Pusat",
                                                                ActionTone::Primary,
                                                                on_forward_pusat,
                                                            ),
                                                        );
                                                    actions
                                                        .push(
                                                            WorkflowAction::new(
                                                                "Kembalikan ke Operator",
                                                                ActionTone::Warning,
                                                                Callback::new(move |_| set_show_return_modal.set(true)),
                                                            ),
                                                        );
                                                }
                                                if detail.can_generate_sk {
                                                    actions
                                                        .push(
                                                            WorkflowAction::new(
                                                                "Generate Konsep SK",
                                                                ActionTone::Primary,
                                                                on_generate_sk,
                                                            ),
                                                        );
                                                }
                                                if detail.can_upload_signed_sk {
                                                    actions
                                                        .push(
                                                            WorkflowAction::new(
                                                                "Upload SK Ditandatangani",
                                                                ActionTone::Success,
                                                                Callback::new(move |_| set_show_upload_modal.set(true)),
                                                            ),
                                                        );
                                                }
                                                view! {
                                                    <WorkflowActions
                                                        actions=actions
                                                        busy=Signal::derive(move || loading_action.get())
                                                    />
                                                }
                                            }

                                            // Terminal status displays + back link
                                            <div class="mt-3 flex flex-wrap items-center gap-3">
                                                // Completed status
                                                {(status_kode == 4007)
                                                    .then(|| {
                                                        view! {
                                                            <div class="flex items-center text-green-600">
                                                                <svg
                                                                    class="w-5 h-5 mr-2"
                                                                    fill="none"
                                                                    stroke="currentColor"
                                                                    viewBox="0 0 24 24"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2"
                                                                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                                                    />
                                                                </svg>
                                                                <span class="font-medium">
                                                                    "Proses Usulan SK Penghapusan BMN Telah Selesai"
                                                                </span>
                                                            </div>
                                                        }
                                                    })} // Rejected status
                                                {(status_kode == 4008)
                                                    .then(|| {
                                                        view! {
                                                            <div class="flex items-center text-red-600">
                                                                <svg
                                                                    class="w-5 h-5 mr-2"
                                                                    fill="none"
                                                                    stroke="currentColor"
                                                                    viewBox="0 0 24 24"
                                                                >
                                                                    <path
                                                                        stroke-linecap="round"
                                                                        stroke-linejoin="round"
                                                                        stroke-width="2"
                                                                        d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                                                                    />
                                                                </svg>
                                                                <span class="font-medium">"Pengajuan Ditolak"</span>
                                                            </div>
                                                        }
                                                    })} // Back button
                                                <a
                                                    href=routes::path::PENGELOLAAN_PENGHAPUSAN_DAFTAR_LEGACY
                                                    class="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50"
                                                >
                                                    "Kembali ke Daftar"
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            // Return to Operator — ApprovalDialog reusable (Fase 2.5)
            </Suspense>
            {move || {
                show_return_modal
                    .get()
                    .then(|| {
                        view! {
                            <ApprovalDialog
                                title="Kembalikan ke Operator Satker"
                                description="Jelaskan alasan pengembalian agar Operator Satker dapat memperbaiki usulan."
                                note_label="Catatan (wajib)"
                                note_placeholder="Jelaskan alasan pengembalian..."
                                require_note=true
                                confirm_label="Kembalikan"
                                confirm_tone=ActionTone::Warning
                                on_confirm=on_return_operator
                                on_close=Callback::new(move |_| set_show_return_modal.set(false))
                                busy=Signal::derive(move || loading_action.get())
                            />
                        }
                    })
            }} // Upload Signed SK Modal
            {move || {
                show_upload_modal
                    .get()
                    .then(|| {
                        view! {
                            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-modal">
                                <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
                                    <h3 class="text-lg font-semibold mb-4">
                                        "Upload SK Ditandatangani"
                                    </h3>
                                    <div class="mb-4">
                                        <label class="block text-sm font-medium text-gray-700 mb-1">
                                            "URL File SK (PDF)"
                                        </label>
                                        <input
                                            type="text"
                                            class="w-full border rounded-lg px-3 py-2 text-sm"
                                            placeholder="https://storage.example.com/sk-penghapusan.pdf"
                                            prop:value=move || signed_sk_url.get()
                                            on:input=move |ev| {
                                                set_signed_sk_url.set(event_target_value(&ev))
                                            }
                                        />
                                        <p class="text-xs text-gray-500 mt-1">
                                            "Upload file PDF SK yang telah ditandatangani ke storage, lalu masukkan URL-nya."
                                        </p>
                                    </div>
                                    <div class="flex gap-3 justify-end">
                                        <button
                                            class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
                                            on:click=move |_| set_show_upload_modal.set(false)
                                        >
                                            "Batal"
                                        </button>
                                        <button
                                            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                                            disabled=move || {
                                                loading_action.get() || signed_sk_url.get().is_empty()
                                            }
                                            on:click=on_upload_signed_sk
                                        >
                                            "Upload & Selesaikan"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    })
            }}
        </div>
    }
}
