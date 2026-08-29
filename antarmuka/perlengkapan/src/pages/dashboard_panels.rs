//! Analytical panels for the perlengkapan landing dashboard.
//!
//! # Every figure here is a query result
//!
//! Nothing on this page is sample data, a placeholder series or a designed-in
//! example. Each panel names the endpoint it reads and renders exactly what
//! came back; where the backend has no such series, the panel is absent rather
//! than filled in with something plausible. Two sources, both scoped per tier
//! server-side:
//!
//! * `GET /bank-aset/dashboard`   — SIMAN asset posture (condition, taxonomy,
//!   acquisition value, per-year, per-satker).
//! * `GET /dashboard/perlengkapan?tahun_anggaran=N` — workflow and module
//!   figures (SLA, processing time, kebutuhan status, gap analysis, permit and
//!   removal counts).
//!
//! Empty is rendered as empty. A satker with no removal proposals reads "0",
//! and a series with no rows says so — a dashboard that invents a shape when
//! the data is missing is worse than one that admits it, because the reader
//! cannot tell the two apart.

use crate::api::bank_aset::BankAsetDashboard;
use crate::api::dashboard::PerlengkapanDashboardMetrics;
use leptos::prelude::*;
use lib_ui::components::charts::{
    DonutChart, HBarChart, Slice, TrendLine, format_id, format_rupiah_short,
};

/// Qualitative palette for the taxonomy series. Distinguishable at small size
/// and on a dark ground; category identity is also carried by the legend text,
/// so colour is never the only signal.
pub const SERIES: [&str; 7] = [
    "#60a5fa", "#34d399", "#fbbf24", "#f472b6", "#a78bfa", "#22d3ee", "#94a3b8",
];

/// Shell shared by every panel so the page reads as one surface.
#[component]
pub fn Panel(
    #[prop(into)] title: String,
    #[prop(into, optional)] subtitle: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="rounded-2xl border border-white/10 bg-slate-900/50 p-5 backdrop-blur">
            <header class="mb-4">
                <h3 class="text-sm font-bold uppercase tracking-[0.08em] text-slate-200">
                    {title}
                </h3>
                {subtitle.map(|s| view! { <p class="mt-1 text-xs text-slate-500">{s}</p> })}
            </header>
            {children()}
        </section>
    }
}

/// Asset count by SIMAN taxonomy (`jenis_aset`).
///
/// Long tail folded into "Lainnya" past the sixth slice: a donut with fifteen
/// wedges communicates less than one with seven, and the full breakdown is one
/// click away on Bank Aset.
#[component]
pub fn KomposisiAset(data: BankAsetDashboard) -> impl IntoView {
    let mut cats = data.kategori_breakdown.clone();
    cats.sort_by(|a, b| b.count.cmp(&a.count));
    let (head, tail) = cats.split_at(cats.len().min(6));
    let mut slices: Vec<Slice> = head
        .iter()
        .enumerate()
        .map(|(i, c)| Slice::new(c.kategori.clone(), c.count as f64, SERIES[i % SERIES.len()]))
        .collect();
    let rest: i64 = tail.iter().map(|c| c.count).sum();
    if rest > 0 {
        slices.push(Slice::new(
            format!("Lainnya ({} jenis)", tail.len()),
            rest as f64,
            SERIES[6],
        ));
    }

    view! {
        <Panel title="Komposisi Aset" subtitle="Jumlah unit per jenis BMN — sumber SIMAN">
            <DonutChart
                slices=slices
                center_value=format_id(data.total_aset as f64)
                center_label="unit"
            />
        </Panel>
    }
}

/// Acquisition value by taxonomy.
///
/// Deliberately a SECOND panel rather than a second column on the donut: count
/// and value rank differently and the gap is the finding. On the national set,
/// Tanah is 0,3% of units and a fifth of the value — a dashboard that shows
/// only counts hides where the money is.
#[component]
pub fn NilaiPerJenis(data: BankAsetDashboard) -> impl IntoView {
    let mut cats = data.kategori_breakdown.clone();
    cats.sort_by(|a, b| {
        b.nilai
            .partial_cmp(&a.nilai)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let slices: Vec<Slice> = cats
        .iter()
        .take(7)
        .enumerate()
        .map(|(i, c)| Slice::new(c.kategori.clone(), c.nilai, SERIES[i % SERIES.len()]))
        .collect();

    view! {
        <Panel
            title="Nilai Perolehan"
            subtitle=format!("Total {}", format_rupiah_short(data.total_nilai_perolehan))
        >
            <HBarChart slices=slices fmt=Callback::new(format_rupiah_short) />
        </Panel>
    }
}

/// Acquisitions per year, oldest first.
#[component]
pub fn TrenPerolehan(data: BankAsetDashboard) -> impl IntoView {
    let mut years = data.per_tahun.clone();
    years.sort_by_key(|t| t.tahun);
    let points: Vec<(String, f64)> = years
        .iter()
        .map(|t| (t.tahun.to_string(), t.count as f64))
        .collect();

    view! {
        <Panel title="Tren Perolehan Aset" subtitle="Jumlah unit menurut tahun perolehan">
            <TrendLine points=points color="#34d399".to_string() height=110 />
        </Panel>
    }
}

/// Requested-versus-held, per barang code, largest shortfall first.
///
/// A negative gap means the satker already holds more than it requested; it is
/// shown as a surplus rather than clamped to zero, because "you asked for 3 and
/// hold 1 903" is the more useful sentence.
#[component]
pub fn KesenjanganKebutuhan(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let rows = m.gap_analysis.clone();
    if rows.is_empty() {
        return view! {
            <Panel
                title="Kesenjangan Kebutuhan"
                subtitle="Selisih permintaan terhadap aset kondisi baik"
            >
                <p class="py-6 text-center text-sm text-slate-500">
                    "Belum ada permintaan barang pada tahun anggaran ini"
                </p>
            </Panel>
        }
        .into_any();
    }

    view! {
        <Panel
            title="Kesenjangan Kebutuhan"
            subtitle="Selisih permintaan terhadap aset kondisi baik yang sudah dimiliki"
        >
            <div class="-mx-2 overflow-x-auto">
                <table class="w-full min-w-[540px] text-sm">
                    <thead>
                        <tr class="border-b border-white/10 text-left text-xs uppercase tracking-wider text-slate-500">
                            <th class="px-2 pb-2 font-semibold">"Nama Barang"</th>
                            <th class="px-2 pb-2 text-right font-semibold">"Diminta"</th>
                            <th class="px-2 pb-2 text-right font-semibold">"Dimiliki"</th>
                            <th class="px-2 pb-2 text-right font-semibold">"Selisih"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows
                            .into_iter()
                            .map(|r| {
                                let surplus = r.gap < 0;
                                let gap_text = if surplus {
                                    format!("+{}", format_id((-r.gap) as f64))
                                } else {
                                    format_id(r.gap as f64)
                                };
                                let gap_class = if surplus {
                                    "px-2 py-2 text-right font-semibold text-emerald-300"
                                } else {
                                    "px-2 py-2 text-right font-semibold text-amber-300"
                                };
                                view! {
                                    <tr class="border-b border-white/5 last:border-0">
                                        <td class="px-2 py-2">
                                            <span class="text-slate-200">{r.nama_barang}</span>
                                            <span class="ml-2 text-xs text-slate-600">
                                                {r.kode_barang}
                                            </span>
                                        </td>
                                        <td class="px-2 py-2 text-right text-slate-300">
                                            {format_id(r.standard_quantity as f64)}
                                        </td>
                                        <td class="px-2 py-2 text-right text-slate-300">
                                            {format_id(r.existing_good_quantity as f64)}
                                        </td>
                                        <td class=gap_class>{gap_text}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
            <p class="mt-3 text-xs text-slate-600">
                "Selisih bertanda + berarti aset yang dimiliki melebihi yang diminta."
            </p>
        </Panel>
    }
        .into_any()
}

/// Kebutuhan campaign responses by workflow state.
#[component]
pub fn StatusKebutuhan(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let mut items: Vec<(String, i64)> = m
        .kebutuhan_metrics
        .total_by_status
        .clone()
        .into_iter()
        .collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    let slices: Vec<Slice> = items
        .iter()
        .enumerate()
        .map(|(i, (k, v))| Slice::new(k.clone(), *v as f64, SERIES[i % SERIES.len()]))
        .collect();

    view! {
        <Panel
            title="Status Pengajuan Kebutuhan"
            subtitle="Tanggapan per satuan kerja menurut tahap persetujuan"
        >
            <HBarChart slices=slices />
        </Panel>
    }
}

/// Which satkers hold the most assets. Hidden for a satker-tier caller, whose
/// answer is always a single row naming themselves.
#[component]
pub fn SatkerTeratas(data: BankAsetDashboard) -> impl IntoView {
    let slices: Vec<Slice> = data
        .top_satker
        .iter()
        .take(7)
        .enumerate()
        .map(|(i, s)| Slice::new(s.satker.clone(), s.count as f64, SERIES[i % SERIES.len()]))
        .collect();

    view! {
        <Panel title="Satuan Kerja Teratas" subtitle="Menurut jumlah aset BMN">
            <HBarChart slices=slices />
        </Panel>
    }
}

/// Where approvals are queueing, from `workflow_metrics.bottlenecks`.
#[component]
pub fn Hambatan(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let b = m.workflow_metrics.bottlenecks.clone();
    if b.is_empty() {
        return view! {
            <Panel title="Hambatan Alur" subtitle="Tahap dengan antrean terlama (30 hari terakhir)">
                <p class="py-6 text-center text-sm text-slate-500">
                    "Tidak ada tahap yang tertahan lebih dari 24 jam"
                </p>
            </Panel>
        }
        .into_any();
    }
    view! {
        <Panel title="Hambatan Alur" subtitle="Tahap dengan antrean terlama (30 hari terakhir)">
            <ul class="space-y-3">
                {b
                    .into_iter()
                    .map(|x| {
                        view! {
                            <li class="flex items-baseline justify-between gap-3 border-b border-white/5 pb-2 last:border-0 last:pb-0">
                                <span class="min-w-0 truncate text-sm text-slate-300">
                                    {x.state}
                                </span>
                                <span class="shrink-0 text-sm">
                                    <strong class="text-amber-300">
                                        {format!("{:.0} jam", x.average_time_hours)}
                                    </strong>
                                    <span class="ml-2 text-xs text-slate-500">
                                        {format!("{} berkas", x.count)}
                                    </span>
                                </span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </Panel>
    }
        .into_any()
}

/// Kebutuhan responses per satker. Multi-satker tiers only — a satker operator
/// gets one row naming themselves, which is not a chart.
#[component]
pub fn KebutuhanPerSatker(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let slices: Vec<Slice> = m
        .kebutuhan_metrics
        .total_by_satker
        .iter()
        .take(7)
        .enumerate()
        .map(|(i, s)| {
            Slice::new(
                if s.satker_nama.is_empty() {
                    s.satker_id.clone()
                } else {
                    s.satker_nama.clone()
                },
                s.count as f64,
                SERIES[i % SERIES.len()],
            )
        })
        .collect();

    view! {
        <Panel
            title="Kebutuhan per Satuan Kerja"
            subtitle="Jumlah tanggapan pada tahun anggaran berjalan"
        >
            <HBarChart slices=slices />
        </Panel>
    }
}

/// Requests per budget year — the backend keys this map by year, and JSON keys
/// are strings, so the year is parsed back rather than assumed.
#[component]
pub fn TrenKebutuhan(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let mut years: Vec<(i32, i64)> = m
        .kebutuhan_metrics
        .total_by_tahun
        .iter()
        .filter_map(|(k, v)| k.parse::<i32>().ok().map(|y| (y, *v)))
        .collect();
    years.sort_by_key(|(y, _)| *y);
    let points: Vec<(String, f64)> = years
        .into_iter()
        .map(|(y, c)| (y.to_string(), c as f64))
        .collect();

    view! {
        <Panel title="Tren Pengajuan Kebutuhan" subtitle="Tanggapan satuan kerja per tahun">
            <TrendLine points=points color="#fbbf24".to_string() height=110 />
        </Panel>
    }
}

/// Uniform sizes actually recorded, largest population first.
///
/// This is the figure procurement needs and it existed in the payload without
/// ever reaching a screen: how many of each size to order.
#[component]
pub fn DistribusiUkuran(m: PerlengkapanDashboardMetrics) -> impl IntoView {
    let mut items: Vec<(String, i64)> = m
        .pakaian_dinas_metrics
        .total_by_ukuran
        .clone()
        .into_iter()
        .collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    let slices: Vec<Slice> = items
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, (k, v))| Slice::new(k.clone(), *v as f64, SERIES[i % SERIES.len()]))
        .collect();

    let jenis: i64 = m.pakaian_dinas_metrics.total_by_jenis.values().sum();

    view! {
        <Panel
            title="Distribusi Ukuran Pakaian Dinas"
            subtitle=format!(
                "Jumlah pegawai per ukuran, dari {} kampanye tahun berjalan",
                format_id(jenis as f64),
            )
        >
            <HBarChart slices=slices />
        </Panel>
    }
}

/// Status rekap for one workflow module.
#[component]
pub fn StatusModul(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    #[prop(into)] statuses: std::collections::HashMap<String, i64>,
) -> impl IntoView {
    let mut items: Vec<(String, i64)> = statuses.into_iter().collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));
    let slices: Vec<Slice> = items
        .iter()
        .enumerate()
        .map(|(i, (k, v))| Slice::new(k.clone(), *v as f64, SERIES[i % SERIES.len()]))
        .collect();

    view! {
        <Panel title=title subtitle=subtitle>
            <HBarChart slices=slices />
        </Panel>
    }
}
