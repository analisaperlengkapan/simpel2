//! Kebutuhan BMN Form Component
//!
//! Form for creating and editing BMN needs analysis requests.

use crate::api::{
    CreateAssetTypeRequest, CreateKebutuhanBmnRequest, PilihanSatker, UpdateKebutuhanBmnRequest,
    create_kebutuhan_bmn, fetch_kebutuhan_bmn_detail, update_kebutuhan_bmn,
};
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
                        set_error.set(Some(format!("Gagal memuat data: {:?}", e)));
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
                    // Redirect after short delay
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
                    set_error.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
            set_submitting.set(false);
        });
    };

    let current_year = 2025;
    let years: Vec<i32> = (2020..=current_year + 2).rev().collect();
    let years = StoredValue::new(years);

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            // Header
            <div class="mb-6">
                <a
                    href=routes::path::KEBUTUHAN_DAFTAR
                    class="text-blue-600 hover:text-blue-800 inline-flex items-center mb-4"
                >
                    <i class="fas fa-arrow-left mr-2"></i>
                    "Kembali ke Daftar"
                </a>
                <h2 class="text-xl font-bold text-gray-800">
                    {move || match mode.get() {
                        FormMode::Create => "Buat Pengajuan Kebutuhan BMN",
                        FormMode::Edit(_) => "Edit Pengajuan Kebutuhan BMN",
                    }}
                </h2>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <div class="text-center py-12">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-3"></div>
                    <p class="text-gray-500">"Memuat data..."</p>
                </div>
            </Show>

            // Success message
            <Show when=move || success.get()>
                <div class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-6 flex items-center">
                    <i class="fas fa-check-circle mr-2"></i>
                    <span>"Data berhasil disimpan! Mengalihkan..."</span>
                </div>
            </Show>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6 flex items-center">
                    <i class="fas fa-exclamation-circle mr-2"></i>
                    <span>{move || error.get().unwrap_or_default()}</span>
                </div>
            </Show>

            // Form
            <Show when=move || !loading.get()>
                <form on:submit=on_submit class="space-y-6">
                    // Nama pengajuan
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Nama Pengajuan " <span class="text-red-500">"*"</span>
                        </label>
                        <input
                            type="text"
                            required
                            minlength="3"
                            maxlength="255"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder="Contoh: Pengajuan Kebutuhan BMN Tahun 2025"
                            on:input=move |ev| set_nama.set(event_target_value(&ev))
                            prop:value=move || nama.get()
                        />
                    </div>

                    // Deskripsi
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">
                            "Deskripsi"
                        </label>
                        <textarea
                            rows="3"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            placeholder="Deskripsi pengajuan (opsional)"
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                set_deskripsi.set(if v.is_empty() { None } else { Some(v) });
                            }
                            prop:value=move || deskripsi.get().unwrap_or_default()
                        ></textarea>
                    </div>

                    // Tahun & Periode
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Tahun Anggaran " <span class="text-red-500">"*"</span>
                            </label>
                            <select
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 bg-white"
                                on:change=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse() {
                                        set_tahun.set(v);
                                    }
                                }
                            >
                                <For
                                    each=move || years.get_value()
                                    key=|y| *y
                                    children=move |y| {
                                        view! {
                                            <option
                                                value=y.to_string()
                                                selected=move || tahun.get() == y
                                            >
                                                {y}
                                            </option>
                                        }
                                    }
                                />
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Tanggal Mulai " <span class="text-red-500">"*"</span>
                            </label>
                            <input
                                type="date"
                                required
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                on:input=move |ev| set_tgl_mulai.set(event_target_value(&ev))
                                prop:value=move || tgl_mulai.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Tanggal Selesai " <span class="text-red-500">"*"</span>
                            </label>
                            <input
                                type="date"
                                required
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                on:input=move |ev| set_tgl_selesai.set(event_target_value(&ev))
                                prop:value=move || tgl_selesai.get()
                            />
                        </div>
                    </div>

                    // Pilihan Satker
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-3">
                            "Pilihan Satker " <span class="text-red-500">"*"</span>
                        </label>
                        <div class="flex gap-6">
                            <label class="flex items-center cursor-pointer">
                                <input
                                    type="radio"
                                    name="pilihan_satker"
                                    class="w-4 h-4 text-blue-600"
                                    checked=move || pilihan_satker.get() == PilihanSatker::Semua
                                    on:change=move |_| set_pilihan_satker.set(PilihanSatker::Semua)
                                />
                                <span class="ml-2">"Semua Satker"</span>
                            </label>
                            <label class="flex items-center cursor-pointer">
                                <input
                                    type="radio"
                                    name="pilihan_satker"
                                    class="w-4 h-4 text-blue-600"
                                    checked=move || pilihan_satker.get() == PilihanSatker::Sebagian
                                    on:change=move |_| set_pilihan_satker.set(PilihanSatker::Sebagian)
                                />
                                <span class="ml-2">"Sebagian Satker"</span>
                            </label>
                        </div>
                    </div>

                    // Action buttons
                    <div class="flex justify-end gap-3 pt-6 border-t">
                        <a
                            href=routes::path::KEBUTUHAN_DAFTAR
                            class="px-6 py-2 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-50 transition-colors"
                        >
                            "Batal"
                        </a>
                        <button
                            type="submit"
                            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed inline-flex items-center"
                            disabled=move || submitting.get() || !is_valid.get()
                        >
                            <Show when=move || submitting.get()>
                                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2"></div>
                            </Show>
                            {move || if submitting.get() { "Menyimpan..." } else { "Simpan" }}
                        </button>
                    </div>
                </form>
            </Show>
        </div>
    }
}
