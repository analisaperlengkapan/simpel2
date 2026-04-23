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
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;
use lib_ui::hooks::{use_form, use_toast::use_toast};

#[derive(Clone, PartialEq, Default)]
pub enum FormMode {
    #[default]
    Create,
    Edit(String),
}

/// Form data for BMN needs analysis request.
#[derive(Clone)]
struct KebutuhanFormData {
    nama: String,
    deskripsi: Option<String>,
    tahun: i32,
    tgl_mulai: String,
    tgl_selesai: String,
    pilihan_satker: PilihanSatker,
    satker_ids: Vec<String>,
    version: i32,
}

impl Default for KebutuhanFormData {
    fn default() -> Self {
        Self {
            nama: String::new(),
            deskripsi: None,
            tahun: 2025,
            tgl_mulai: String::new(),
            tgl_selesai: String::new(),
            pilihan_satker: PilihanSatker::Semua,
            satker_ids: vec![],
            version: 0,
        }
    }
}

#[component]
pub fn KebutuhanBmnForm() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| params.read().get("id"));

    let mode = Memo::new(move |_| match id.get() {
        Some(id_val) if !id_val.is_empty() && id_val != "baru" => FormMode::Edit(id_val),
        _ => FormMode::Create,
    });

    let form = use_form(KebutuhanFormData::default());
    let toast = use_toast();
    let (loading, set_loading) = signal(false);

    // Load existing data if editing
    let _load_effect = Effect::new(move || {
        if let FormMode::Edit(edit_id) = mode.get() {
            set_loading.set(true);
            spawn_local(async move {
                match fetch_kebutuhan_bmn_detail(&edit_id).await {
                    Ok(response) => {
                        let p = response.data.pengajuan;
                        let ids: Vec<String> = response
                            .data
                            .satkers
                            .iter()
                            .map(|s| s.ms_satker_id.clone())
                            .collect();
                        form.set(KebutuhanFormData {
                            nama: p.nama,
                            deskripsi: p.deskripsi,
                            tahun: p.tahun,
                            tgl_mulai: p.tgl_mulai,
                            tgl_selesai: p.tgl_selesai,
                            pilihan_satker: p.pilihan_satker,
                            satker_ids: ids,
                            version: p.version,
                        });
                    }
                    Err(e) => {
                        form.set_error(e.user_message());
                        toast.error(e.user_message());
                    }
                }
                set_loading.set(false);
            });
        }
    });

    // Form validation
    let is_valid = Memo::new(move |_| {
        let d = form.get();
        !d.nama.is_empty() && d.nama.len() >= 3 && !d.tgl_mulai.is_empty() && !d.tgl_selesai.is_empty()
    });

    // Submit handler
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        if !is_valid.get() {
            form.set_error("Mohon lengkapi semua field yang wajib diisi");
            return;
        }

        let data = form.begin_submit();
        let current_mode = mode.get();

        spawn_local(async move {
            let result = match current_mode {
                FormMode::Create => {
                    let request = CreateKebutuhanBmnRequest {
                        nama: data.nama,
                        deskripsi: data.deskripsi,
                        tahun: data.tahun,
                        tgl_mulai: data.tgl_mulai,
                        tgl_selesai: data.tgl_selesai,
                        pilihan_satker: Some(match data.pilihan_satker {
                            PilihanSatker::Semua => "semua".to_string(),
                            PilihanSatker::Sebagian => "sebagian".to_string(),
                        }),
                        satker_ids: data.satker_ids,
                        asset_types: vec![],
                    };
                    create_kebutuhan_bmn(request).await
                }
                FormMode::Edit(edit_id) => {
                    let request = UpdateKebutuhanBmnRequest {
                        nama: Some(data.nama),
                        deskripsi: data.deskripsi,
                        tgl_mulai: Some(data.tgl_mulai),
                        tgl_selesai: Some(data.tgl_selesai),
                        pilihan_satker: Some(match data.pilihan_satker {
                            PilihanSatker::Semua => "semua".to_string(),
                            PilihanSatker::Sebagian => "sebagian".to_string(),
                        }),
                        version: data.version,
                    };
                    update_kebutuhan_bmn(&edit_id, request).await
                }
            };

            match result {
                Ok(_) => {
                    form.finish_ok();
                    toast.success("Data berhasil disimpan!");
                    let nav = leptos_router::hooks::use_navigate();
                    gloo_timers::callback::Timeout::new(1500, move || {
                        nav(routes::path::KEBUTUHAN_DAFTAR, Default::default());
                    })
                    .forget();
                }
                Err(e) => {
                    form.finish_err(e.user_message());
                    toast.error(e.user_message());
                }
            }
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
        <Title text=format!("{} — SIMPEL", page_title) />
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
                                on:input=move |ev| form.update(|f| f.nama = event_target_value(&ev))
                                prop:value=move || form.get().nama.clone()
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
                                    form.update(|f| f.deskripsi = if v.is_empty() { None } else { Some(v) });
                                }
                                prop:value=move || form.get().deskripsi.clone().unwrap_or_default()
                            ></textarea>
                        </FormField>

                        // Tahun & Periode
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-3">
                            <FormField label="Tahun Anggaran" required=true>
                                <select
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                                    on:change=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            form.update(|f| f.tahun = v);
                                        }
                                    }
                                >
                                    {move || years.get_value().into_iter().map(|y| {
                                        view! {
                                            <option
                                                value=y.to_string()
                                                selected=move || form.get().tahun == y
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
                                    on:input=move |ev| form.update(|f| f.tgl_mulai = event_target_value(&ev))
                                    prop:value=move || form.get().tgl_mulai.clone()
                                />
                            </FormField>
                            <FormField label="Tanggal Selesai" required=true>
                                <input
                                    type="date"
                                    required
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-200"
                                    on:input=move |ev| form.update(|f| f.tgl_selesai = event_target_value(&ev))
                                    prop:value=move || form.get().tgl_selesai.clone()
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
                                        checked=move || form.get().pilihan_satker == PilihanSatker::Semua
                                        on:change=move |_| form.update(|f| f.pilihan_satker = PilihanSatker::Semua)
                                    />
                                    "Semua Satker"
                                </label>
                                <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                                    <input
                                        type="radio"
                                        name="pilihan_satker"
                                        class="h-4 w-4 border-white/20 bg-white/[0.04] text-gold-500 focus:ring-gold-500/30"
                                        checked=move || form.get().pilihan_satker == PilihanSatker::Sebagian
                                        on:change=move |_| form.update(|f| f.pilihan_satker = PilihanSatker::Sebagian)
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
                                disabled=move || form.submitting.get() || !is_valid.get()
                            >
                                <Show when=move || form.submitting.get()>
                                    <span class="relative flex h-4 w-4 items-center justify-center">
                                        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-navy-950/40"></span>
                                        <span class="relative inline-flex h-2.5 w-2.5 rounded-full bg-navy-950"></span>
                                    </span>
                                </Show>
                                {move || if form.submitting.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                        </div>
                    </form>
                </SectionCard>
            </Show>
        </PageLayout>
    }
}
