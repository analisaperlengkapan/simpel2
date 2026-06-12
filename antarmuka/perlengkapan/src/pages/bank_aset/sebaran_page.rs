//! Bank Aset sebaran — per-satker distribution.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::LIST;

use super::dashboard_page::{format_rupiah, format_thousands};
use crate::api::bank_aset::{self, BankAsetSebaran, SebaranSatker};
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::path;

#[component]
pub fn BankAsetSebaranPage() -> impl IntoView {
    let (data, set_data) = signal::<Option<BankAsetSebaran>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match bank_aset::fetch_sebaran().await {
                Ok(d) => set_data.set(Some(d)),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Bank Aset", path::BANK_ASET_DASHBOARD),
        PageBreadcrumb::leaf("Sebaran"),
    ];

    view! {
        <PageLayout
            title="Sebaran Aset per Satker"
            description="Distribusi BMN berdasarkan satuan kerja, lengkap dengan komposisi kondisi."
            icon="fas fa-map-location-dot"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || {
                view! {
                    <A
                        href=path::BANK_ASET_DAFTAR
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=LIST />
                        </span>
                        "Daftar Aset"
                    </A>
                }
                    .into_any()
            })
        >
            <SectionCard
                title="Ringkasan per Satker"
                description="Satker diurutkan dari jumlah aset terbanyak."
                icon="fas fa-building"
            >
                {move || {
                    if loading.get() && data.get().is_none() {
                        view! { <LoadingState message="Memuat data sebaran..." /> }.into_any()
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
                    } else if let Some(d) = data.get() {
                        if d.satker.is_empty() {
                            view! {
                                <EmptyState
                                    title="Belum ada data sebaran"
                                    description="Sinkronisasi SIMAN belum menghasilkan data aset per satker."
                                    icon="fas fa-building-circle-exclamation"
                                />
                            }
                                .into_any()
                        } else {
                            render_table(d.satker).into_any()
                        }
                    } else {
                        view! { <LoadingState message="Memuat data sebaran..." /> }.into_any()
                    }
                }}
            </SectionCard>
        </PageLayout>
    }
}

fn render_table(satker: Vec<SebaranSatker>) -> impl IntoView {
    let total_aset: i64 = satker.iter().map(|s| s.total_aset).sum();
    let rows: Vec<_> = satker
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            let rank = (i + 1).to_string();
            let kode = s.kode_satker.clone().unwrap_or_else(|| "-".to_string());
            let nama = s.nama_satker.clone();
            let total = format_thousands(s.total_aset);
            let baik = format_thousands(s.aset_baik);
            let rusak = format_thousands(s.aset_rusak);
            let nilai = format_rupiah(s.nilai_perolehan);
            let pct = if total_aset > 0 {
                (s.total_aset as f64) / (total_aset as f64) * 100.0
            } else {
                0.0
            };
            let pct_label = format!("{:.1}%", pct);
            view! {
                <tr class="border-b border-white/[0.04] last:border-0">
                    <td class="py-3 pr-3 text-xs font-semibold text-gold-300">{rank}</td>
                    <td class="py-3 pr-3">
                        <p class="text-sm font-semibold text-slate-100">{nama}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{kode}</p>
                    </td>
                    <td class="py-3 pr-3 text-right text-sm text-slate-200">{total}</td>
                    <td class="py-3 pr-3 text-right text-xs text-success-300">{baik}</td>
                    <td class="py-3 pr-3 text-right text-xs text-danger-300">{rusak}</td>
                    <td class="py-3 pr-3 text-right text-sm text-slate-300">{nilai}</td>
                    <td class="py-3 text-right text-xs text-slate-400">{pct_label}</td>
                </tr>
            }
        })
        .collect();
    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="border-b border-white/[0.08]">
                    <tr class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                        <th class="py-2 pr-3 text-left font-medium">"#"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Satker"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Total Aset"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Baik"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Rusak"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Nilai"</th>
                        <th class="py-2 text-right font-medium">"% Populasi"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}
