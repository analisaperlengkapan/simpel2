//! Ukuran Pegawai Component
//!
//! Allows employees to input their personal uniform sizes (baju, celana, sepatu).

use crate::api::{
    AppError, PegawaiPakaianDinas, PegawaiWithSizes, Ukuran, UpsertPegawaiUkuranRequest,
    fetch_master_ukuran, fetch_pegawai_ukuran, fetch_pegawai_with_sizes, upsert_pegawai_ukuran,
};
use crate::components::layout::{
    DataTable, DataTableColumn, EmptyState, ErrorState, LoadingState, PageLayout, SectionCard,
    FormField,
};
use crate::routes;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn UkuranPegawai(
    /// Pegawai ID from JWT claims or context
    pegawai_id: String,
    /// Pegawai name for display
    pegawai_nama: String,
    /// Pegawai NIP for display
    pegawai_nip: String,
) -> impl IntoView {
    let (ukuran_baju, set_ukuran_baju) = signal(String::new());
    let (ukuran_celana, set_ukuran_celana) = signal(String::new());
    let (ukuran_sepatu, set_ukuran_sepatu) = signal(String::new());
    let (is_saving, set_is_saving) = signal(false);
    let (success_message, set_success_message) = signal(Option::<String>::None);
    let (error_message, set_error_message) = signal(Option::<String>::None);

    let pegawai_id_clone = pegawai_id.clone();

    // Fetch master ukuran data
    let master_ukuran = LocalResource::new(|| async move {
        let baju = fetch_master_ukuran(Some("BAJU".to_string())).await;
        let celana = fetch_master_ukuran(Some("CELANA".to_string())).await;
        let sepatu = fetch_master_ukuran(Some("SEPATU".to_string())).await;
        // Return first error if any, otherwise combine
        let baju = baju.map(|r| r.data)?;
        let celana = celana.map(|r| r.data)?;
        let sepatu = sepatu.map(|r| r.data)?;
        Ok::<_, AppError>((baju, celana, sepatu))
    });

    // Fetch existing ukuran for pegawai
    let pegawai_id_for_fetch = pegawai_id.clone();
    let existing_ukuran = LocalResource::new(move || {
        let pid = pegawai_id_for_fetch.clone();
        async move { fetch_pegawai_ukuran(pid).await.map(|r| r.data) }
    });

    // Effect to populate form when existing data loads
    Effect::new(move || {
        if let Some(Ok(Some(ukuran))) = existing_ukuran.get() {
            if let Some(baju) = ukuran.ukuran_baju {
                set_ukuran_baju.set(baju);
            }
            if let Some(celana) = ukuran.ukuran_celana {
                set_ukuran_celana.set(celana);
            }
            if let Some(sepatu) = ukuran.ukuran_sepatu {
                set_ukuran_sepatu.set(sepatu);
            }
        }
    });

    let pegawai_id_for_submit = pegawai_id.clone();
    let initial = pegawai_nama.chars().next().unwrap_or('?').to_string();

    view! {
        <PageLayout
            title="Ukuran Pakaian Dinas"
            icon="fas fa-tshirt"
            description="Isi ukuran pakaian dinas Anda untuk keperluan pengajuan"
        >
            // Employee info card
            <SectionCard title="Data Pegawai" dense=true>
                <div class="flex items-center gap-4">
                    <div class="flex h-11 w-11 items-center justify-center rounded-full bg-gold-500/15 text-sm font-bold text-gold-400 ring-1 ring-gold-500/25">
                        {initial}
                    </div>
                    <div>
                        <p class="text-sm font-semibold text-slate-100">{pegawai_nama}</p>
                        <p class="text-xs text-slate-400">"NIP: " {pegawai_nip}</p>
                    </div>
                </div>
            </SectionCard>

            // Messages
            <Show when=move || success_message.get().is_some()>
                <div class="mt-4 flex items-center gap-2 rounded-xl border border-success-500/30 bg-success-500/[0.08] px-4 py-3 text-sm text-success-300">
                    <i class="fas fa-check-circle"></i>
                    {move || success_message.get()}
                </div>
            </Show>
            <Show when=move || error_message.get().is_some()>
                <div class="mt-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <i class="fas fa-exclamation-circle"></i>
                    {move || error_message.get()}
                </div>
            </Show>

            // Size form
            <div class="mt-4">
                <Suspense fallback=move || view! { <LoadingState message="Memuat data ukuran...".to_string() /> }>
                    {move || {
                        let pid = pegawai_id_for_submit.clone();
                        match master_ukuran.get() {
                            None => view! { <LoadingState /> }.into_any(),
                            Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                            Some(Ok((baju_sizes, celana_sizes, sepatu_sizes))) => {
                                let pid2 = pid.clone();
                                let on_submit = move |ev: leptos::ev::SubmitEvent| {
                                    ev.prevent_default();
                                    set_is_saving.set(true);
                                    set_success_message.set(None);
                                    set_error_message.set(None);
                                    let pid = pid2.clone();
                                    let baju = ukuran_baju.get();
                                    let celana = ukuran_celana.get();
                                    let sepatu = ukuran_sepatu.get();
                                    spawn_local(async move {
                                        let request = UpsertPegawaiUkuranRequest {
                                            pegawai_id: pid,
                                            ukuran_baju: if baju.is_empty() { None } else { Some(baju) },
                                            ukuran_celana: if celana.is_empty() { None } else { Some(celana) },
                                            ukuran_sepatu: if sepatu.is_empty() { None } else { Some(sepatu) },
                                        };
                                        match upsert_pegawai_ukuran(request).await {
                                            Ok(_) => set_success_message.set(Some("Ukuran berhasil disimpan!".to_string())),
                                            Err(e) => set_error_message.set(Some(e.user_message())),
                                        }
                                        set_is_saving.set(false);
                                    });
                                };
                                view! {
                                    <SectionCard title="Isi Ukuran">
                                        <form on:submit=on_submit>
                                            <div class="grid grid-cols-1 gap-5 sm:grid-cols-3">
                                                {render_size_select("Ukuran Baju", "fas fa-tshirt text-info-400", baju_sizes, ukuran_baju, set_ukuran_baju, "Ukuran standar: S, M, L, XL, XXL, XXXL")}
                                                {render_size_select("Ukuran Celana", "fas fa-male text-success-400", celana_sizes, ukuran_celana, set_ukuran_celana, "Ukuran standar: 27-42 (angka)")}
                                                {render_size_select("Ukuran Sepatu", "fas fa-shoe-prints text-gold-400", sepatu_sizes, ukuran_sepatu, set_ukuran_sepatu, "Ukuran standar: 36-46 (angka)")}
                                            </div>
                                            <div class="mt-5 flex justify-end border-t border-white/[0.04] pt-4">
                                                <button
                                                    type="submit"
                                                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                                    prop:disabled=move || is_saving.get()
                                                >
                                                    <i class="fas fa-save text-xs"></i>
                                                    {move || if is_saving.get() { "Menyimpan..." } else { "Simpan Ukuran" }}
                                                </button>
                                            </div>
                                        </form>
                                    </SectionCard>
                                }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>

            // Size guide
            <SectionCard title="Panduan Pengukuran">
                <ul class="flex flex-col gap-2 text-sm text-slate-300">
                    <li class="flex items-start gap-2">
                        <i class="fas fa-info-circle mt-0.5 text-xs text-gold-400 shrink-0"></i>
                        <span><strong class="text-slate-100">"Baju:"</strong>" Ukur lingkar dada pada bagian terlebar, pilih ukuran yang sesuai."</span>
                    </li>
                    <li class="flex items-start gap-2">
                        <i class="fas fa-info-circle mt-0.5 text-xs text-gold-400 shrink-0"></i>
                        <span><strong class="text-slate-100">"Celana:"</strong>" Ukur lingkar pinggang pada posisi normal."</span>
                    </li>
                    <li class="flex items-start gap-2">
                        <i class="fas fa-info-circle mt-0.5 text-xs text-gold-400 shrink-0"></i>
                        <span><strong class="text-slate-100">"Sepatu:"</strong>" Ukur panjang kaki dari tumit ke ujung jari terpanjang."</span>
                    </li>
                </ul>
            </SectionCard>
        </PageLayout>
    }
}

/// Helper to render a size select dropdown inside a styled card.
fn render_size_select(
    label: &'static str,
    icon_class: &'static str,
    sizes: Vec<Ukuran>,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
    hint: &'static str,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4">
            <label class="mb-2 flex items-center gap-2 text-sm font-medium text-slate-200">
                <i class=icon_class></i>
                {label}
            </label>
            <select
                class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3.5 py-2.5 text-sm text-slate-100"
                prop:value=move || value.get()
                on:change=move |ev| set_value.set(event_target_value(&ev))
            >
                <option value="">"-- Pilih Ukuran --"</option>
                <For
                    each=move || sizes.clone()
                    key=|u| u.id.clone()
                    children=move |u: Ukuran| {
                        let size = u.size.clone();
                        let size2 = size.clone();
                        view! { <option value=size>{size2}</option> }
                    }
                />
            </select>
            <p class="mt-1.5 text-xs text-slate-500">{hint}</p>
        </div>
    }
}

/// Component for admin to view/edit all employees' sizes in a satker
#[component]
pub fn UkuranPegawaiSatker(
    /// Satker ID to show employees for
    satker_id: String,
    /// Satker name for display
    satker_nama: String,
) -> impl IntoView {
    let (page, set_page) = signal(1);
    let satker_id_clone = satker_id.clone();

    let data_resource = LocalResource::new(move || {
        let sid = satker_id_clone.clone();
        let p = page.get();
        async move { fetch_pegawai_with_sizes(sid, p, 20).await }
    });

    view! {
        <PageLayout
            title=format!("Ukuran Pegawai — {}", satker_nama)
            icon="fas fa-users"
            description="Daftar ukuran pakaian dinas seluruh pegawai di satker"
        >
            <Suspense fallback=move || view! { <LoadingState /> }>
                {move || match data_resource.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(response)) => {
                        if response.data.is_empty() {
                            view! {
                                <EmptyState
                                    icon="fas fa-users"
                                    title="Tidak Ada Data Pegawai"
                                    description="Belum ada data pegawai di satker ini."
                                />
                            }.into_any()
                        } else {
                            render_satker_table(response.data, response.total, response.total_pages, page, set_page)
                        }
                    }
                }}
            </Suspense>
        </PageLayout>
    }
}

/// Renders the satker employee size table with pagination.
fn render_satker_table(
    data: Vec<PegawaiWithSizes>,
    total: i64,
    total_pages: i32,
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
) -> AnyView {
    let data_len = data.len();
    let columns: Vec<DataTableColumn<(usize, PegawaiWithSizes)>> = vec![
        DataTableColumn::new("No", move |item: &(usize, PegawaiWithSizes)| {
            let num = ((page.get() - 1) * 20 + item.0 as i32 + 1).to_string();
            view! { <span class="text-slate-400">{num}</span> }.into_any()
        }),
        DataTableColumn::new("NIP", |item: &(usize, PegawaiWithSizes)| {
            let nip = item.1.pegawai.nip.clone();
            view! { <span class="font-mono text-xs text-slate-300">{nip}</span> }.into_any()
        }),
        DataTableColumn::new("Nama", |item: &(usize, PegawaiWithSizes)| {
            let nama = item.1.pegawai.nama.clone();
            view! { <span class="font-medium text-slate-100">{nama}</span> }.into_any()
        }),
        DataTableColumn::new("Jabatan", |item: &(usize, PegawaiWithSizes)| {
            let jab = item.1.pegawai.jabatan.clone().unwrap_or_else(|| "-".to_string());
            view! { <span class="text-slate-400">{jab}</span> }.into_any()
        }),
        DataTableColumn::new("Baju", |item: &(usize, PegawaiWithSizes)| {
            let v = item.1.ukuran.as_ref().and_then(|u| u.ukuran_baju.clone()).unwrap_or_else(|| "-".to_string());
            view! { <span class="text-center text-slate-300">{v}</span> }.into_any()
        }),
        DataTableColumn::new("Celana", |item: &(usize, PegawaiWithSizes)| {
            let v = item.1.ukuran.as_ref().and_then(|u| u.ukuran_celana.clone()).unwrap_or_else(|| "-".to_string());
            view! { <span class="text-center text-slate-300">{v}</span> }.into_any()
        }),
        DataTableColumn::new("Sepatu", |item: &(usize, PegawaiWithSizes)| {
            let v = item.1.ukuran.as_ref().and_then(|u| u.ukuran_sepatu.clone()).unwrap_or_else(|| "-".to_string());
            view! { <span class="text-center text-slate-300">{v}</span> }.into_any()
        }),
        DataTableColumn::new("Status", |item: &(usize, PegawaiWithSizes)| {
            if item.1.ukuran.is_some() {
                view! {
                    <span class="inline-flex items-center rounded-full bg-success-500/15 px-2 py-0.5 text-xs font-medium text-success-300 ring-1 ring-success-500/25">
                        "Lengkap"
                    </span>
                }.into_any()
            } else {
                view! {
                    <span class="inline-flex items-center rounded-full bg-gold-500/15 px-2 py-0.5 text-xs font-medium text-gold-300 ring-1 ring-gold-500/25">
                        "Belum Isi"
                    </span>
                }.into_any()
            }
        }),
    ];

    let rows: Vec<(usize, PegawaiWithSizes)> = data.into_iter().enumerate().collect();

    view! {
        <SectionCard title="Daftar Pegawai">
            <DataTable columns=columns rows=rows />

            // Pagination
            <div class="mt-4 flex items-center justify-between border-t border-white/[0.04] pt-4">
                <p class="text-xs text-slate-400">
                    "Menampilkan "
                    <span class="font-medium text-slate-200">{data_len}</span>
                    " dari "
                    <span class="font-medium text-slate-200">{total}</span>
                    " pegawai"
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
        </SectionCard>
    }.into_any()
}
