//! Bank Aset detail page — shows SIMAN record + riwayat tabs.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::ARROW_LEFT;

use super::dashboard_page::format_rupiah;
use crate::api::bank_aset::{self, BankAsetDetail, RiwayatEntry};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::path;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Ringkasan,
    Pemakaian,
    Penghapusan,
    Kebutuhan,
}

#[component]
pub fn BankAsetDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id_signal = Signal::derive(move || params.with(|p| p.get("id").unwrap_or_default()));

    let (detail, set_detail) = signal::<Option<BankAsetDetail>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (tab, set_tab) = signal(Tab::Ringkasan);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let id = id_signal.get();
        if id.is_empty() {
            set_loading.set(false);
            set_error.set(Some(AppError::not_found("Id aset tidak diberikan.")));
            return;
        }
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match bank_aset::fetch_detail(&id).await {
                Ok(d) => {
                    set_detail.set(Some(d));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Bank Aset", path::BANK_ASET_DASHBOARD),
        PageBreadcrumb::new("Daftar", path::BANK_ASET_DAFTAR),
        PageBreadcrumb::leaf("Detail"),
    ];

    view! {
        <PageLayout
            title="Detail Aset"
            description="Informasi lengkap aset beserta riwayat pemakaian, penghapusan, dan kebutuhan."
            icon="fas fa-cube"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || {
                view! {
                    <A
                        href=path::BANK_ASET_DAFTAR
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=ARROW_LEFT />
                        </span>
                        "Kembali ke Daftar"
                    </A>
                }
                    .into_any()
            })
        >
            {move || {
                if loading.get() && detail.get().is_none() {
                    view! { <LoadingState message="Memuat detail aset..." /> }.into_any()
                } else if let Some(err) = error.get() {
                    view! {
                        <ErrorState
                            error=err
                            on_retry=Box::new(move || {
                                set_reload_tick.update(|t| *t += 1);
                            })
                        />
                    }
                        .into_any()
                } else if let Some(d) = detail.get() {
                    view! { <DetailBody detail=d tab=tab set_tab=set_tab /> }.into_any()
                } else {
                    view! {
                        <EmptyState
                            title="Aset tidak ditemukan"
                            description="Data aset tidak tersedia di basis data SIMAN."
                            icon="fas fa-box-open"
                        />
                    }
                        .into_any()
                }
            }}
        </PageLayout>
    }
}

#[component]
fn DetailBody(
    detail: BankAsetDetail,
    tab: ReadSignal<Tab>,
    set_tab: WriteSignal<Tab>,
) -> impl IntoView {
    let item = detail.item.clone();
    let nama = item.nama_aset.clone().unwrap_or_else(|| "-".to_string());
    let kategori = item.kategori_aset.clone();
    let no_aset = item.no_aset.clone();
    let nilai = item
        .nilai_perolehan
        .map(format_rupiah)
        .unwrap_or_else(|| "-".to_string());
    let kondisi = item.kondisi.clone().unwrap_or_else(|| "-".to_string());

    let pemakaian = detail.riwayat_pemakaian.clone();
    let penghapusan = detail.riwayat_penghapusan.clone();
    let kebutuhan = detail.riwayat_kebutuhan.clone();

    view! {
        <SectionCard
            title=nama.clone()
            description=format!("{} · No. Aset {}", kategori, no_aset)
            icon="fas fa-cube"
        >
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <InfoField
                    label="Kode Barang"
                    value=item.kode_barang.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField label="NUP" value=item.nup.clone().unwrap_or_else(|| "-".to_string()) />
                <InfoField
                    label="Merk"
                    value=item.merk.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField
                    label="Tipe"
                    value=item.tipe.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField label="Kondisi" value=kondisi />
                <InfoField
                    label="Tanggal Perolehan"
                    value=item.tgl_perolehan.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField label="Nilai Perolehan" value=nilai />
                <InfoField
                    label="Satker"
                    value=item.satker.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField
                    label="Kode Satker"
                    value=item.kode_satker.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField
                    label="Lokasi"
                    value=item.lokasi.clone().unwrap_or_else(|| "-".to_string())
                />
                <InfoField label="Diperbarui" value=item.updated_at.clone() />
            </div>
        </SectionCard>

        <SectionCard
            title="Riwayat"
            description="Hubungan aset dengan modul lain dalam SIMPel."
            icon="fas fa-clock-rotate-left"
        >
            <div class="flex flex-wrap gap-2 border-b border-white/[0.08] pb-3">
                <TabButton
                    current=tab
                    value=Tab::Ringkasan
                    label="Ringkasan"
                    icon="fas fa-info"
                    set_tab=set_tab
                />
                <TabButton
                    current=tab
                    value=Tab::Pemakaian
                    label="Pemakaian"
                    icon="fas fa-handshake"
                    set_tab=set_tab
                />
                <TabButton
                    current=tab
                    value=Tab::Penghapusan
                    label="Penghapusan"
                    icon="fas fa-trash"
                    set_tab=set_tab
                />
                <TabButton
                    current=tab
                    value=Tab::Kebutuhan
                    label="Kebutuhan"
                    icon="fas fa-clipboard-list"
                    set_tab=set_tab
                />
            </div>
            <div class="mt-4">
                {move || match tab.get() {
                    Tab::Ringkasan => {
                        view! {
                            <p class="rounded-lg border border-white/[0.05] bg-white/[0.02] px-4 py-3 text-sm leading-relaxed text-slate-300">
                                "Pilih tab di atas untuk melihat riwayat pemakaian, usulan penghapusan, atau pengajuan kebutuhan yang terkait dengan aset ini."
                            </p>
                        }
                            .into_any()
                    }
                    Tab::Pemakaian => {
                        render_riwayat(&pemakaian, "Belum ada riwayat pemakaian untuk aset ini.")
                            .into_any()
                    }
                    Tab::Penghapusan => {
                        render_riwayat(&penghapusan, "Belum ada usulan penghapusan untuk aset ini.")
                            .into_any()
                    }
                    Tab::Kebutuhan => {
                        render_riwayat(
                                &kebutuhan,
                                "Belum ada pengajuan kebutuhan yang terkait aset ini.",
                            )
                            .into_any()
                    }
                }}
            </div>
        </SectionCard>
    }
}

#[component]
fn TabButton(
    current: ReadSignal<Tab>,
    value: Tab,
    #[prop(into)] label: String,
    #[prop(into)] icon: String,
    set_tab: WriteSignal<Tab>,
) -> impl IntoView {
    let is_active = move || current.get() == value;
    view! {
        <button
            type="button"
            on:click=move |_| set_tab.set(value)
            class=move || {
                let base = "focus-ring inline-flex items-center gap-2 rounded-lg px-3 py-1.5 text-xs font-semibold transition";
                if is_active() {
                    format!("{base} bg-gold-gradient text-navy-950 shadow-sm")
                } else {
                    format!(
                        "{base} border border-white/10 bg-white/[0.04] text-slate-200 hover:bg-white/[0.08]",
                    )
                }
            }
        >
            <AppIcon icon=icon_from_fa_class(&icon) size=10 />
            {label}
        </button>
    }
}

#[component]
fn InfoField(#[prop(into)] label: String, #[prop(into)] value: String) -> impl IntoView {
    view! {
        <div class="rounded-lg border border-white/[0.05] bg-white/[0.02] p-3">
            <dt class="text-[0.65rem] uppercase tracking-wide text-slate-500">{label}</dt>
            <dd class="mt-1 text-sm text-slate-100">{value}</dd>
        </div>
    }
}

fn render_riwayat(entries: &[RiwayatEntry], empty_msg: &str) -> impl IntoView {
    if entries.is_empty() {
        let msg = empty_msg.to_string();
        return view! { <EmptyState title="Belum ada riwayat" description=msg icon="fas fa-clock" /> }
        .into_any();
    }
    let rows: Vec<_> = entries.iter().map(|e| {
        let ref_no = e.ref_no.clone().unwrap_or_else(|| "-".to_string());
        let status = e.status.clone().unwrap_or_else(|| "-".to_string());
        let deskripsi = e.deskripsi.clone().unwrap_or_else(|| "-".to_string());
        let tanggal = e.tanggal.clone().unwrap_or_else(|| "-".to_string());
        view! {
            <li class="flex flex-col gap-1 rounded-lg border border-white/[0.05] bg-white/[0.03] px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <p class="text-sm font-semibold text-slate-100">{ref_no}</p>
                    <p class="mt-0.5 text-xs text-slate-400">{deskripsi}</p>
                </div>
                <div class="flex flex-col items-start sm:items-end">
                    <span class="inline-flex rounded-full bg-gold-500/10 px-2.5 py-0.5 text-[0.65rem] font-semibold text-gold-300 ring-1 ring-gold-500/20">
                        {status}
                    </span>
                    <span class="mt-1 text-[0.65rem] text-slate-500">{tanggal}</span>
                </div>
            </li>
        }
    }).collect();
    view! { <ul class="flex flex-col gap-2">{rows}</ul> }.into_any()
}
