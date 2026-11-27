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
use chrono::Datelike;
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
    cats.sort_by_key(|c| std::cmp::Reverse(c.count));
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
    // The current year is not over: drawn like a finished one it falls off a cliff at the right edge.
    let berjalan = years.last().map(|t| t.tahun) == Some(crate::tahun::tahun_berjalan());

    view! {
        <Panel title="Tren Perolehan Aset" subtitle="Jumlah unit menurut tahun perolehan">
            <TrendLine
                points=points
                color="#34d399".to_string()
                height=110
                partial_last=berjalan
            />
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
    items.sort_by_key(|i| std::cmp::Reverse(i.1));
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
            <Panel title="Hambatan Alur" subtitle="Tahap yang sedang menahan berkas paling lama (rata-rata lama berada di tahap itu)">
                <p class="py-6 text-center text-sm text-slate-500">
                    "Tidak ada tahap yang tertahan lebih dari 24 jam"
                </p>
            </Panel>
        }
        .into_any();
    }
    view! {
        <Panel title="Hambatan Alur" subtitle="Tahap yang sedang menahan berkas paling lama (rata-rata lama berada di tahap itu)">
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
    // The current budget year is still collecting responses: set its point apart.
    let berjalan = years.last().map(|(y, _)| *y) == Some(crate::tahun::tahun_berjalan());
    let points: Vec<(String, f64)> = years
        .into_iter()
        .map(|(y, c)| (y.to_string(), c as f64))
        .collect();

    view! {
        <Panel title="Tren Pengajuan Kebutuhan" subtitle="Tanggapan satuan kerja per tahun">
            <TrendLine
                points=points
                color="#fbbf24".to_string()
                height=110
                partial_last=berjalan
            />
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
    // Equal counts fall back to the label: a HashMap iterates in a different order on every
    // request, so without it two sizes with the same count swapped places on each refresh.
    items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
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
    items.sort_by_key(|i| std::cmp::Reverse(i.1));
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

/// The drill-down controls, showing only what the caller's tier can act on.
///
/// - A **satker operator** has no region or satker to choose: their scope IS one satker. They keep
///   the filters that still mean something (jenis, tahun, periode) and a line saying whose figures
///   these are.
/// - A **wilayah validator** sees their whole wilayah by default and may narrow to one satker under
///   it; the region is fixed by their claims, so there is no region selector.
/// - **Pusat** sees the whole country by default and may narrow to one wilayah, to one satker, or
///   to a satker within a wilayah: choosing a wilayah narrows the satker list to that wilayah.
///
/// The options come from `/bank-aset/filter-options`, which is scoped server-side, so the lists
/// cannot offer what the reader may not see, and the server would refuse it anyway.
///
/// The lists arrive ONCE and whole. Each satker carries the region it sits in (`induk`), so
/// choosing a region narrows the satker list here, at once. It used to ask the server again, which
/// takes 2-3 s for pusat; for those seconds the old list still offered every satker in the country
/// and a pick from it answered zeros. The component also stays mounted while the figures reload,
/// and every selector's `value` PROPERTY is bound to its signal, so what the screen shows is what
/// is being filtered. (Binding the `selected` attribute of each option did not hold: once a person
/// has chosen an option by hand the browser stops following the attribute, and "Bersihkan filter"
/// then cleared the filter while the select kept showing the old choice.)
#[component]
pub fn FilterBar(
    options: crate::api::bank_aset::BankAsetFilterOptions,
    show_wilayah: bool,
    show_satker: bool,
    wilayah: RwSignal<Option<String>>,
    satker: RwSignal<Option<String>>,
    jenis: RwSignal<Option<String>>,
    tahun: RwSignal<i32>,
    tgl_from: RwSignal<Option<String>>,
    tgl_to: RwSignal<Option<String>>,
) -> impl IntoView {
    // Years offered run from the current one backwards; the campaign data this
    // filters is keyed by budget year, which cannot be in the future.
    let now = chrono::Local::now().year();
    let years: Vec<i32> = (now - 5..=now).rev().collect();

    // `w-full max-w-full` here and `min-w-0` on each wrapper: a flex item will not shrink below
    // its content, and a select's content is its longest option (a satker name). On a phone that
    // made the row 154px wider than its card, clipped on the right with no scrollbar.
    let sel = "w-full max-w-full rounded-lg border border-white/10 bg-slate-900/70 px-3 py-2 text-sm \
               text-slate-100 outline-none transition-colors hover:border-white/20 \
               focus:border-gold-400/50";
    let lab = "mb-1 block text-[11px] font-semibold uppercase tracking-wider text-slate-500";

    let has_any = move || {
        wilayah.get().is_some()
            || satker.get().is_some()
            || jenis.get().is_some()
            || tgl_from.get().is_some()
            || tgl_to.get().is_some()
    };

    let wilayah_opts = options.wilayah.clone();
    let satker_opts = options.satker_kode.clone();

    // Whose figures these are, in words. For the operator it is the one thing they cannot change,
    // so it is the only place the page says which satker it is showing.
    let ringkasan = {
        let wilayah_opts = wilayah_opts.clone();
        let satker_opts = satker_opts.clone();
        move || {
            let nama = |daftar: &[crate::api::bank_aset::FilterOption], kode: &str| {
                daftar
                    .iter()
                    .find(|o| o.value == kode)
                    .map(|o| o.display().to_string())
            };
            let w = wilayah.get().and_then(|k| nama(&wilayah_opts, &k));
            let s = satker.get().and_then(|k| nama(&satker_opts, &k));
            match (show_wilayah, show_satker) {
                // Pusat.
                (true, _) => match (w, s) {
                    (Some(w), Some(s)) => format!("{w} › {s}"),
                    (Some(w), None) => format!("{w} · seluruh satuan kerjanya"),
                    (None, Some(s)) => s,
                    (None, None) => "Seluruh Indonesia".to_string(),
                },
                // Wilayah validator: the wilayah is fixed, the satker is the choice.
                (false, true) => {
                    let wil = wilayah_opts
                        .iter()
                        .map(|o| o.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let wil = if wil.is_empty() {
                        "wilayah Anda".to_string()
                    } else {
                        wil
                    };
                    match s {
                        Some(s) => format!("{wil} › {s}"),
                        None => format!("{wil} · seluruh satuan kerjanya"),
                    }
                }
                // Operator: their own satker, always.
                (false, false) => satker_opts
                    .first()
                    .map(|o| o.display().to_string())
                    .unwrap_or_else(|| "Satuan kerja Anda".to_string()),
            }
        }
    };

    // The satker list for the chosen region (all of them when no region is chosen).
    let satker_tampil = {
        let satker_opts = satker_opts.clone();
        move || {
            let w = wilayah.get();
            satker_opts
                .iter()
                .filter(|o| match &w {
                    Some(w) => o.di_wilayah(w),
                    None => true,
                })
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    view! {
        <section class="rounded-2xl border border-white/10 bg-slate-900/40 p-4 backdrop-blur">
            <p
                data-testid="filter-ringkasan"
                class="mb-3 text-xs text-slate-400 sm:text-sm"
            >
                "Menampilkan: " <strong class="text-slate-100">{ringkasan}</strong>
            </p>
            <div class="flex flex-wrap items-end gap-3">
                {show_wilayah
                    .then(|| {
                        view! {
                    <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                        <label class=lab for="f-wilayah">
                            "Wilayah"
                        </label>
                        <select
                            id="f-wilayah"
                            prop:value=move || wilayah.get().unwrap_or_default()
                            class=sel
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                wilayah.set(if v.is_empty() { None } else { Some(v) });
                                // A satker of another wilayah is not a choice any more.
                                satker.set(None);
                            }
                        >
                            <option value="">
                                "Seluruh wilayah"
                            </option>
                            {wilayah_opts
                                .iter()
                                .map(|o| {
                                    let v = o.value.clone();
                                    let text = format!(
                                        "{} ({})",
                                        o.display(),
                                        format_id(o.count as f64),
                                    );
                                    view! {
                                        <option
                                            value=v
                                        >
                                            {text}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    </div>
                        }
                    })}

                {show_satker
                    .then(|| {
                        view! {
                    <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                        <label class=lab for="f-satker">
                            "Satuan Kerja"
                        </label>
                        <select
                            id="f-satker"
                            prop:value=move || satker.get().unwrap_or_default()
                            class=sel
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                satker.set(if v.is_empty() { None } else { Some(v) });
                            }
                        >
                            <option value="">
                                "Seluruh satuan kerja"
                            </option>
                            {move || {
                                satker_tampil()
                                    .into_iter()
                                    .map(|o| {
                                        let v = o.value.clone();
                                        let text = format!(
                                            "{} ({})",
                                            o.display(),
                                            format_id(o.count as f64),
                                        );
                                        view! {
                                            <option
                                                value=v
                                            >
                                                {text}
                                            </option>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </select>
                    </div>
                        }
                    })}

                <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                    <label class=lab for="f-jenis">
                        "Jenis Aset"
                    </label>
                    <select
                        id="f-jenis"
                            prop:value=move || jenis.get().unwrap_or_default()
                        class=sel
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            jenis.set(if v.is_empty() { None } else { Some(v) });
                        }
                    >
                        <option value="">
                            "Seluruh jenis"
                        </option>
                        {options
                            .jenis
                            .iter()
                            .map(|o| {
                                let v = o.value.clone();
                                let text = format!(
                                    "{} ({})",
                                    o.display(),
                                    format_id(o.count as f64),
                                );
                                view! {
                                    <option
                                        value=v
                                    >
                                        {text}
                                    </option>
                                }
                            })
                            .collect_view()}
                    </select>
                </div>

                <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                    <label class=lab for="f-tahun">
                        "Tahun Anggaran"
                    </label>
                    <select
                        id="f-tahun"
                        class=sel
                        on:change=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                tahun.set(v);
                            }
                        }
                    >
                        {years
                            .into_iter()
                            .map(|y| {
                                view! {
                                    <option value=y.to_string() selected=move || tahun.get() == y>
                                        {y.to_string()}
                                    </option>
                                }
                            })
                            .collect_view()}
                    </select>
                </div>

                <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                    <label class=lab for="f-from">
                        "Perolehan Dari"
                    </label>
                    <input
                        id="f-from"
                        type="date"
                        class=sel
                        prop:value=move || tgl_from.get().unwrap_or_default()
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            tgl_from.set(if v.is_empty() { None } else { Some(v) });
                        }
                    />
                </div>
                <div class="w-full min-w-0 sm:w-auto sm:max-w-full">
                    <label class=lab for="f-to">
                        "Sampai"
                    </label>
                    <input
                        id="f-to"
                        type="date"
                        class=sel
                        prop:value=move || tgl_to.get().unwrap_or_default()
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            tgl_to.set(if v.is_empty() { None } else { Some(v) });
                        }
                    />
                </div>

                <Show when=has_any>
                    <button
                        type="button"
                        data-testid="filter-bersihkan"
                        class="rounded-lg border border-white/10 px-3 py-2 text-sm font-semibold text-slate-300 transition-colors hover:border-white/25 hover:text-white"
                        on:click=move |_| {
                            wilayah.set(None);
                            satker.set(None);
                            jenis.set(None);
                            tgl_from.set(None);
                            tgl_to.set(None);
                        }
                    >
                        "Bersihkan filter"
                    </button>
                </Show>
            </div>
        </section>
    }
}
