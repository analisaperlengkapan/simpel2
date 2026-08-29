//! Chart primitives: plain SVG, no chart dependency, no theme assumptions.
//!
//! # Why hand-rolled SVG rather than a chart crate
//!
//! These render four shapes — an arc, a bar, a line and a label. A charting
//! library buys axis autoscaling, interaction and layout engines that a
//! dashboard summary does not use, at the cost of a WASM payload every user
//! downloads. `leptos-chartistry` stays available (see [`super::chartistry`])
//! for the day a page genuinely needs interactive plotting.
//!
//! # Why no colours are baked in
//!
//! The previous in-house widgets (`components::dashboard`) hard-coded
//! `bg-white dark:bg-gray-800`, which is why they could not be used by the one
//! frontend that wanted charts: its design system is dark navy and gold, so
//! every widget would have arrived as a white card in the middle of it. They
//! sat unused for that reason. Here the container class and the series colours
//! are props, so the call site owns the palette and one implementation serves
//! both frontends.
//!
//! # Accessibility
//!
//! Colour is never the only carrier of meaning: every series is also named and
//! its value written out in the legend or beside the bar. Each chart exposes
//! `role="img"` with an `aria-label` summarising the whole series, so a screen
//! reader gets the numbers rather than "graphic".

use leptos::prelude::*;

/// One labelled, coloured measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct Slice {
    pub label: String,
    pub value: f64,
    /// Any CSS colour. Not a Tailwind class: these land in SVG paint
    /// attributes, which Tailwind's class scanner cannot reach anyway.
    pub color: String,
}

impl Slice {
    pub fn new(label: impl Into<String>, value: f64, color: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value,
            color: color.into(),
        }
    }
}

/// Group thousands with a dot, the Indonesian convention.
pub fn format_id(n: f64) -> String {
    let neg = n < 0.0;
    let mut s = String::new();
    for (i, ch) in (n.abs().round() as i64)
        .to_string()
        .chars()
        .rev()
        .enumerate()
    {
        if i > 0 && i % 3 == 0 {
            s.push('.');
        }
        s.push(ch);
    }
    let mut out: String = s.chars().rev().collect();
    if neg {
        out.insert(0, '-');
    }
    out
}

/// Rupiah, abbreviated to the largest sensible unit.
///
/// A dashboard tile cannot show `81.937.885.460.397` legibly, and rounding to
/// "Rp 81,9 T" is what a reader actually needs from a summary. The exact figure
/// stays available on the detail pages.
pub fn format_rupiah_short(v: f64) -> String {
    let a = v.abs();
    let (div, unit) = if a >= 1e12 {
        (1e12, "T")
    } else if a >= 1e9 {
        (1e9, "M")
    } else if a >= 1e6 {
        (1e6, "Jt")
    } else {
        (1.0, "")
    };
    if unit.is_empty() {
        format!("Rp {}", format_id(v))
    } else {
        format!("Rp {:.1} {}", v / div, unit).replace('.', ",")
    }
}

/// Donut chart: proportions of a whole, with a named + numbered legend.
///
/// Drawn with `stroke-dasharray` on concentric circles rather than arc paths —
/// the same picture with none of the large-arc-flag arithmetic, and it degrades
/// correctly when one slice is the entire total.
#[component]
pub fn DonutChart(
    #[prop(into)] slices: Vec<Slice>,
    /// Rendered in the hole. Usually the total.
    #[prop(into, optional)]
    center_value: Option<String>,
    #[prop(into, optional)] center_label: Option<String>,
    #[prop(default = 168)] size: u32,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let total: f64 = slices.iter().map(|s| s.value.max(0.0)).sum();
    let r = 70.0_f64;
    let circumference = 2.0 * std::f64::consts::PI * r;

    let aria = if total <= 0.0 {
        "Tidak ada data".to_string()
    } else {
        slices
            .iter()
            .map(|s| {
                format!(
                    "{}: {} ({:.0}%)",
                    s.label,
                    format_id(s.value),
                    s.value / total * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Pre-compute each ring's dash geometry so the view stays declarative.
    let mut acc = 0.0_f64;
    let rings: Vec<(String, f64, f64)> = slices
        .iter()
        .map(|s| {
            let frac = if total > 0.0 {
                s.value.max(0.0) / total
            } else {
                0.0
            };
            let len = frac * circumference;
            let offset = -acc * circumference;
            acc += frac;
            (s.color.clone(), len, offset)
        })
        .collect();

    let legend: Vec<(String, String, String, String)> = slices
        .iter()
        .map(|s| {
            let pct = if total > 0.0 {
                format!("{:.0}%", s.value / total * 100.0)
            } else {
                "0%".to_string()
            };
            (s.color.clone(), s.label.clone(), format_id(s.value), pct)
        })
        .collect();

    view! {
        <div class=format!("flex flex-col items-center gap-4 sm:flex-row sm:items-start {class}")>
            <svg
                viewBox="0 0 180 180"
                width=size
                height=size
                role="img"
                aria-label=aria
                class="shrink-0"
            >
                <circle
                    cx="90"
                    cy="90"
                    r="70"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="18"
                    class="text-white/5"
                />
                {rings
                    .into_iter()
                    .map(|(color, len, offset)| {
                        view! {
                            <circle
                                cx="90"
                                cy="90"
                                r="70"
                                fill="none"
                                stroke=color
                                stroke-width="18"
                                stroke-dasharray=format!("{len} {}", circumference - len)
                                stroke-dashoffset=offset.to_string()
                                transform="rotate(-90 90 90)"
                                stroke-linecap="butt"
                            />
                        }
                    })
                    .collect_view()}
                {center_value
                    .map(|v| {
                        view! {
                            <text
                                x="90"
                                y="88"
                                text-anchor="middle"
                                class="fill-white text-[22px] font-extrabold"
                            >
                                {v}
                            </text>
                        }
                    })}
                {center_label
                    .map(|l| {
                        view! {
                            <text
                                x="90"
                                y="107"
                                text-anchor="middle"
                                class="fill-slate-400 text-[10px] uppercase tracking-wider"
                            >
                                {l}
                            </text>
                        }
                    })}
            </svg>

            <ul class="w-full space-y-2">
                {legend
                    .into_iter()
                    .map(|(color, label, value, pct)| {
                        view! {
                            <li class="flex items-center gap-3 text-sm">
                                <span
                                    class="h-2.5 w-2.5 shrink-0 rounded-sm"
                                    style=format!("background-color:{color}")
                                ></span>
                                <span class="min-w-0 flex-1 truncate text-slate-300">{label}</span>
                                <span class="font-semibold text-white">{value}</span>
                                <span class="w-10 text-right text-xs text-slate-500">{pct}</span>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}

/// Horizontal bars, each labelled and captioned with its own value.
///
/// Horizontal rather than vertical on purpose: the categories here are BMN
/// taxonomy names ("Peralatan Mesin Khusus TIK"), which are unreadable rotated
/// under a vertical axis.
#[component]
pub fn HBarChart(
    #[prop(into)] slices: Vec<Slice>,
    /// Format each value for display. Defaults to grouped digits.
    #[prop(optional)]
    fmt: Option<Callback<f64, String>>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let max = slices.iter().map(|s| s.value).fold(0.0_f64, f64::max);
    let render = move |v: f64| match &fmt {
        Some(f) => f.run(v),
        None => format_id(v),
    };

    let aria = slices
        .iter()
        .map(|s| format!("{}: {}", s.label, render(s.value)))
        .collect::<Vec<_>>()
        .join(", ");

    if slices.is_empty() {
        return view! { <p class="py-6 text-center text-sm text-slate-500">"Belum ada data"</p> }
            .into_any();
    }

    view! {
        <div class=format!("space-y-3 {class}") role="img" aria-label=aria>
            {slices
                .into_iter()
                .map(|s| {
                    let pct = if max > 0.0 { (s.value / max * 100.0).max(1.5) } else { 0.0 };
                    let shown = render(s.value);
                    view! {
                        <div>
                            <div class="mb-1 flex items-baseline justify-between gap-3 text-sm">
                                <span class="min-w-0 truncate text-slate-300">{s.label}</span>
                                <span class="shrink-0 font-semibold text-white">{shown}</span>
                            </div>
                            <div class="h-2 w-full overflow-hidden rounded-full bg-white/5">
                                <div
                                    class="h-full rounded-full transition-all duration-500"
                                    style=format!("width:{pct:.1}%;background-color:{}", s.color)
                                ></div>
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

/// A single ordered series over time, as an area + line + endpoint dots.
#[component]
pub fn TrendLine(
    /// `(label, value)` in display order, oldest first.
    #[prop(into)]
    points: Vec<(String, f64)>,
    #[prop(into, default = "#60a5fa".to_string())] color: String,
    #[prop(default = 64)] height: u32,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    if points.len() < 2 {
        return view! {
            <p class="py-6 text-center text-sm text-slate-500">
                "Belum cukup data untuk menampilkan tren"
            </p>
        }
        .into_any();
    }

    let max = points.iter().map(|p| p.1).fold(0.0_f64, f64::max).max(1.0);
    let w = 100.0_f64;
    let h = 30.0_f64;
    let step = w / (points.len() - 1) as f64;
    let xy: Vec<(f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(i, (_, v))| (i as f64 * step, h - (v / max) * h))
        .collect();

    let line: String = xy
        .iter()
        .map(|(x, y)| format!("{x:.2},{y:.2}"))
        .collect::<Vec<_>>()
        .join(" ");
    let area = format!("{line} {w:.2},{h:.2} 0,{h:.2}");

    let aria = points
        .iter()
        .map(|(l, v)| format!("{l}: {}", format_id(*v)))
        .collect::<Vec<_>>()
        .join(", ");
    let labels: Vec<String> = points.iter().map(|(l, _)| l.clone()).collect();
    let last = points
        .last()
        .map(|(_, v)| format_id(*v))
        .unwrap_or_default();

    view! {
        <div class=class>
            <svg
                viewBox=format!("0 0 {w} {h}")
                preserveAspectRatio="none"
                height=height
                width="100%"
                role="img"
                aria-label=aria
            >
                <polygon points=area fill=color.clone() opacity="0.16" />
                <polyline
                    points=line
                    fill="none"
                    stroke=color.clone()
                    stroke-width="1.2"
                    vector-effect="non-scaling-stroke"
                    stroke-linejoin="round"
                />
                {xy
                    .last()
                    .map(|(x, y)| {
                        view! { <circle cx=*x cy=*y r="1.6" fill=color.clone() /> }
                    })}
            </svg>
            <div class="mt-1 flex justify-between text-[11px] text-slate-500">
                {labels.into_iter().map(|l| view! { <span>{l}</span> }).collect_view()}
            </div>
            <p class="sr-only">{format!("Nilai terakhir: {last}")}</p>
        </div>
    }
    .into_any()
}
