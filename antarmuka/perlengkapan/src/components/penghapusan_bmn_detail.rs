//! SK Penghapusan BMN Detail Component
//!
//! Displays full detail of a Penghapusan BMN workflow item with action buttons
//! based on the current workflow status.

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::api::{
    fetch_penghapusan_bmn_detail, generate_penghapusan_konsep_sk,
    penghapusan_validator_wilayah_action, submit_penghapusan_to_wilayah,
    upload_penghapusan_signed_sk, PenghapusanBmnDetailResponse,
    PenghapusanValidatorWilayahActionRequest, UploadSignedSKRequest,
};

/// Status badge color helper
fn status_badge_class(status_kode: i32) -> &'static str {
    match status_kode {
        4000 => "bg-gray-100 text-gray-800",       // Draft
        4001 => "bg-blue-100 text-blue-800",        // SubmitWilayah
        4002 => "bg-yellow-100 text-yellow-800",    // ReturnedToOperator
        4003 => "bg-indigo-100 text-indigo-800",    // SubmitPusat
        4004 => "bg-purple-100 text-purple-800",    // VerifikasiPusat
        4005 => "bg-cyan-100 text-cyan-800",        // KonsepSKGenerated
        4006 => "bg-teal-100 text-teal-800",        // SKSigned
        4007 => "bg-green-100 text-green-800",      // Completed
        4008 => "bg-red-100 text-red-800",          // Rejected
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

    let detail_resource = Resource::new(
        move || params.get().get("id").unwrap_or_default(),
        |id| async move {
            if id.is_empty() {
                return Err("ID tidak ditemukan".to_string());
            }
            fetch_penghapusan_bmn_detail(&id)
                .await
                .map(|r| r.data)
                .map_err(|e| format!("{:?}", e))
        },
    );

    // Action: Submit to Validator Wilayah (Draft → SubmitWilayah)
    let on_submit_wilayah = move |_| {
        let id = params.get().get("id").unwrap_or_default();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match submit_penghapusan_to_wilayah(&id).await {
                Ok(_) => {
                    set_success_msg.set(Some("Berhasil diajukan ke Validator Wilayah".to_string()));
                    detail_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    // Action: Validator Wilayah forwards to Pusat
    let on_forward_pusat = move |_| {
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
                    set_success_msg
                        .set(Some("Berhasil diteruskan ke Validator Pusat".to_string()));
                    set_catatan_input.set(String::new());
                    detail_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    // Action: Validator Wilayah returns to Operator
    let on_return_operator = move |_| {
        let id = params.get().get("id").unwrap_or_default();
        let catatan = catatan_input.get();
        if catatan.is_empty() {
            set_error_msg.set(Some("Catatan wajib diisi saat mengembalikan".to_string()));
            return;
        }
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
                    set_catatan_input.set(String::new());
                    set_show_return_modal.set(false);
                    detail_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    // Action: Generate Konsep SK (Validator Pusat)
    let on_generate_sk = move |_| {
        let id = params.get().get("id").unwrap_or_default();
        set_loading_action.set(true);
        set_error_msg.set(None);
        leptos::task::spawn_local(async move {
            match generate_penghapusan_konsep_sk(&id).await {
                Ok(_) => {
                    set_success_msg.set(Some("Konsep SK berhasil digenerate".to_string()));
                    detail_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    // Action: Upload Signed SK PDF
    let on_upload_signed_sk = move |_| {
        let id = params.get().get("id").unwrap_or_default();
        let url = signed_sk_url.get();
        if url.is_empty() {
            set_error_msg.set(Some("URL file SK yang ditandatangani wajib diisi".to_string()));
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
                    set_success_msg
                        .set(Some("SK Penghapusan BMN berhasil diupload. Proses selesai.".to_string()));
                    set_signed_sk_url.set(String::new());
                    set_show_upload_modal.set(false);
                    detail_resource.refetch();
                }
                Err(e) => set_error_msg.set(Some(format!("{:?}", e))),
            }
            set_loading_action.set(false);
        });
    };

    view! {
        <div class="space-y-6">
            // Messages
            {move || error_msg.get().map(|msg| view! {
                <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
                    <p class="font-medium">{msg}</p>
                </div>
            })}
            {move || success_msg.get().map(|msg| view! {
                <div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg">
                    <p class="font-medium">{msg}</p>
                </div>
            })}

            <Suspense fallback=move || view! {
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    <span class="ml-3 text-gray-600">"Memuat detail..."</span>
                </div>
            }>
                {move || detail_resource.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
                            <p>{format!("Error: {}", e)}</p>
                        </div>
                    }.into_any(),
                    Ok(detail) => {
                        let d = detail.clone();
                        let status_kode = d.penghapusan.status_kode;

                        view! {
                            <div class="space-y-6">
                                // Header
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h2 class="text-2xl font-bold text-gray-900">
                                            "SK Penghapusan BMN"
                                        </h2>
                                        <p class="text-gray-500">
                                            {format!("{} - {}", d.penghapusan.nama_barang, d.penghapusan.nup)}
                                        </p>
                                    </div>
                                    <span class={format!("px-3 py-1 rounded-full text-sm font-medium {}", status_badge_class(status_kode))}>
                                        {status_label(status_kode)}
                                    </span>
                                </div>

                                // Info Cards
                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    // Informasi BMN
                                    <div class="bg-white rounded-lg shadow p-4">
                                        <h3 class="font-semibold text-gray-700 mb-3">"Informasi BMN"</h3>
                                        <dl class="space-y-2 text-sm">
                                            <div>
                                                <dt class="text-gray-500">"Kode Barang"</dt>
                                                <dd class="font-medium">{d.penghapusan.kode_barang.clone()}</dd>
                                            </div>
                                            <div>
                                                <dt class="text-gray-500">"Nama Barang"</dt>
                                                <dd class="font-medium">{d.penghapusan.nama_barang.clone()}</dd>
                                            </div>
                                            <div>
                                                <dt class="text-gray-500">"NUP"</dt>
                                                <dd class="font-medium">{d.penghapusan.nup.clone()}</dd>
                                            </div>
                                            <div>
                                                <dt class="text-gray-500">"Nilai Residu"</dt>
                                                <dd class="font-medium">
                                                    {d.penghapusan.nilai_residu.map(|v| format!("Rp {:.2}", v)).unwrap_or_else(|| "-".to_string())}
                                                </dd>
                                            </div>
                                        </dl>
                                    </div>

                                    // Detail Penghapusan
                                    <div class="bg-white rounded-lg shadow p-4">
                                        <h3 class="font-semibold text-gray-700 mb-3">"Detail Penghapusan"</h3>
                                        <dl class="space-y-2 text-sm">
                                            <div>
                                                <dt class="text-gray-500">"Tanggal Penghapusan"</dt>
                                                <dd class="font-medium">{d.penghapusan.tanggal_penghapusan.clone()}</dd>
                                            </div>
                                            <div>
                                                <dt class="text-gray-500">"Metode"</dt>
                                                <dd class="font-medium">{d.penghapusan.metode_penghapusan.clone()}</dd>
                                            </div>
                                            <div>
                                                <dt class="text-gray-500">"Alasan"</dt>
                                                <dd class="font-medium">{d.penghapusan.alasan.clone()}</dd>
                                            </div>
                                        </dl>
                                    </div>

                                    // Lampiran & Catatan
                                    <div class="bg-white rounded-lg shadow p-4">
                                        <h3 class="font-semibold text-gray-700 mb-3">"Lampiran & Catatan"</h3>
                                        <dl class="space-y-2 text-sm">
                                            {d.penghapusan.lampiran_persyaratan.clone().map(|url| view! {
                                                <div>
                                                    <dt class="text-gray-500">"Lampiran Persyaratan"</dt>
                                                    <dd>
                                                        <a href={url.clone()} target="_blank" class="text-blue-600 hover:underline">
                                                            "Lihat Dokumen"
                                                        </a>
                                                    </dd>
                                                </div>
                                            })}
                                            {d.penghapusan.catatan_operator.clone().map(|c| view! {
                                                <div>
                                                    <dt class="text-gray-500">"Catatan Operator"</dt>
                                                    <dd>{c}</dd>
                                                </div>
                                            })}
                                            {d.penghapusan.catatan_validator_wilayah.clone().map(|c| view! {
                                                <div>
                                                    <dt class="text-gray-500">"Catatan Validator Wilayah"</dt>
                                                    <dd>{c}</dd>
                                                </div>
                                            })}
                                            {d.penghapusan.catatan_validator_pusat.clone().map(|c| view! {
                                                <div>
                                                    <dt class="text-gray-500">"Catatan Validator Pusat"</dt>
                                                    <dd>{c}</dd>
                                                </div>
                                            })}
                                        </dl>
                                    </div>
                                </div>

                                // SK Document Section
                                {(d.penghapusan.konsep_sk_url.is_some() || d.penghapusan.signed_sk_pdf_url.is_some()).then(|| {
                                    let konsep = d.penghapusan.konsep_sk_url.clone();
                                    let signed = d.penghapusan.signed_sk_pdf_url.clone();
                                    view! {
                                        <div class="bg-white rounded-lg shadow p-4">
                                            <h3 class="font-semibold text-gray-700 mb-3">"Dokumen SK"</h3>
                                            <div class="flex gap-4">
                                                {konsep.map(|url| view! {
                                                    <a href={url} target="_blank"
                                                       class="inline-flex items-center px-4 py-2 bg-cyan-50 text-cyan-700 rounded-lg hover:bg-cyan-100">
                                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z"/>
                                                        </svg>
                                                        "Konsep SK (DOCX)"
                                                    </a>
                                                })}
                                                {signed.map(|url| view! {
                                                    <a href={url} target="_blank"
                                                       class="inline-flex items-center px-4 py-2 bg-green-50 text-green-700 rounded-lg hover:bg-green-100">
                                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                                        </svg>
                                                        "SK Ditandatangani (PDF)"
                                                    </a>
                                                })}
                                            </div>
                                        </div>
                                    }
                                })}

                                // Action Buttons based on status
                                <div class="bg-white rounded-lg shadow p-4">
                                    <h3 class="font-semibold text-gray-700 mb-3">"Aksi"</h3>

                                    // Catatan input for validator actions
                                    {(status_kode == 4001 || status_kode == 4003).then(|| view! {
                                        <div class="mb-4">
                                            <label class="block text-sm font-medium text-gray-700 mb-1">"Catatan"</label>
                                            <textarea
                                                class="w-full border rounded-lg px-3 py-2 text-sm"
                                                rows="2"
                                                placeholder="Tambahkan catatan (opsional untuk meneruskan, wajib untuk mengembalikan)"
                                                prop:value=move || catatan_input.get()
                                                on:input=move |ev| set_catatan_input.set(event_target_value(&ev))
                                            />
                                        </div>
                                    })}

                                    <div class="flex flex-wrap gap-3">
                                        // Draft: Submit to Wilayah
                                        {(status_kode == 4000 || status_kode == 4002).then(|| view! {
                                            <button
                                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                                disabled=move || loading_action.get()
                                                on:click=on_submit_wilayah
                                            >
                                                {move || if loading_action.get() { "Mengirim..." } else { "Ajukan ke Validator Wilayah" }}
                                            </button>
                                        })}

                                        // SubmitWilayah: Forward to Pusat / Return to Operator
                                        {(status_kode == 4001).then(|| view! {
                                            <button
                                                class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 disabled:opacity-50"
                                                disabled=move || loading_action.get()
                                                on:click=on_forward_pusat
                                            >
                                                "Teruskan ke Validator Pusat"
                                            </button>
                                            <button
                                                class="px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 disabled:opacity-50"
                                                disabled=move || loading_action.get()
                                                on:click=move |_| set_show_return_modal.set(true)
                                            >
                                                "Kembalikan ke Operator"
                                            </button>
                                        })}

                                        // VerifikasiPusat: Generate SK / Reject
                                        {(detail.can_generate_sk).then(|| view! {
                                            <button
                                                class="px-4 py-2 bg-cyan-600 text-white rounded-lg hover:bg-cyan-700 disabled:opacity-50"
                                                disabled=move || loading_action.get()
                                                on:click=on_generate_sk
                                            >
                                                {move || if loading_action.get() { "Generating..." } else { "Generate Konsep SK" }}
                                            </button>
                                        })}

                                        // KonsepSKGenerated: Upload Signed SK
                                        {(detail.can_upload_signed_sk).then(|| view! {
                                            <button
                                                class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50"
                                                disabled=move || loading_action.get()
                                                on:click=move |_| set_show_upload_modal.set(true)
                                            >
                                                "Upload SK Ditandatangani"
                                            </button>
                                        })}

                                        // Completed status
                                        {(status_kode == 4007).then(|| view! {
                                            <div class="flex items-center text-green-600">
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                                </svg>
                                                <span class="font-medium">"Proses Penghapusan BMN Telah Selesai"</span>
                                            </div>
                                        })}

                                        // Rejected status
                                        {(status_kode == 4008).then(|| view! {
                                            <div class="flex items-center text-red-600">
                                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                                </svg>
                                                <span class="font-medium">"Pengajuan Ditolak"</span>
                                            </div>
                                        })}

                                        // Back button
                                        <a
                                            href="/dashboard/pengelolaan/penghapusan/daftar"
                                            class="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50"
                                        >
                                            "Kembali ke Daftar"
                                        </a>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>

            // Return to Operator Modal
            {move || show_return_modal.get().then(|| view! {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
                        <h3 class="text-lg font-semibold mb-4">"Kembalikan ke Operator Satker"</h3>
                        <div class="mb-4">
                            <label class="block text-sm font-medium text-gray-700 mb-1">"Catatan (Wajib)"</label>
                            <textarea
                                class="w-full border rounded-lg px-3 py-2 text-sm"
                                rows="3"
                                placeholder="Jelaskan alasan pengembalian..."
                                prop:value=move || catatan_input.get()
                                on:input=move |ev| set_catatan_input.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="flex gap-3 justify-end">
                            <button
                                class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
                                on:click=move |_| set_show_return_modal.set(false)
                            >
                                "Batal"
                            </button>
                            <button
                                class="px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 disabled:opacity-50"
                                disabled=move || loading_action.get() || catatan_input.get().is_empty()
                                on:click=on_return_operator
                            >
                                "Kembalikan"
                            </button>
                        </div>
                    </div>
                </div>
            })}

            // Upload Signed SK Modal
            {move || show_upload_modal.get().then(|| view! {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                    <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
                        <h3 class="text-lg font-semibold mb-4">"Upload SK Ditandatangani"</h3>
                        <div class="mb-4">
                            <label class="block text-sm font-medium text-gray-700 mb-1">"URL File SK (PDF)"</label>
                            <input
                                type="text"
                                class="w-full border rounded-lg px-3 py-2 text-sm"
                                placeholder="https://storage.example.com/sk-penghapusan.pdf"
                                prop:value=move || signed_sk_url.get()
                                on:input=move |ev| set_signed_sk_url.set(event_target_value(&ev))
                            />
                            <p class="text-xs text-gray-500 mt-1">"Upload file PDF SK yang telah ditandatangani ke storage, lalu masukkan URL-nya."</p>
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
                                disabled=move || loading_action.get() || signed_sk_url.get().is_empty()
                                on:click=on_upload_signed_sk
                            >
                                "Upload & Selesaikan"
                            </button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }
}
