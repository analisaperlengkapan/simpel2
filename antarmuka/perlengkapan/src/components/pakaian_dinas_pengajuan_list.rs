//! Pengajuan Pakaian Dinas List Component
//!
//! Displays list of uniform request periods with workflow management.

use crate::api::{
    AppError, CreatePengajuanPakaianDinasRequest, PengajuanPakaianDinas,
    create_pengajuan_pakaian_dinas, delete_pengajuan_pakaian_dinas,
    fetch_pengajuan_pakaian_dinas,
};
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn PakaianDinasPengajuanList() -> impl IntoView {
    let (page, set_page) = signal(1);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let (form_tahun, set_form_tahun) = signal(js_sys::Date::new_0().get_full_year().to_string());
    let (form_tgl_open, set_form_tgl_open) = signal(String::new());
    let (form_tgl_close, set_form_tgl_close) = signal(String::new());
    let (form_keterangan, set_form_keterangan) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error_message, set_error_message) = signal(Option::<String>::None);
    let refresh_trigger = RwSignal::new(0);

    let data_resource = LocalResource::new(move || {
        let p = page.get();
        let _trigger = refresh_trigger.get();
        async move { fetch_pengajuan_pakaian_dinas(p, 20, None).await }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_is_loading.set(true);
        set_error_message.set(None);

        let nama = form_nama.get();
        let tahun = form_tahun.get().parse::<i32>().unwrap_or(2025);
        let tgl_open = form_tgl_open.get();
        let tgl_close = form_tgl_close.get();
        let keterangan = form_keterangan.get();

        spawn_local(async move {
            let request = CreatePengajuanPakaianDinasRequest {
                nama,
                tahun,
                tgl_open: if tgl_open.is_empty() { None } else { Some(tgl_open) },
                tgl_close: if tgl_close.is_empty() { None } else { Some(tgl_close) },
                keterangan: if keterangan.is_empty() { None } else { Some(keterangan) },
            };

            match create_pengajuan_pakaian_dinas(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
                    set_form_tgl_open.set(String::new());
                    set_form_tgl_close.set(String::new());
                    set_form_keterangan.set(String::new());
                    set_show_form.set(false);
                    refresh_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    set_error_message.set(Some(e.user_message()));
                }
            }
            set_is_loading.set(false);
        });
    };

    view! {
        <PageLayout
            title="Pengajuan Pakaian Dinas"
            icon="fas fa-calendar-alt"
            description="Kelola periode pengajuan pakaian dinas per tahun"
        >
            // Action bar
            <div class="mb-4 flex justify-end">
                <button
                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    <i class="fas fa-plus text-xs"></i>
                    "Buat Pengajuan"
                </button>
            </div>

            // Create form
            <Show when=move || show_form.get()>
                <SectionCard title="Buat Periode Pengajuan Baru">
                    <Show when=move || error_message.get().is_some()>
                        <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                            <i class="fas fa-exclamation-circle"></i>
                            {move || error_message.get()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-2">
                            <FormField label="Nama Pengajuan">
                                <input
                                    type="text"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Contoh: Pengajuan PDH Tahun 2025"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField label="Tahun">
                                <input
                                    type="number"
                                    min="2020"
                                    max="2099"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                                    prop:value=move || form_tahun.get()
                                    on:input=move |ev| set_form_tahun.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField label="Tanggal Buka">
                                <input
                                    type="date"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                                    prop:value=move || form_tgl_open.get()
                                    on:input=move |ev| set_form_tgl_open.set(event_target_value(&ev))
                                />
                            </FormField>
                            <FormField label="Tanggal Tutup">
                                <input
                                    type="date"
                                    class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                                    prop:value=move || form_tgl_close.get()
                                    on:input=move |ev| set_form_tgl_close.set(event_target_value(&ev))
                                />
                            </FormField>
                        </div>
                        <div class="mt-5">
                            <FormField label="Keterangan" full_width=true>
                                <textarea
                                    class="focus-ring min-h-[80px] w-full resize-y rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500"
                                    placeholder="Keterangan tambahan (opsional)"
                                    prop:value=move || form_keterangan.get()
                                    on:input=move |ev| set_form_keterangan.set(event_target_value(&ev))
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

            // Data list
            <Suspense fallback=move || view! { <LoadingState /> }>
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
                            }.into_any()
                        } else {
                            render_pengajuan_cards(response.data, response.total, response.total_pages, page, set_page, refresh_trigger)
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
        ("bg-success-500/15 text-success-300 ring-success-500/25", "Dibuka".to_string())
    } else {
        match status.as_str() {
            "draft" => ("bg-slate-500/15 text-slate-300 ring-slate-500/25", "Draft".to_string()),
            "selesai" => ("bg-info-500/15 text-info-300 ring-info-500/25", "Selesai".to_string()),
            _ => ("bg-gold-500/15 text-gold-300 ring-gold-500/25", status),
        }
    };
    view! {
        <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ring-1 {}", class)>
            {text}
        </span>
    }
}

/// Renders the card grid for pengajuan items.
fn render_pengajuan_cards(
    data: Vec<PengajuanPakaianDinas>,
    total: i64,
    total_pages: i32,
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
                    let status_str = item.status.clone();
                    let is_open = item.is_open;
                    let keterangan = item.keterangan.clone();
                    let has_keterangan = keterangan.is_some();
                    view! {
                        <div class="rounded-2xl border border-white/[0.06] bg-surface-panel p-5 transition hover:border-white/10">
                            <div class="flex items-start justify-between mb-3">
                                <div>
                                    <h3 class="text-sm font-semibold text-slate-100">{item.nama}</h3>
                                    <p class="text-xs text-slate-400">"Tahun: " {item.tahun.to_string()}</p>
                                </div>
                                {status_badge(status_str, is_open)}
                            </div>

                            <div class="space-y-1 text-xs text-slate-400">
                                <p>
                                    <i class="fas fa-calendar-check mr-1.5 text-success-400 w-3.5"></i>
                                    "Buka: " {item.tgl_open.unwrap_or_else(|| "-".to_string())}
                                </p>
                                <p>
                                    <i class="fas fa-calendar-times mr-1.5 text-danger-400 w-3.5"></i>
                                    "Tutup: " {item.tgl_close.unwrap_or_else(|| "-".to_string())}
                                </p>
                            </div>

                            <Show when=move || has_keterangan>
                                <p class="mt-2 text-xs italic text-slate-500">{keterangan.clone()}</p>
                            </Show>

                            <div class="mt-4 flex gap-2 border-t border-white/[0.04] pt-3">
                                <a
                                    href=format!("/perlengkapan/pakaian-dinas/pengajuan/{}/satker", item_id)
                                    class="flex-1 inline-flex items-center justify-center gap-1.5 rounded-lg border border-info-500/30 bg-info-500/10 px-3 py-1.5 text-xs font-medium text-info-300 transition hover:bg-info-500/20"
                                >
                                    <i class="fas fa-building text-2xs"></i>
                                    "Satker"
                                </a>
                                <button
                                    class="inline-flex items-center justify-center rounded-lg border border-danger-500/30 bg-danger-500/10 px-3 py-1.5 text-xs text-danger-300 transition hover:bg-danger-500/20"
                                    on:click=move |_| on_delete(item_id_for_delete.clone())
                                >
                                    <i class="fas fa-trash text-2xs"></i>
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
                    prop:disabled=move || page.get() >= total_pages
                    on:click=move |_| set_page.update(|p| *p += 1)
                >
                    "Selanjutnya"
                </button>
            </div>
        </div>
    }.into_any()
}
