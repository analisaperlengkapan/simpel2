//! Period Management Page for Validator Pusat
//!
//! This page allows Validator Pusat to:
//! - Create kebutuhan BMN periods with start/end dates and deadline
//! - Select eligible BMN items (filtered by standar kodefikasi)
//! - Select eligible satkers that can submit
//! - View active periods
//! - Edit/delete periods
//!
//! Requirements: REQ-K001, REQ-K002, REQ-K003

use crate::api::{
    CreateKebutuhanBmnRequest, KebutuhanBmnQuery, KebutuhanBmnSummary, PilihanSatker,
    UpdateKebutuhanBmnRequest, create_kebutuhan_bmn, fetch_kebutuhan_bmn_list,
    update_kebutuhan_bmn,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::callback::Callback;

#[derive(Clone, PartialEq)]
enum ViewMode {
    List,
    Create,
    Edit(String),
}

#[component]
pub fn PeriodManagement() -> impl IntoView {
    let (view_mode, set_view_mode) = signal(ViewMode::List);
    let (periods, set_periods) = signal::<Vec<KebutuhanBmnSummary>>(vec![]);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Load periods on mount
    let load_periods = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let query = KebutuhanBmnQuery {
                tahun: None,
                status_kode: None,
                satker_id: None,
                search: None,
            };
            match fetch_kebutuhan_bmn_list(query, 1, 100).await {
                Ok(response) => {
                    set_periods.set(response.data);
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memuat data: {:?}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    Effect::new(move || {
        load_periods();
    });

    view! {
        <div class="mx-auto max-w-7xl space-y-6">
            // Header
            <div class="relative overflow-hidden rounded-2xl border border-white/10 bg-gradient-to-br from-navy-900 via-navy-800 to-slate-950 p-6 shadow-lg">
                <div class="pointer-events-none absolute -right-12 -top-12 h-40 w-40 rounded-full bg-gold-400/10 blur-3xl"></div>
                <div class="relative z-10">
                    <div class="flex items-center justify-between">
                        <div>
                            <div class="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/5 px-3 py-1 text-xs font-semibold text-gold-300 mb-3">
                                <i class="fas fa-calendar-alt"></i>
                                "Manajemen Periode"
                            </div>
                            <h1 class="text-2xl font-black text-white sm:text-3xl">
                                "Periode Kebutuhan BMN"
                            </h1>
                            <p class="mt-2 text-sm text-slate-300">
                                "Kelola periode pengumpulan kebutuhan BMN dari satker"
                            </p>
                        </div>
                        <Show when=move || view_mode.get() == ViewMode::List>
                            <button
                                on:click=move |_| set_view_mode.set(ViewMode::Create)
                                class="inline-flex items-center gap-2 rounded-lg bg-gold-500 px-4 py-2 text-sm font-semibold text-navy-900 transition-all hover:bg-gold-400 hover:shadow-lg"
                            >
                                <i class="fas fa-plus"></i>
                                "Buat Periode Baru"
                            </button>
                        </Show>
                    </div>
                </div>
            </div>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-red-700">
                    <div class="flex items-center gap-2">
                        <i class="fas fa-exclamation-circle"></i>
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </div>
            </Show>

            // Content based on view mode
            {move || match view_mode.get() {
                ViewMode::List => view! {
                    <PeriodList
                        periods=periods
                        loading=loading
                        on_edit=Callback::new(move |id: String| set_view_mode.set(ViewMode::Edit(id)))
                        on_refresh=Callback::new(move |_| load_periods())
                    />
                }.into_any(),
                ViewMode::Create => view! {
                    <PeriodForm
                        mode=FormMode::Create
                        on_cancel=Callback::new(move |_| set_view_mode.set(ViewMode::List))
                        on_success=Callback::new(move |_| {
                            set_view_mode.set(ViewMode::List);
                            load_periods();
                        })
                    />
                }.into_any(),
                ViewMode::Edit(id) => view! {
                    <PeriodForm
                        mode=FormMode::Edit(id)
                        on_cancel=Callback::new(move |_| set_view_mode.set(ViewMode::List))
                        on_success=Callback::new(move |_| {
                            set_view_mode.set(ViewMode::List);
                            load_periods();
                        })
                    />
                }.into_any(),
            }}
        </div>
    }
}

#[derive(Clone, PartialEq)]
enum FormMode {
    Create,
    Edit(String),
}

#[component]
fn PeriodList(
    periods: ReadSignal<Vec<KebutuhanBmnSummary>>,
    loading: ReadSignal<bool>,
    on_edit: Callback<String>,
    on_refresh: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-white/10 bg-slate-900/60 p-6 backdrop-blur">
            <div class="mb-4 flex items-center justify-between">
                <h2 class="text-lg font-bold text-slate-100">
                    "Daftar Periode"
                </h2>
                <button
                    on:click=move |_| on_refresh.run(())
                    class="rounded-lg border border-white/10 bg-slate-800/50 px-3 py-1.5 text-sm text-slate-300 transition-colors hover:bg-slate-700/50"
                >
                    <i class="fas fa-sync-alt mr-2"></i>
                    "Refresh"
                </button>
            </div>

            <Show
                when=move || loading.get()
                fallback=move || view! {
                    <Show
                        when=move || !periods.get().is_empty()
                        fallback=|| view! {
                            <div class="py-12 text-center">
                                <i class="fas fa-inbox text-4xl text-slate-600 mb-3"></i>
                                <p class="text-slate-400">"Belum ada periode yang dibuat"</p>
                            </div>
                        }
                    >
                        <div class="space-y-3">
                            <For
                                each=move || periods.get()
                                key=|p| p.id.clone()
                                children=move |period| {
                                    let edit_id = period.id.clone();
                                    view! {
                                        <PeriodCard
                                            period=period
                                            on_edit=Callback::new(move |_| on_edit.run(edit_id.clone()))
                                        />
                                    }
                                }
                            />
                        </div>
                    </Show>
                }
            >
                <div class="space-y-3">
                    <div class="h-24 animate-pulse rounded-lg bg-slate-800/50"></div>
                    <div class="h-24 animate-pulse rounded-lg bg-slate-800/50"></div>
                    <div class="h-24 animate-pulse rounded-lg bg-slate-800/50"></div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn PeriodCard(
    period: KebutuhanBmnSummary,
    on_edit: Callback<()>,
) -> impl IntoView {
    let status_badge = match period.status_kode {
        2000 => ("bg-gray-500/15 text-gray-300", "Draft"),
        2001..=2005 => ("bg-blue-500/15 text-blue-300", "Aktif"),
        2006 => ("bg-green-500/15 text-green-300", "Selesai"),
        2007 => ("bg-red-500/15 text-red-300", "Ditolak"),
        2009 => ("bg-slate-500/15 text-slate-300", "Dibatalkan"),
        _ => ("bg-slate-500/15 text-slate-300", "Unknown"),
    };

    let is_active = period.status_kode >= 2001 && period.status_kode <= 2005;

    view! {
        <div class="group relative overflow-hidden rounded-lg border border-white/10 bg-slate-800/50 p-4 transition-all hover:border-white/20 hover:bg-slate-800/80">
            <div class="flex items-start justify-between gap-4">
                <div class="min-w-0 flex-1">
                    <div class="mb-2 flex items-center gap-2">
                        <h3 class="text-base font-semibold text-white">
                            {period.nama.clone()}
                        </h3>
                        <Show when=move || is_active>
                            <span class="inline-flex items-center gap-1 rounded-full bg-emerald-500/15 px-2 py-0.5 text-xs font-medium text-emerald-300">
                                <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
                                "Aktif"
                            </span>
                        </Show>
                    </div>

                    <div class="mb-3 grid grid-cols-1 gap-2 text-sm sm:grid-cols-3">
                        <div class="flex items-center gap-2 text-slate-400">
                            <i class="fas fa-calendar text-xs"></i>
                            <span>"Tahun: " <span class="font-medium text-slate-300">{period.tahun}</span></span>
                        </div>
                        <div class="flex items-center gap-2 text-slate-400">
                            <i class="fas fa-building text-xs"></i>
                            <span>"Satker: " <span class="font-medium text-slate-300">{period.total_satker}</span></span>
                        </div>
                        <div class="flex items-center gap-2 text-slate-400">
                            <i class="fas fa-box text-xs"></i>
                            <span>"Barang: " <span class="font-medium text-slate-300">{period.total_barang}</span></span>
                        </div>
                    </div>

                    <div class="flex items-center gap-2">
                        <span class=format!("inline-flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium {}", status_badge.0)>
                            {status_badge.1}
                        </span>
                    </div>
                </div>

                <div class="flex flex-col gap-2">
                    <button
                        on:click=move |_| on_edit.run(())
                        class="rounded-lg border border-white/10 bg-slate-700/50 px-3 py-1.5 text-sm text-slate-300 transition-colors hover:bg-slate-600/50"
                    >
                        <i class="fas fa-edit mr-1"></i>
                        "Edit"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PeriodForm(
    mode: FormMode,
    on_cancel: Callback<()>,
    on_success: Callback<()>,
) -> impl IntoView {
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

    // Load existing data if editing
    let mode_clone = mode.clone();
    let _load_effect = Effect::new(move || {
        if let FormMode::Edit(edit_id) = mode_clone.clone() {
            set_loading.set(true);
            spawn_local(async move {
                match crate::api::fetch_kebutuhan_bmn_detail(&edit_id).await {
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

    // Submit action
    let mode_for_submit = mode.clone();
    let submit_action = Action::new_local(move |_: &()| {
        let current_mode = mode_for_submit.clone();
        let nama_val = nama.get();
        let deskripsi_val = deskripsi.get();
        let tahun_val = tahun.get();
        let tgl_mulai_val = tgl_mulai.get();
        let tgl_selesai_val = tgl_selesai.get();
        let pilihan_satker_val = pilihan_satker.get();
        let satker_ids_val = satker_ids.get();
        let version_val = version.get();
        let on_success_clone = on_success.clone();

        async move {
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
                    on_success_clone.run(());
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan: {:?}", e)));
                }
            }
        }
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
        submit_action.dispatch(());
    };

    let current_year = 2025;
    let years: Vec<i32> = (2020..=current_year + 2).rev().collect();
    let years = StoredValue::new(years);

    view! {
        <div class="rounded-xl border border-white/10 bg-slate-900/60 p-6 backdrop-blur">
            // Header
            <div class="mb-6">
                <h2 class="text-xl font-bold text-slate-100">
                    {move || match mode.clone() {
                        FormMode::Create => "Buat Periode Baru",
                        FormMode::Edit(_) => "Edit Periode",
                    }}
                </h2>
                <p class="mt-1 text-sm text-slate-400">
                    "Isi informasi periode pengumpulan kebutuhan BMN"
                </p>
            </div>

            // Loading state
            <Show when=move || loading.get()>
                <div class="py-12 text-center">
                    <div class="mx-auto mb-3 h-8 w-8 animate-spin rounded-full border-b-2 border-gold-400"></div>
                    <p class="text-slate-400">"Memuat data..."</p>
                </div>
            </Show>

            // Error message
            <Show when=move || error.get().is_some()>
                <div class="mb-6 rounded-lg border border-red-200/20 bg-red-500/10 px-4 py-3 text-red-300">
                    <div class="flex items-center gap-2">
                        <i class="fas fa-exclamation-circle"></i>
                        <span>{move || error.get().unwrap_or_default()}</span>
                    </div>
                </div>
            </Show>

            // Form
            <Show when=move || !loading.get()>
                <form on:submit=on_submit class="space-y-6">
                    // Nama periode
                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-300">
                            "Nama Periode " <span class="text-red-400">"*"</span>
                        </label>
                        <input
                            type="text"
                            required
                            minlength="3"
                            maxlength="255"
                            class="w-full rounded-lg border border-white/10 bg-slate-800/50 px-4 py-2 text-white placeholder-slate-500 focus:border-gold-400/50 focus:ring-2 focus:ring-gold-400/20"
                            placeholder="Contoh: Pengajuan Kebutuhan BMN Tahun 2025"
                            on:input=move |ev| set_nama.set(event_target_value(&ev))
                            prop:value=move || nama.get()
                        />
                    </div>

                    // Deskripsi
                    <div>
                        <label class="mb-1 block text-sm font-medium text-slate-300">
                            "Deskripsi"
                        </label>
                        <textarea
                            rows="3"
                            class="w-full rounded-lg border border-white/10 bg-slate-800/50 px-4 py-2 text-white placeholder-slate-500 focus:border-gold-400/50 focus:ring-2 focus:ring-gold-400/20"
                            placeholder="Deskripsi periode (opsional)"
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                set_deskripsi.set(if v.is_empty() { None } else { Some(v) });
                            }
                            prop:value=move || deskripsi.get().unwrap_or_default()
                        ></textarea>
                    </div>

                    // Tahun & Periode
                    <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
                        <div>
                            <label class="mb-1 block text-sm font-medium text-slate-300">
                                "Tahun Anggaran " <span class="text-red-400">"*"</span>
                            </label>
                            <select
                                class="w-full rounded-lg border border-white/10 bg-slate-800/50 px-4 py-2 text-white focus:border-gold-400/50 focus:ring-2 focus:ring-gold-400/20"
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
                            <label class="mb-1 block text-sm font-medium text-slate-300">
                                "Tanggal Mulai " <span class="text-red-400">"*"</span>
                            </label>
                            <input
                                type="date"
                                required
                                class="w-full rounded-lg border border-white/10 bg-slate-800/50 px-4 py-2 text-white focus:border-gold-400/50 focus:ring-2 focus:ring-gold-400/20"
                                on:input=move |ev| set_tgl_mulai.set(event_target_value(&ev))
                                prop:value=move || tgl_mulai.get()
                            />
                        </div>
                        <div>
                            <label class="mb-1 block text-sm font-medium text-slate-300">
                                "Tanggal Selesai " <span class="text-red-400">"*"</span>
                            </label>
                            <input
                                type="date"
                                required
                                class="w-full rounded-lg border border-white/10 bg-slate-800/50 px-4 py-2 text-white focus:border-gold-400/50 focus:ring-2 focus:ring-gold-400/20"
                                on:input=move |ev| set_tgl_selesai.set(event_target_value(&ev))
                                prop:value=move || tgl_selesai.get()
                            />
                        </div>
                    </div>

                    // Pilihan Satker
                    <div>
                        <label class="mb-3 block text-sm font-medium text-slate-300">
                            "Pilihan Satker " <span class="text-red-400">"*"</span>
                        </label>
                        <div class="flex gap-6">
                            <label class="flex cursor-pointer items-center">
                                <input
                                    type="radio"
                                    name="pilihan_satker"
                                    class="h-4 w-4 text-gold-500"
                                    checked=move || pilihan_satker.get() == PilihanSatker::Semua
                                    on:change=move |_| set_pilihan_satker.set(PilihanSatker::Semua)
                                />
                                <span class="ml-2 text-slate-300">"Semua Satker"</span>
                            </label>
                            <label class="flex cursor-pointer items-center">
                                <input
                                    type="radio"
                                    name="pilihan_satker"
                                    class="h-4 w-4 text-gold-500"
                                    checked=move || pilihan_satker.get() == PilihanSatker::Sebagian
                                    on:change=move |_| set_pilihan_satker.set(PilihanSatker::Sebagian)
                                />
                                <span class="ml-2 text-slate-300">"Sebagian Satker"</span>
                            </label>
                        </div>
                        <p class="mt-2 text-xs text-slate-500">
                            "Pilih 'Semua Satker' untuk mengizinkan semua satker mengajukan, atau 'Sebagian Satker' untuk memilih satker tertentu"
                        </p>
                    </div>

                    // Info box for eligible BMN and satkers
                    <div class="rounded-lg border border-blue-500/20 bg-blue-500/10 p-4">
                        <div class="flex items-start gap-3">
                            <i class="fas fa-info-circle text-blue-300 mt-0.5"></i>
                            <div class="text-sm text-blue-200">
                                <p class="font-medium mb-1">"Konfigurasi Lanjutan"</p>
                                <p class="text-blue-300/80">
                                    "Setelah periode dibuat, Anda dapat mengkonfigurasi:"
                                </p>
                                <ul class="mt-2 space-y-1 text-blue-300/80">
                                    <li>"• Daftar BMN yang dapat diajukan (filtered by standar kodefikasi)"</li>
                                    <li>"• Daftar satker yang dapat mengajukan (jika pilih 'Sebagian Satker')"</li>
                                </ul>
                            </div>
                        </div>
                    </div>

                    // Action buttons
                    <div class="flex justify-end gap-3 border-t border-white/10 pt-6">
                        <button
                            type="button"
                            on:click=move |_| on_cancel.run(())
                            class="rounded-lg border border-white/10 px-6 py-2 text-slate-300 transition-colors hover:bg-slate-800/50"
                        >
                            "Batal"
                        </button>
                        <button
                            type="submit"
                            class="inline-flex items-center gap-2 rounded-lg bg-gold-500 px-6 py-2 font-semibold text-navy-900 transition-all hover:bg-gold-400 disabled:cursor-not-allowed disabled:opacity-50"
                            disabled=move || submitting.get() || !is_valid.get()
                        >
                            <Show when=move || submitting.get()>
                                <div class="h-4 w-4 animate-spin rounded-full border-b-2 border-navy-900"></div>
                            </Show>
                            {move || if submitting.get() { "Menyimpan..." } else { "Simpan Periode" }}
                        </button>
                    </div>
                </form>
            </Show>
        </div>
    }
}
