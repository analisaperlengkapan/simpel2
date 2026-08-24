//! Pengajuan Pakaian Dinas List Component
//!
//! Displays uniform request periods + a working create form (#19). The create
//! form was previously broken against the backend contract; it now sends the
//! full `CreatePengajuanRequest` (periode, jenis, spesifikasi, scope) and
//! supports scope `semua` / `wilayah` (auto-resolves satkers per Kejati).

use crate::api::{
    AppError, CreatePengajuanPakaianDinasRequest, JenisPakaianDinas, PaginatedResponse,
    PengajuanPakaianDinas, SpesifikasiPakaianDinas, create_pengajuan_pakaian_dinas,
    delete_pengajuan_pakaian_dinas, fetch_jenis_pakaian_dinas, fetch_pengajuan_pakaian_dinas,
    fetch_spesifikasi_pakaian, fetch_wilayah_kejati,
};
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{BUILDING, CALENDAR_CHECK, CALENDAR_X, PLUS, TRASH, WARNING_CIRCLE};

async fn query_pengajuan_pakaian_dinas_page(
    key: (i32, i32),
) -> Result<PaginatedResponse<PengajuanPakaianDinas>, AppError> {
    let (page, _trigger) = key;
    fetch_pengajuan_pakaian_dinas(page, 20, None).await
}

async fn query_jenis(_: ()) -> Vec<JenisPakaianDinas> {
    fetch_jenis_pakaian_dinas(1, 100)
        .await
        .map(|r| r.data)
        .unwrap_or_default()
}

async fn query_wilayah(_: ()) -> Vec<String> {
    fetch_wilayah_kejati().await.unwrap_or_default()
}

async fn query_spesifikasi(jenis_id: String) -> Vec<SpesifikasiPakaianDinas> {
    let jid = if jenis_id.is_empty() {
        None
    } else {
        Some(jenis_id)
    };
    fetch_spesifikasi_pakaian(1, 200, jid)
        .await
        .map(|r| r.data)
        .unwrap_or_default()
}

#[component]
pub fn PakaianDinasPengajuanList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let (form_tgl_mulai, set_form_tgl_mulai) = signal(String::new());
    let (form_tgl_selesai, set_form_tgl_selesai) = signal(String::new());
    let form_jenis_id = RwSignal::new(String::new());
    let form_spesifikasi_ids = RwSignal::new(Vec::<String>::new());
    let form_scope = RwSignal::new("semua".to_string());
    let form_wilayah = RwSignal::new(String::new());
    let (form_keterangan, set_form_keterangan) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let refresh_trigger = RwSignal::new(0);

    let client: QueryClient = expect_context();
    let data_resource = client.local_resource(query_pengajuan_pakaian_dinas_page, move || {
        (page.get(), refresh_trigger.get())
    });
    let jenis_resource = client.local_resource(query_jenis, || ());
    let wilayah_resource = client.local_resource(query_wilayah, || ());
    let spesifikasi_resource =
        client.local_resource(query_spesifikasi, move || form_jenis_id.get());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error_message.set(None);

        let nama = form_nama.get();
        let tgl_mulai = form_tgl_mulai.get();
        let tgl_selesai = form_tgl_selesai.get();
        let spesifikasi_ids = form_spesifikasi_ids.get();
        let scope = form_scope.get();
        let wilayah = form_wilayah.get();
        let jenis_id = form_jenis_id.get();
        let keterangan = form_keterangan.get();

        // Client-side guards mirroring the backend contract.
        if tgl_mulai.is_empty() || tgl_selesai.is_empty() {
            set_error_message.set(Some(
                "Periode (tanggal mulai & selesai) wajib diisi".to_string(),
            ));
            return;
        }
        if spesifikasi_ids.is_empty() {
            set_error_message.set(Some("Pilih minimal satu spesifikasi pakaian".to_string()));
            return;
        }
        if scope == "wilayah" && wilayah.is_empty() {
            set_error_message.set(Some("Pilih wilayah Kejaksaan Tinggi".to_string()));
            return;
        }

        let tahun = tgl_mulai.get(0..4).and_then(|y| y.parse::<i32>().ok());

        set_is_loading.set(true);
        spawn_local(async move {
            let request = CreatePengajuanPakaianDinasRequest {
                nama,
                deskripsi: if keterangan.is_empty() {
                    None
                } else {
                    Some(keterangan)
                },
                tgl_mulai: Some(tgl_mulai),
                tgl_selesai: Some(tgl_selesai),
                is_reguler: true,
                tahun,
                pilihan_satker: scope.clone(),
                dengan_unit_kerja: false,
                jenis_pakaian_dinas_id: if jenis_id.is_empty() {
                    None
                } else {
                    Some(jenis_id)
                },
                spesifikasi_ids,
                satker_ids: None,
                wilayah_id: if scope == "wilayah" {
                    Some(wilayah)
                } else {
                    None
                },
            };

            match create_pengajuan_pakaian_dinas(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
                    set_form_tgl_mulai.set(String::new());
                    set_form_tgl_selesai.set(String::new());
                    form_jenis_id.set(String::new());
                    form_spesifikasi_ids.set(Vec::new());
                    form_scope.set("semua".to_string());
                    form_wilayah.set(String::new());
                    set_form_keterangan.set(String::new());
                    set_show_form.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => set_error_message.set(Some(e.user_message())),
            }
            set_is_loading.set(false);
        });
    };

    let input_class = "focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500";

    view! {
        <PageLayout
            title="Pengajuan Pakaian Dinas"
            icon="fas fa-calendar-alt"
            description="Kelola periode pengajuan pakaian dinas per tahun"
        >
            <div class="mb-4 flex justify-end">
                <button
                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    <span class="text-xs">
                        <AppIcon icon=PLUS />
                    </span>
                    "Buat Pengajuan"
                </button>
            </div>

            <Show when=move || show_form.get()>
                <SectionCard title="Buat Periode Pengajuan Baru">
                    <Show when=move || error_message.get().is_some()>
                        <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                            <AppIcon icon=WARNING_CIRCLE />
                            {move || error_message.get()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-2">
                            <FormField label="Nama Pengajuan">
                                <input
                                    type="text"
                                    class=input_class
                                    placeholder="Contoh: Pengajuan PDH Tahun 2027"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField label="Jenis Pakaian">
                                <select
                                    class=input_class
                                    prop:value=move || form_jenis_id.get()
                                    on:change=move |ev| {
                                        form_jenis_id.set(event_target_value(&ev));
                                        form_spesifikasi_ids.set(Vec::new());
                                    }
                                >
                                    <option value="">"— Semua / Pilih jenis —"</option>
                                    {move || {
                                        jenis_resource
                                            .get()
                                            .unwrap_or_default()
                                            .into_iter()
                                            .map(|j| {
                                                view! {
                                                    <option value=j.id.clone()>{j.nama.clone()}</option>
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </select>
                            </FormField>
                            <FormField label="Tanggal Mulai">
                                <input
                                    type="date"
                                    class=input_class
                                    prop:value=move || form_tgl_mulai.get()
                                    on:input=move |ev| {
                                        set_form_tgl_mulai.set(event_target_value(&ev))
                                    }
                                    required=true
                                />
                            </FormField>
                            <FormField label="Tanggal Selesai">
                                <input
                                    type="date"
                                    class=input_class
                                    prop:value=move || form_tgl_selesai.get()
                                    on:input=move |ev| {
                                        set_form_tgl_selesai.set(event_target_value(&ev))
                                    }
                                    required=true
                                />
                            </FormField>
                        </div>

                        // Spesifikasi multi-select
                        <div class="mt-5">
                            <FormField
                                label="Spesifikasi Pakaian (pilih satu atau lebih)"
                                full_width=true
                            >
                                <div class="flex flex-wrap gap-2">
                                    {move || {
                                        let specs = spesifikasi_resource.get().unwrap_or_default();
                                        if specs.is_empty() {
                                            view! {
                                                <span class="text-xs text-slate-500">
                                                    "Tidak ada spesifikasi untuk jenis ini."
                                                </span>
                                            }
                                                .into_any()
                                        } else {
                                            specs
                                                .into_iter()
                                                .map(|s| {
                                                    let sid = s.id.clone();
                                                    let sid_check = sid.clone();
                                                    let checked = move || {
                                                        form_spesifikasi_ids.get().contains(&sid_check)
                                                    };
                                                    view! {
                                                        <label class="inline-flex cursor-pointer items-center gap-2 rounded-lg border border-white/10 bg-white/[0.03] px-3 py-1.5 text-sm text-slate-200">
                                                            <input
                                                                type="checkbox"
                                                                class="h-4 w-4"
                                                                prop:checked=checked
                                                                on:change={
                                                                    let sid = sid.clone();
                                                                    move |_| {
                                                                        form_spesifikasi_ids
                                                                            .update(|v| {
                                                                                if let Some(pos) = v.iter().position(|x| x == &sid) {
                                                                                    v.remove(pos);
                                                                                } else {
                                                                                    v.push(sid.clone());
                                                                                }
                                                                            })
                                                                    }
                                                                }
                                                            />
                                                            {s.nama.clone()}
                                                        </label>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    }}
                                </div>
                            </FormField>
                        </div>

                        // Scope satker (semua / wilayah)
                        <div class="mt-5">
                            <FormField label="Lingkup Satker" full_width=true>
                                <div class="flex flex-wrap gap-6">
                                    <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                                        <input
                                            type="radio"
                                            name="scope_satker"
                                            class="h-4 w-4"
                                            prop:checked=move || form_scope.get() == "semua"
                                            on:change=move |_| form_scope.set("semua".to_string())
                                        />
                                        "Semua Satker"
                                    </label>
                                    <label class="flex cursor-pointer items-center gap-2 text-sm text-slate-200">
                                        <input
                                            type="radio"
                                            name="scope_satker"
                                            class="h-4 w-4"
                                            prop:checked=move || form_scope.get() == "wilayah"
                                            on:change=move |_| form_scope.set("wilayah".to_string())
                                        />
                                        "Per Wilayah (Kejati)"
                                    </label>
                                </div>
                            </FormField>
                        </div>

                        <Show when=move || form_scope.get() == "wilayah">
                            <div class="mt-3">
                                <FormField label="Wilayah Kejaksaan Tinggi" full_width=true>
                                    <select
                                        class=input_class
                                        prop:value=move || form_wilayah.get()
                                        on:change=move |ev| {
                                            form_wilayah.set(event_target_value(&ev))
                                        }
                                    >
                                        <option value="">"— Pilih wilayah —"</option>
                                        {move || {
                                            wilayah_resource
                                                .get()
                                                .unwrap_or_default()
                                                .into_iter()
                                                .map(|w| {
                                                    view! { <option value=w.clone()>{w.clone()}</option> }
                                                })
                                                .collect_view()
                                        }}
                                    </select>
                                </FormField>
                            </div>
                        </Show>

                        <div class="mt-5">
                            <FormField label="Keterangan" full_width=true>
                                <textarea
                                    class=format!("min-h-[80px] resize-y {}", input_class)
                                    placeholder="Keterangan tambahan (opsional)"
                                    prop:value=move || form_keterangan.get()
                                    on:input=move |ev| {
                                        set_form_keterangan.set(event_target_value(&ev))
                                    }
                                ></textarea>
                            </FormField>
                        </div>

                        <div class="mt-5 flex gap-3 border-t border-white/[0.04] pt-4">
                            <button
                                type="submit"
                                class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                prop:disabled=move || is_loading.get()
                            >
                                {move || if is_loading.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                            <button
                                type="button"
                                class="rounded-lg border border-white/10 bg-white/[0.04] px-5 py-2.5 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Batal"
                            </button>
                        </div>
                    </form>
                </SectionCard>
            </Show>

            <Suspense fallback=move || {
                view! { <LoadingState /> }
            }>
                {move || match data_resource.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(response)) => {
                        if response.data.is_empty() {
                            view! {
                                <EmptyState
                                    icon="fas fa-calendar-alt"
                                    title="Belum Ada Pengajuan"
                                    description="Klik tombol \"Buat Pengajuan\" untuk membuat periode baru."
                                />
                            }
                                .into_any()
                        } else {
                            render_pengajuan_cards(
                                response.data,
                                response.total,
                                response.total_pages,
                                page,
                                set_page,
                                refresh_trigger,
                            )
                        }
                    }
                }}
            </Suspense>
        </PageLayout>
    }
}

/// Status badge for pengajuan items.
fn status_badge(status: String, is_open: bool) -> impl IntoView {
    let (class, text) = if is_open {
        (
            "bg-success-500/15 text-success-300 ring-success-500/25",
            "Dibuka".to_string(),
        )
    } else {
        // `status` is the backend's `aktivitas_label` — the workflow step's
        // human label from `ms_aktivitas_bmn` ("Input", "Selesai", …). It is
        // empty only when the campaign points at an aktivitas code the lookup
        // table does not carry, so say "Ditutup" rather than render a blank pill.
        match status.as_str() {
            "" => (
                "bg-slate-500/15 text-slate-300 ring-slate-500/25",
                "Ditutup".to_string(),
            ),
            "Selesai" => (
                "bg-info-500/15 text-info-300 ring-info-500/25",
                "Selesai".to_string(),
            ),
            _ => ("bg-gold-500/15 text-gold-300 ring-gold-500/25", status),
        }
    };
    view! {
        <span class=format!(
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ring-1 {}",
            class,
        )>{text}</span>
    }
}

/// Renders the card grid for pengajuan items.
fn render_pengajuan_cards(
    data: Vec<PengajuanPakaianDinas>,
    total: i64,
    _total_pages: i32,
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
    refresh_trigger: RwSignal<i32>,
) -> AnyView {
    let on_delete = move |id: String| {
        let confirmed = web_sys::window()
            .and_then(|w| {
                w.confirm_with_message("Yakin ingin menghapus periode pengajuan ini?")
                    .ok()
            })
            .unwrap_or(false);

        if confirmed {
            spawn_local(async move {
                match delete_pengajuan_pakaian_dinas(id).await {
                    Ok(_) => refresh_trigger.update(|v| *v += 1),
                    Err(e) => leptos::logging::error!("Failed to delete: {}", e.user_message()),
                }
            });
        }
    };
    view! {
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            <For
                each=move || data.clone()
                key=|item| item.id.clone()
                children=move |item: PengajuanPakaianDinas| {
                    let item_id = item.id.clone();
                    let item_id_for_delete = item_id.clone();
                    let status_str = item.aktivitas_label.clone().unwrap_or_default();
                    let is_open = item.is_open;
                    let keterangan = item.deskripsi.clone();
                    let has_keterangan = keterangan.is_some();
                    view! {
                        <div class="rounded-2xl border border-white/[0.06] bg-surface-panel p-5 transition hover:border-white/10">
                            <div class="flex items-start justify-between mb-3">
                                <div>
                                    <h3 class="text-sm font-semibold text-slate-100">
                                        {item.nama}
                                    </h3>
                                    <p class="text-xs text-slate-400">
                                        "Tahun: " {item.tahun.to_string()}
                                    </p>
                                </div>
                                {status_badge(status_str, is_open)}
                            </div>

                            <div class="space-y-1 text-xs text-slate-400">
                                <p>
                                    <span class="mr-1.5 text-success-400 w-3.5">
                                        <AppIcon icon=CALENDAR_CHECK />
                                    </span>
                                    "Buka: "
                                    {item.tgl_mulai.unwrap_or_else(|| "-".to_string())}
                                </p>
                                <p>
                                    <span class="mr-1.5 text-danger-400 w-3.5">
                                        <AppIcon icon=CALENDAR_X />
                                    </span>
                                    "Tutup: "
                                    {item.tgl_selesai.unwrap_or_else(|| "-".to_string())}
                                </p>
                            </div>

                            <Show when=move || has_keterangan>
                                <p class="mt-2 text-xs italic text-slate-500">
                                    {keterangan.clone()}
                                </p>
                            </Show>

                            <div class="mt-4 flex gap-2 border-t border-white/[0.04] pt-3">
                                <a
                                    href=crate::routes::url::pakaian_satker_list(&item_id)
                                    class="flex-1 inline-flex items-center justify-center gap-1.5 rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-1.5 text-xs font-medium text-info-300 transition hover:bg-info-500/20"
                                >
                                    <span class="text-2xs">
                                        <AppIcon icon=BUILDING />
                                    </span>
                                    "Satker"
                                </a>
                                <button
                                    class="inline-flex items-center justify-center rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-1.5 text-xs text-danger-300 transition hover:bg-danger-500/20"
                                    on:click=move |_| on_delete(item_id_for_delete.clone())
                                >
                                    <span class="text-2xs">
                                        <AppIcon icon=TRASH />
                                    </span>
                                </button>
                            </div>
                        </div>
                    }
                }
            />
        </div>

        // Pagination
        <div class="mt-4 flex items-center justify-between border-t border-white/[0.04] pt-4">
            <p class="text-xs text-slate-400">
                "Total: " <span class="font-medium text-slate-200">{total}</span> " pengajuan"
            </p>
            <div class="flex gap-2">
                <button
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get() <= 1
                    on:click=move |_| set_page.update(|p| *p -= 1)
                >
                    "Sebelumnya"
                </button>
                <button
                    class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                    prop:disabled=move || page.get()
                >
                    = total_pages
                    on:click=move |_| set_page.update(|p| *p += 1)
                    >
                    "Selanjutnya"
                </button>
            </div>
        </div>
    }.into_any()
}
