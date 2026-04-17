//! Kebutuhan BMN Form Component
//!
//! Form for creating and editing BMN needs analysis requests.

use crate::api::{
    CreateAssetTypeRequest, CreateKebutuhanBmnRequest, PilihanSatker, UpdateKebutuhanBmnRequest,
    create_kebutuhan_bmn, fetch_kebutuhan_bmn_detail, update_kebutuhan_bmn,
};
use crate::components::layout::{FormField, LoadingState, PageLayout, SectionCard};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;

#[derive(Clone, PartialEq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(String),
}

#[component]
pub fn KebutuhanBmnForm() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id"));

    let mode = Memo::new(move |_| match id.get() {
        Some(id_val) if !id_val.is_empty() && id_val != "baru" => FormMode::Edit(id_val),
        _ => FormMode::Create,
    });

    // Form state
    let (nama, set_nama) = signal(String::new());
    let (deskripsi, set_deskripsi) = signal::<Option<String>>(None);
    let (tahun, set_tahun) = signal(2025i32);
    let (tgl_mulai, set_tgl_mulai) = signal(String::new());
    let (tgl_selesai, set_tgl_selesai) = signal(String::new());
    let (pilihan_satker, set_pilihan_satker) = signal(PilihanSatker::Semua);
    let (satker_ids, set_satker_ids) = signal::<Vec<String>>(vec![]);
    let (version, set_version) = signal(0i32);

    // UI state
    let (loading, set_loading) = signal(false);
    let (submitting, set_submitting) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (success, set_success) = signal(false);

    // Load existing data if editing
    let _load_effect = Effect::new(move || {
        if let FormMode::Edit(edit_id) = mode.get() {
            set_loading.set(true);
            spawn_local(async move {
                match fetch_kebutuhan_bmn_detail(&edit_id).await {
                    Ok(response) => {
                        let p = response.data.pengajuan;
                        set_nama.set(p.nama);
                        set_deskripsi.set(p.deskripsi);
                        set_tahun.set(p.tahun);
                        set_tgl_mulai.set(p.tgl_mulai);
                        set_tgl_selesai.set(p.tgl_selesai);
                        set_pilihan_satker.set(p.pilihan_satker);
                        set_version.set(p.version);
                        let ids: Vec<String> = response
                            .data
                            .satkers
                            .iter()
                            .map(|s| s.ms_satker_id.clone())
                            .collect();
                        set_satker_ids.set(ids);
                    }
                    Err(e) => {
                        set_error.set(Some(e.user_message()));
                    }
                }
                set_loading.set(false);
            });
        }
    });

    // Form validation
    let is_valid = Memo::new(move |_| {
        let n = nama.get();
        let tm = tgl_mulai.get();
        let ts = tgl_selesai.get();
        !n.is_empty() && n.len() >= 3 && !tm.is_empty() && !ts.is_empty()
    });

    // Submit handler
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if !is_valid.get() {
            set_error.set(Some(
                "Mohon lengkapi semua field yang wajib diisi".to_string(),
            ));
            return;
        }

        set_submitting.set(true);
        set_error.set(None);

        let current_mode = mode.get();
        let nama_val = nama.get();
        let deskripsi_val = deskripsi.get();
        let tahun_val = tahun.get();
        let tgl_mulai_val = tgl_mulai.get();
        let tgl_selesai_val = tgl_selesai.get();
        let pilihan_satker_val = pilihan_satker.get();
        let satker_ids_val = satker_ids.get();
        let version_val = version.get();

        spawn_local(async move {
            let result = match current_mode {
                FormMode::Create => {
                    let request = CreateKebutuhanBmnRequest {
                        nama: nama_val,
                        deskripsi: deskripsi_val,
                        tahun: tahun_val,
                        tgl_mulai: tgl_mulai_val,
                        tgl_selesai: tgl_selesai_val,
                        pilihan_satker: Some(match pilihan_satker_val {
                            PilihanSatker::Semua => "semua".to_string(),
                            PilihanSatker::Sebagian => "sebagian".to_string(),
                        }),
                        satker_ids: satker_ids_val,
                        asset_types: vec![],
                    };
                    create_kebutuhan_bmn(request).await
                }
                FormMode::Edit(edit_id) => {
                    let request = UpdateKebutuhanBmnRequest {
                        nama: Some(nama_val),
                        deskripsi: deskripsi_val,
                        tgl_mulai: Some(tgl_mulai_val),
                        tgl_selesai: Some(tgl_selesai_val),
                        pilihan_satker: Some(match pilihan_satker_val {
                            PilihanSatker::Semua => "semua".to_string(),
                            PilihanSatker::Sebagian => "sebagian".to_string(),
                        }),
                        version: version_val,
                    };
                    update_kebutuhan_bmn(&edit_id, request).await
                }
            };

            match result {
                Ok(_) => {
                    set_success.set(true);
                    gloo_timers::callback::Timeout::new(1500, || {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(routes::path::KEBUTUHAN_DAFTAR_LEGACY);
                        }
                    })
                    .forget();
                }
                Err(e) => {
                    set_error.set(Some(e.user_message()));
                }
            }
            set_submitting.set(false);
        });
    };

    let current_year = 2026;
    let years: Vec<i32> = (2020..=current_year + 2).rev().collect();
    let years = StoredValue::new(years);

    let page_title = match mode.get_untracked() {
        FormMode::Create => "Buat Pengajuan Kebutuhan BMN",
        FormMode::Edit(_) => "Edit Pengajuan Kebutuhan BMN",
    };

    view! {
        <PageLayout
            title=page_title
            icon="fas fa-clipboard-list"
            description="Formulir pengajuan analisis kebutuhan barang milik negara"
        >
            // Back link
            <a
                href=routes::path::KEBUTUHAN_DAFTAR
                class="mb-4 inline-flex items-center gap-2 text-sm text-gold-400 transition hover:text-gold-300"
            >
                <i class="fas fa-arrow-left text-xs"></i>
                "Kembali ke Daftar"
            </a>

            // Loading state
            <Show when=move || loading.get()>
                <LoadingState message="Memuat data pengajuan...".to_string() />
            </Show>

            // Success message
            <Show when=move || success.get()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300">
                    <i class="fas fa-check-circle"></i>
                    "Data berhasil disimpan! Mengalihkan..."
                </div>
            </Show>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <i class="fas fa-exclamation-circle"></i>
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Form
            <Show when=move || !loading.get()>
                <SectionCard title="Detail Pengajuan">
                    <form on:submit=on_submit class="flex flex-col gap-5">
                        // Nama pengajuan
                        <FormField label="Nama Pengajuan" required=true full_width=true>
                            <input
                                type="text"
                                required
                                minlength="3"
                                maxlength="255"
                                class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                placeholder="Contoh: Pengajuan Kebutuhan BMN Tahun 2025"
                                on:input=move |ev| set_nama.set(event_target_value(&ev))
                                prop:value=move || nama.get()
                            />
                        </FormField>

                        // Deskripsi
                        <FormField label="Deskripsi" full_width=true>
                            <textarea
                                rows="3"
                                class="focus-ring min-h-[80px] w-full resize-y rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                placeholder="Deskripsi pengajuan (opsional)"
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    set_deskripsi.set(if v.is_empty() { None } else { Some(v) });
                                }
                                prop:value=move || deskripsi.get().unwrap_or_default()
                            ></textarea>
                        </FormField>

                        // Tahun & Periode
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-3">
                            <FormField label="Tahun Anggaran" required=true>
                                <select
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                                    on:change=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            set_tahun.set(v);
                                        }
                                    }
                                >
                                    {move || years.get_value().into_iter().map(|y| {
                                        view! {
                                            <option
                                                value=y.to_string()
                                                selected=move || tahun.get() == y
                                            >
                                                {y}
                                            </option>
                                        }
                                    }).collect_view()}
                                </select>
                            </FormField>
                            <FormField label="Tanggal Mulai" required=true>
                                <input
                                    type="date"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                                    on:input=move |ev| set_tgl_mulai.set(event_target_value(&ev))
                                    prop:value=move || tgl_mulai.get()
                                />
                            </FormField>
                            <FormField label="Tanggal Selesai" required=true>
                                <input
                                    type="date"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                                    on:input=move |ev| set_tgl_selesai.set(event_target_value(&ev))
                                    prop:value=move || tgl_selesai.get()
                                />
                            </FormField>
                        </div>

                        // Pilihan Satker
                        <FormField label="Pilihan Satker" required=true full_width=true>
                            <div class="flex gap-6">
                                <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                                    <input
                                        type="radio"
                                        name="pilihan_satker"
                                        class="h-4 w-4 border-white/20 bg-white/[0.04] text-gold-500 focus:ring-gold-500/30"
                                        checked=move || pilihan_satker.get() == PilihanSatker::Semua
                                        on:change=move |_| set_pilihan_satker.set(PilihanSatker::Semua)
                                    />
                                    "Semua Satker"
                                </label>
                                <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                                    <input
                                        type="radio"
                                        name="pilihan_satker"
                                        class="h-4 w-4 border-white/20 bg-white/[0.04] text-gold-500 focus:ring-gold-500/30"
                                        checked=move || pilihan_satker.get() == PilihanSatker::Sebagian
                                        on:change=move |_| set_pilihan_satker.set(PilihanSatker::Sebagian)
                                    />
                                    "Sebagian Satker"
                                </label>
                            </div>
                        </FormField>

                        // Action buttons
                        <div class="flex justify-end gap-3 border-t border-white/[0.04] pt-5">
                            <a
                                href=routes::path::KEBUTUHAN_DAFTAR
                                class="rounded-lg border border-white/10 bg-white/[0.04] px-5 py-2.5 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                            >
                                "Batal"
                            </a>
                            <button
                                type="submit"
                                class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                disabled=move || submitting.get() || !is_valid.get()
                            >
                                <Show when=move || submitting.get()>
                                    <span class="relative flex h-4 w-4 items-center justify-center">
                                        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-navy-950/40"></span>
                                        <span class="relative inline-flex h-2.5 w-2.5 rounded-full bg-navy-950"></span>
                                    </span>
                                </Show>
                                {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                        </div>
                    </form>
                </SectionCard>
            </Show>
        </PageLayout>
    }
}
