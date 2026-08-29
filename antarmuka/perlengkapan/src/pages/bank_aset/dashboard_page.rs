//! Bank Aset dashboard — KPI overview from SIMAN sync.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use lib_ui::utils::formatters::format_iso_local;
use phosphor_leptos::{DATABASE, LIST, MAP_PIN_AREA};

use crate::api::bank_aset::{self, BankAsetDashboard, LastSyncInfo};
use crate::api::error::AppError;
use crate::components::layout::{
    ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard, StatCard, StatTone,
};
use crate::routes::path;

#[component]
pub fn BankAsetDashboardPage() -> impl IntoView {
    let (data, set_data) = signal::<Option<BankAsetDashboard>>(None);
    let (last_sync, set_last_sync) = signal::<Option<LastSyncInfo>>(None);
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            let dash = bank_aset::fetch_dashboard(&Default::default()).await;
            let sync = bank_aset::fetch_last_sync().await;
            match (dash, sync) {
                (Ok(d), Ok(s)) => {
                    set_data.set(Some(d));
                    set_last_sync.set(Some(s));
                }
                (Err(e), _) | (_, Err(e)) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::leaf("Bank Aset"),
    ];

    view! {
        <PageLayout
            title="Dashboard Bank Aset"
            description="Ringkasan data BMN dari hasil sinkronisasi SIMAN."
            icon="fas fa-boxes-stacked"
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
                    <A
                        href=path::BANK_ASET_SEBARAN
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=MAP_PIN_AREA />
                        </span>
                        "Sebaran"
                    </A>
                }
                    .into_any()
            })
        >
            {move || {
                if loading.get() && data.get().is_none() {
                    view! { <LoadingState message="Memuat ringkasan bank aset..." /> }.into_any()
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
                    view! { <DashboardContent data=d last_sync=last_sync.get() /> }.into_any()
                } else {
                    view! { <LoadingState message="Memuat ringkasan bank aset..." /> }.into_any()
                }
            }}
        </PageLayout>
    }
}

#[component]
fn DashboardContent(data: BankAsetDashboard, last_sync: Option<LastSyncInfo>) -> impl IntoView {
    // Was printed as stored — `2026-08-28T09:28:53.886456Z` — which is both
    // unreadable and, being UTC, the wrong hour for a reader in WIB.
    let sync_label = last_sync
        .as_ref()
        .and_then(|s| s.last_sync_at.as_deref())
        .map(format_iso_local)
        .unwrap_or_else(|| "Belum pernah sinkron".to_string());

    let total_nilai = format_rupiah(data.total_nilai_perolehan);
    let total_aset = format_thousands(data.total_aset);
    let total_satker = format_thousands(data.total_satker);
    let total_kategori = format_thousands(data.total_kategori);

    let kondisi_view = render_kondisi(&data);
    let kategori_view = render_kategori(&data);
    let top_satker_view = render_top_satker(&data);
    let per_tahun_view = render_per_tahun(&data);

    view! {
        <SectionCard
            title="Ringkasan"
            description="Agregat BMN lintas kategori dan satuan kerja."
            icon="fas fa-chart-simple"
        >
            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                <StatCard
                    label="Total Aset"
                    value=total_aset
                    caption="unit tercatat"
                    icon="fas fa-cube"
                    tone=StatTone::Gold
                />
                <StatCard
                    label="Nilai Perolehan"
                    value=total_nilai
                    caption="akumulasi rupiah"
                    icon="fas fa-sack-dollar"
                    tone=StatTone::Success
                />
                <StatCard
                    label="Satuan Kerja"
                    value=total_satker
                    caption="satker dengan aset"
                    icon="fas fa-building"
                    tone=StatTone::Info
                />
                <StatCard
                    label="Kategori"
                    value=total_kategori
                    caption="klasifikasi SIMAN"
                    icon="fas fa-tags"
                    tone=StatTone::Neutral
                />
            </div>
            <p class="mt-4 text-xs text-slate-500">
                <span class="mr-2 text-gold-400">
                    <AppIcon icon=DATABASE />
                </span>
                "Sumber data: SIMAN — sinkron terakhir: "
                <span class="font-medium text-slate-300">{sync_label}</span>
            </p>
        </SectionCard>

        <div class="grid gap-4 lg:grid-cols-2">
            <SectionCard
                title="Kondisi Aset"
                description="Distribusi aset berdasarkan kondisi."
                icon="fas fa-heart-pulse"
            >
                {kondisi_view}
            </SectionCard>
            <SectionCard
                title="Kategori Aset"
                description="10 kategori terbanyak."
                icon="fas fa-layer-group"
            >
                {kategori_view}
            </SectionCard>
        </div>

        <SectionCard
            title="Top 10 Satker"
            description="Satker dengan aset terbanyak."
            icon="fas fa-trophy"
        >
            {top_satker_view}
        </SectionCard>

        <SectionCard
            title="Perolehan per Tahun"
            description="Jumlah aset menurut tahun perolehan."
            icon="fas fa-calendar-days"
        >
            {per_tahun_view}
        </SectionCard>
    }
}

fn render_kondisi(data: &BankAsetDashboard) -> impl IntoView + use<> {
    if data.kondisi_breakdown.is_empty() {
        return view! { <p class="py-6 text-center text-sm text-slate-500">"Belum ada data kondisi."</p> }
        .into_any();
    }
    let total: i64 = data.kondisi_breakdown.iter().map(|k| k.count).sum();
    let rows: Vec<_> = data
        .kondisi_breakdown
        .iter()
        .take(8)
        .map(|k| {
            let pct = if total > 0 {
                (k.count as f64) / (total as f64) * 100.0
            } else {
                0.0
            };
            let pct_label = format!("{:.1}%", pct);
            let width = format!("width: {:.1}%;", pct.clamp(2.0, 100.0));
            let kondisi = k.kondisi.clone();
            let count_label = format_thousands(k.count);
            view! {
                <div>
                    <div class="flex items-center justify-between text-xs">
                        <span class="font-medium text-slate-200">{kondisi}</span>
                        <span class="text-slate-400">{count_label} " (" {pct_label} ")"</span>
                    </div>
                    <div class="mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-white/[0.05]">
                        <div class="h-full rounded-full bg-gold-gradient" style=width></div>
                    </div>
                </div>
            }
        })
        .collect();
    view! { <div class="flex flex-col gap-3">{rows}</div> }.into_any()
}

fn render_kategori(data: &BankAsetDashboard) -> impl IntoView + use<> {
    if data.kategori_breakdown.is_empty() {
        return view! { <p class="py-6 text-center text-sm text-slate-500">"Belum ada data kategori."</p> }
        .into_any();
    }
    let rows: Vec<_> = data.kategori_breakdown.iter().take(10).map(|k| {
        let kategori = k.kategori.clone();
        let count = format_thousands(k.count);
        let nilai = format_rupiah(k.nilai);
        view! {
            <li class="flex items-center justify-between gap-3 rounded-lg border border-white/[0.05] bg-white/[0.03] px-3 py-2">
                <div class="min-w-0">
                    <p class="truncate text-sm font-medium text-slate-100">{kategori}</p>
                    <p class="mt-0.5 text-xs text-slate-500">{nilai}</p>
                </div>
                <span class="shrink-0 rounded-full bg-gold-500/10 px-2.5 py-1 text-xs font-semibold text-gold-300 ring-1 ring-gold-500/20">
                    {count}
                </span>
            </li>
        }
    }).collect();
    view! { <ul class="flex flex-col gap-2">{rows}</ul> }.into_any()
}

fn render_top_satker(data: &BankAsetDashboard) -> impl IntoView + use<> {
    if data.top_satker.is_empty() {
        return view! { <p class="py-6 text-center text-sm text-slate-500">"Belum ada data satker."</p> }
        .into_any();
    }
    let rows: Vec<_> = data
        .top_satker
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let rank = (i + 1).to_string();
            let satker = s.satker.clone();
            let count = format_thousands(s.count);
            let nilai = format_rupiah(s.nilai);
            view! {
                <tr class="border-b border-white/[0.04] last:border-0">
                    <td class="py-3 pr-3 text-xs font-semibold text-gold-300">{rank}</td>
                    <td class="py-3 pr-3 text-sm text-slate-100">{satker}</td>
                    <td class="py-3 pr-3 text-right text-sm text-slate-300">{count}</td>
                    <td class="py-3 text-right text-sm text-slate-300">{nilai}</td>
                </tr>
            }
        })
        .collect();
    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="border-b border-white/[0.08]">
                    <tr class="text-xs uppercase tracking-wide text-slate-500">
                        <th class="py-2 pr-3 text-left font-medium">"#"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Satker"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Jumlah Aset"</th>
                        <th class="py-2 text-right font-medium">"Nilai Perolehan"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}

fn render_per_tahun(data: &BankAsetDashboard) -> impl IntoView + use<> {
    if data.per_tahun.is_empty() {
        return view! { <p class="py-6 text-center text-sm text-slate-500">"Belum ada data tahun perolehan."</p> }
        .into_any();
    }
    let max = data
        .per_tahun
        .iter()
        .map(|t| t.count)
        .max()
        .unwrap_or(1)
        .max(1);
    let bars: Vec<_> = data
        .per_tahun
        .iter()
        .take(15)
        .map(|t| {
            let pct = (t.count as f64) / (max as f64) * 100.0;
            let height = format!("height: {:.1}%;", pct.max(4.0));
            let year = t.tahun.to_string();
            let count = format_thousands(t.count);
            view! {
                <div class="flex flex-col items-center gap-2">
                    <div class="flex h-28 w-full items-end">
                        <div class="w-full rounded-t bg-gold-gradient" style=height></div>
                    </div>
                    <span class="text-[0.65rem] font-semibold text-slate-200">{year}</span>
                    <span class="text-[0.6rem] text-slate-500">{count}</span>
                </div>
            }
        })
        .collect();
    view! {
        <div class="grid grid-cols-5 gap-3 sm:grid-cols-8 lg:grid-cols-10 xl:grid-cols-15">
            {bars}
        </div>
    }
    .into_any()
}

pub(super) fn format_thousands<N: Into<i128>>(n: N) -> String {
    let n: i128 = n.into();
    let s = n.abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(*b as char);
    }
    if n < 0 { format!("-{out}") } else { out }
}

pub(super) fn format_rupiah(n: f64) -> String {
    let truncated = n.round() as i128;
    format!("Rp {}", format_thousands(truncated))
}
