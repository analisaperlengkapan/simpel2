//! Ukuran Pegawai Component
//!
//! Allows employees to input their personal uniform sizes (baju, celana, sepatu).

use crate::api::{
    AppError, PegawaiPakaianDinas, Ukuran, UpsertPegawaiUkuranRequest, fetch_master_ukuran,
    fetch_pegawai_ukuran, upsert_pegawai_ukuran,
};
use crate::components::layout::{ErrorState, LoadingState, PageLayout, SectionCard};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{CHECK_CIRCLE, FLOPPY_DISK, INFO, WARNING_CIRCLE};

/// `()`-keyed master ukuran query — same baju/celana/sepatu fetch
/// for every consumer, so a single cache slot suffices.
///
/// The three category fetches run in parallel via `try_join3`, so the
/// total wait time is `max(latencies)` instead of `sum(latencies)`.
async fn query_master_ukuran(_: ()) -> Result<(Vec<Ukuran>, Vec<Ukuran>, Vec<Ukuran>), AppError> {
    let (baju, celana, sepatu) = futures::future::try_join3(
        fetch_master_ukuran(Some("BAJU".to_string())),
        fetch_master_ukuran(Some("CELANA".to_string())),
        fetch_master_ukuran(Some("SEPATU".to_string())),
    )
    .await?;
    Ok((baju.data, celana.data, sepatu.data))
}

/// Per-pegawai ukuran query — keyed by pegawai id (`String`).
async fn query_pegawai_ukuran(pegawai_id: String) -> Result<Option<PegawaiPakaianDinas>, AppError> {
    fetch_pegawai_ukuran(pegawai_id).await.map(|r| r.data)
}

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

    let client: QueryClient = expect_context();

    // Master ukuran shared across the app — single `()` cache slot.
    let master_ukuran = client.local_resource(query_master_ukuran, || ());

    // Existing ukuran for this pegawai — keyed cache so multiple
    // pegawai pages each cache under their own id.
    let pegawai_id_for_fetch = pegawai_id.clone();
    let existing_ukuran =
        client.local_resource(query_pegawai_ukuran, move || pegawai_id_for_fetch.clone());

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
                    <AppIcon icon=CHECK_CIRCLE />
                    {move || success_message.get()}
                </div>
            </Show>
            <Show when=move || error_message.get().is_some()>
                <div class="mt-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                    <AppIcon icon=WARNING_CIRCLE />
                    {move || error_message.get()}
                </div>
            </Show>

            // Size form
            <div class="mt-4">
                <Suspense fallback=move || {
                    view! { <LoadingState message="Memuat data ukuran...".to_string() /> }
                }>
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
                                            ukuran_baju: if baju.is_empty() {
                                                None
                                            } else {
                                                Some(baju)
                                            },
                                            ukuran_celana: if celana.is_empty() {
                                                None
                                            } else {
                                                Some(celana)
                                            },
                                            ukuran_sepatu: if sepatu.is_empty() {
                                                None
                                            } else {
                                                Some(sepatu)
                                            },
                                        };
                                        match upsert_pegawai_ukuran(request).await {
                                            Ok(_) => {
                                                set_success_message
                                                    .set(Some("Ukuran berhasil disimpan!".to_string()))
                                            }
                                            Err(e) => set_error_message.set(Some(e.user_message())),
                                        }
                                        set_is_saving.set(false);
                                    });
                                };
                                view! {
                                    <SectionCard title="Isi Ukuran">
                                        <form on:submit=on_submit>
                                            <div class="grid grid-cols-1 gap-5 sm:grid-cols-3">
                                                {render_size_select(
                                                    "Ukuran Baju",
                                                    "fas fa-tshirt text-info-400",
                                                    baju_sizes,
                                                    ukuran_baju,
                                                    set_ukuran_baju,
                                                    "Ukuran standar: S, M, L, XL, XXL, XXXL",
                                                )}
                                                {render_size_select(
                                                    "Ukuran Celana",
                                                    "fas fa-male text-success-400",
                                                    celana_sizes,
                                                    ukuran_celana,
                                                    set_ukuran_celana,
                                                    "Ukuran standar: 27-42 (angka)",
                                                )}
                                                {render_size_select(
                                                    "Ukuran Sepatu",
                                                    "fas fa-shoe-prints text-gold-400",
                                                    sepatu_sizes,
                                                    ukuran_sepatu,
                                                    set_ukuran_sepatu,
                                                    "Ukuran standar: 36-46 (angka)",
                                                )}
                                            </div>
                                            <div class="mt-5 flex justify-end border-t border-white/[0.04] pt-4">
                                                <button
                                                    type="submit"
                                                    class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                                    prop:disabled=move || is_saving.get()
                                                >
                                                    <span class="text-xs">
                                                        <AppIcon icon=FLOPPY_DISK />
                                                    </span>
                                                    {move || {
                                                        if is_saving.get() {
                                                            "Menyimpan..."
                                                        } else {
                                                            "Simpan Ukuran"
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </form>
                                    </SectionCard>
                                }
                                    .into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>

            // Size guide
            <SectionCard title="Panduan Pengukuran">
                <ul class="flex flex-col gap-2 text-sm text-slate-300">
                    <li class="flex items-start gap-2">
                        <span class="mt-0.5 text-xs text-gold-400 shrink-0">
                            <AppIcon icon=INFO />
                        </span>
                        <span>
                            <strong class="text-slate-100">"Baju:"</strong>
                            " Ukur lingkar dada pada bagian terlebar, pilih ukuran yang sesuai."
                        </span>
                    </li>
                    <li class="flex items-start gap-2">
                        <span class="mt-0.5 text-xs text-gold-400 shrink-0">
                            <AppIcon icon=INFO />
                        </span>
                        <span>
                            <strong class="text-slate-100">"Celana:"</strong>
                            " Ukur lingkar pinggang pada posisi normal."
                        </span>
                    </li>
                    <li class="flex items-start gap-2">
                        <span class="mt-0.5 text-xs text-gold-400 shrink-0">
                            <AppIcon icon=INFO />
                        </span>
                        <span>
                            <strong class="text-slate-100">"Sepatu:"</strong>
                            " Ukur panjang kaki dari tumit ke ujung jari terpanjang."
                        </span>
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
    // The legacy `icon_class` strings carry both an FA icon name and Tailwind
    // color utilities (e.g. "fas fa-tshirt text-info-400"). `icon_from_fa_class`
    // only consumes the icon name; the trailing color classes have to be
    // re-applied on a wrapping span so per-icon coloring (info/success/gold)
    // isn't silently flattened into `currentColor`.
    let trailing_classes: String = icon_class
        .split_whitespace()
        .filter(|tok| !matches!(*tok, "fas" | "far" | "fab") && !tok.starts_with("fa-"))
        .collect::<Vec<_>>()
        .join(" ");
    let wrapper_class = format!("inline-flex {}", trailing_classes);
    view! {
        <div class="rounded-xl border border-white/[0.06] bg-white/[0.03] p-4">
            <label class="mb-2 flex items-center gap-2 text-sm font-medium text-slate-200">
                <span class=wrapper_class>
                    <AppIcon icon=icon_from_fa_class(icon_class) size=14 />
                </span>
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
