//! Dashboard widgets: MetricCard, Charts, GapAnalysisTable
//!
//! Production-ready dashboard components for SIMPEL analytics and visualization.

use crate::core::types::*;
use leptos::prelude::*;

// ============================================================================
// METRIC CARD COMPONENT
// ============================================================================

/// Metric card for displaying key performance indicators
#[component]
pub fn MetricCard(
    /// Card title
    #[prop(into)]
    title: String,
    /// Main value to display
    #[prop(into)]
    value: String,
    /// Optional change percentage (positive or negative)
    #[prop(optional)]
    change: Option<f64>,
    /// Optional icon (emoji or SVG)
    #[prop(optional, into)]
    icon: Option<String>,
    /// Optional subtitle/description
    #[prop(optional, into)]
    subtitle: Option<String>,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let (change_class, change_icon) = if let Some(c) = change {
        if c >= 0.0 {
            ("text-green-600 dark:text-green-400", "↑")
        } else {
            ("text-red-600 dark:text-red-400", "↓")
        }
    } else {
        ("", "")
    };

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 {}",
            class
        )>
            <div class="flex items-center justify-between">
                <div class="flex-1">
                    <p class="text-sm font-medium text-gray-600 dark:text-gray-400 mb-1">
                        {title}
                    </p>
                    <p class="text-3xl font-bold text-gray-900 dark:text-gray-100">
                        {value}
                    </p>

                    {subtitle.map(|s| view! {
                        <p class="text-xs text-gray-500 dark:text-gray-500 mt-1">
                            {s}
                        </p>
                    })}

                    {change.map(|c| view! {
                        <div class=format!("flex items-center mt-2 text-sm font-medium {}", change_class)>
                            <span class="mr-1">{change_icon}</span>
                            <span>{format!("{:.1}%", c.abs())}</span>
                            <span class="ml-1 text-gray-500 dark:text-gray-400 font-normal">
                                "vs periode sebelumnya"
                            </span>
                        </div>
                    })}
                </div>

                {icon.map(|i| view! {
                    <div class="flex-shrink-0 ml-4">
                        <div class="text-4xl text-gray-400 dark:text-gray-500">
                            {i}
                        </div>
                    </div>
                })}
            </div>
        </div>
    }
}

// ============================================================================
// BAR CHART COMPONENT
// ============================================================================

/// Simple bar chart component for data visualization
#[component]
pub fn BarChart(
    /// Chart title
    #[prop(into)]
    title: String,
    /// Data points with labels and values
    #[prop(into)]
    data: Vec<ChartDataPoint>,
    /// Maximum value for scaling (auto-calculated if None)
    #[prop(optional)]
    max_value: Option<f64>,
    /// Show values on bars
    #[prop(default = true)]
    show_values: bool,
    /// Chart height in pixels
    #[prop(default = 300)]
    height: u32,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let max = max_value.unwrap_or_else(|| {
        data.iter().map(|d| d.value).fold(0.0, f64::max).max(1.0) // Prevent division by zero
    });

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 {}",
            class
        )>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                {title}
            </h3>

            <div class="space-y-3" style=format!("height: {}px", height)>
                {data.clone().into_iter().map(|point| {
                    let percentage = (point.value / max * 100.0).min(100.0);
                    let bar_color = point.color.unwrap_or_else(|| "bg-emerald-500".to_string());

                    view! {
                        <div class="flex items-center space-x-3">
                            <div class="w-24 text-sm text-gray-700 dark:text-gray-300 truncate">
                                {point.label}
                            </div>
                            <div class="flex-1 relative">
                                <div class="h-8 bg-gray-200 dark:bg-gray-700 rounded-md overflow-hidden">
                                    <div
                                        class=format!("{} h-full rounded-md transition-all duration-500", bar_color)
                                        style=format!("width: {}%", percentage)
                                    />
                                </div>
                                {show_values.then(|| view! {
                                    <span class="absolute right-2 top-1/2 -translate-y-1/2 text-xs font-medium text-gray-700 dark:text-gray-300">
                                        {format!("{:.0}", point.value)}
                                    </span>
                                })}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            {data.is_empty().then(|| view! {
                <div class="flex items-center justify-center h-full text-gray-500 dark:text-gray-400">
                    <p>"Tidak ada data untuk ditampilkan"</p>
                </div>
            })}
        </div>
    }
}

// ============================================================================
// PIE CHART COMPONENT
// ============================================================================

/// Simple pie chart component using CSS conic-gradient
#[component]
pub fn PieChart(
    /// Chart title
    #[prop(into)]
    title: String,
    /// Data points with labels, values, and colors
    #[prop(into)]
    data: Vec<ChartDataPoint>,
    /// Show legend
    #[prop(default = true)]
    show_legend: bool,
    /// Chart size in pixels
    #[prop(default = 200)]
    size: u32,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    let total: f64 = data.iter().map(|d| d.value).sum();

    // Calculate percentages and cumulative angles for conic-gradient
    let mut cumulative = 0.0;
    let segments: Vec<_> = data
        .iter()
        .map(|point| {
            let percentage = if total > 0.0 {
                (point.value / total) * 100.0
            } else {
                0.0
            };
            let start = cumulative;
            cumulative += percentage;
            (point.clone(), percentage, start, cumulative)
        })
        .collect();

    // Build conic-gradient CSS
    let gradient = segments
        .iter()
        .map(|(point, _, start, end)| {
            let color = point.color.as_deref().unwrap_or("gray");
            format!("{} {}% {}%", color, start, end)
        })
        .collect::<Vec<_>>()
        .join(", ");

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 {}",
            class
        )>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                {title}
            </h3>

            <div class="flex flex-col md:flex-row items-center justify-center space-y-4 md:space-y-0 md:space-x-8">
                // Pie chart
                <div
                    class="rounded-full shadow-lg"
                    style=format!(
                        "width: {}px; height: {}px; background: conic-gradient({})",
                        size, size, gradient
                    )
                />

                // Legend
                {show_legend.then(|| view! {
                    <div class="space-y-2">
                        {segments.into_iter().map(|(point, percentage, _, _)| {
                            let color = point.color.unwrap_or_else(|| "bg-gray-500".to_string());
                            view! {
                                <div class="flex items-center space-x-2">
                                    <div class=format!("w-4 h-4 rounded {}", color) />
                                    <span class="text-sm text-gray-700 dark:text-gray-300">
                                        {format!("{}: {:.1}%", point.label, percentage)}
                                    </span>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                })}
            </div>

            {data.is_empty().then(|| view! {
                <div class="flex items-center justify-center h-32 text-gray-500 dark:text-gray-400">
                    <p>"Tidak ada data untuk ditampilkan"</p>
                </div>
            })}
        </div>
    }
}

// ============================================================================
// LINE CHART COMPONENT
// ============================================================================

/// Simple line chart component using SVG
#[component]
pub fn LineChart(
    /// Chart title
    #[prop(into)]
    title: String,
    /// Data points with labels and values
    #[prop(into)]
    data: Vec<ChartDataPoint>,
    /// Chart height in pixels
    #[prop(default = 300)]
    height: u32,
    /// Show data points
    #[prop(default = true)]
    show_points: bool,
    /// Show grid lines
    #[prop(default = true)]
    show_grid: bool,
    /// Line color
    #[prop(default = "stroke-emerald-500".to_string(), into)]
    line_color: String,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();

    if data.is_empty() {
        return view! {
            <div class=format!(
                "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 {}",
                class
            )>
                <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                    {title}
                </h3>
                <div class="flex items-center justify-center h-32 text-gray-500 dark:text-gray-400">
                    <p>"Tidak ada data untuk ditampilkan"</p>
                </div>
            </div>
        }
        .into_any();
    }

    let max_value = data.iter().map(|d| d.value).fold(0.0, f64::max).max(1.0);
    let min_value = data
        .iter()
        .map(|d| d.value)
        .fold(f64::MAX, f64::min)
        .min(0.0);
    let value_range = max_value - min_value;

    let width = 600;
    let padding = 40;
    let chart_width = width - 2 * padding;
    let chart_height = height - 2 * padding;

    // Calculate points
    let points: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let x =
                padding as f64 + (i as f64 / (data.len() - 1).max(1) as f64) * chart_width as f64;
            let y = padding as f64 + chart_height as f64
                - ((point.value - min_value) / value_range.max(1.0)) * chart_height as f64;
            (x, y)
        })
        .collect();

    // Build path
    let path_d = points
        .iter()
        .enumerate()
        .map(|(i, (x, y))| {
            if i == 0 {
                format!("M {} {}", x, y)
            } else {
                format!("L {} {}", x, y)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 border border-gray-200 dark:border-gray-700 {}",
            class
        )>
            <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
                {title}
            </h3>

            <svg
                width=width
                height=height
                viewBox=format!("0 0 {} {}", width, height)
                class="w-full"
            >
                // Grid lines
                {show_grid.then(|| {

                    (0..5)
                        .map(|i| {
                            let y = padding + (chart_height / 4) * i;
                            view! {
                                <line
                                    x1=format!("{}", padding)
                                    y1=format!("{}", y)
                                    x2=format!("{}", width - padding)
                                    y2=format!("{}", y)
                                    stroke="currentColor"
                                    stroke-width="1"
                                    class="stroke-gray-200 dark:stroke-gray-700"
                                    stroke-dasharray="4"
                                />
                            }
                        })
                        .collect_view()
                })}

                // Line path
                <path
                    d=path_d
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    class=line_color
                />

                // Data points
                {show_points.then(|| {
                    points
                        .iter()
                        .map(|(x, y)| {
                            view! {
                                <circle
                                    cx=format!("{}", x)
                                    cy=format!("{}", y)
                                    r="4"
                                    fill="currentColor"
                                    class="fill-emerald-500"
                                />
                            }
                        })
                        .collect_view()
                })}

                // X-axis labels
                {data.iter().enumerate().map(|(i, point)| {
                    let x = padding as f64 + (i as f64 / (data.len() - 1).max(1) as f64) * chart_width as f64;
                    view! {
                        <text
                            x=format!("{}", x)
                            y=format!("{}", height - 10)
                            text-anchor="middle"
                            class="text-xs fill-gray-600 dark:fill-gray-400"
                        >
                            {point.label.clone()}
                        </text>
                    }
                }).collect_view()}
            </svg>
        </div>
    }
    .into_any()
}

// ============================================================================
// GAP ANALYSIS TABLE COMPONENT
// ============================================================================

/// Gap analysis table for displaying BMN requirements vs existing assets
#[component]
pub fn GapAnalysisTable(
    /// Gap analysis data
    #[prop(into)]
    data: Vec<GapAnalysisRow>,
    /// Show satker column
    #[prop(default = true)]
    show_satker: bool,
    /// Enable sorting
    #[prop(default = true)]
    sortable: bool,
    /// Additional CSS classes
    #[prop(optional, into)]
    class: Option<String>,
) -> impl IntoView {
    let class = class.unwrap_or_default();
    let is_empty = data.is_empty();

    let (sort_column, set_sort_column) = signal::<Option<String>>(None);
    let (sort_direction, set_sort_direction) = signal(SortDirection::Asc);

    // Sort data
    let sorted_data = move || {
        let mut data_clone = data.clone();
        if let Some(col) = sort_column.get() {
            data_clone.sort_by(|a, b| {
                let ordering = match col.as_str() {
                    "kode_barang" => a.kode_barang.cmp(&b.kode_barang),
                    "nama_barang" => a.nama_barang.cmp(&b.nama_barang),
                    "satker" => a.satker_name.cmp(&b.satker_name),
                    "standard" => a.standard_quantity.cmp(&b.standard_quantity),
                    "existing" => a.existing_quantity.cmp(&b.existing_quantity),
                    "gap" => a.gap.cmp(&b.gap),
                    _ => std::cmp::Ordering::Equal,
                };
                match sort_direction.get() {
                    SortDirection::Asc => ordering,
                    SortDirection::Desc => ordering.reverse(),
                }
            });
        }
        data_clone
    };

    let handle_sort = move |column: String| {
        if sortable {
            if sort_column.get().as_ref() == Some(&column) {
                // Toggle direction
                set_sort_direction.update(|d| {
                    *d = match d {
                        SortDirection::Asc => SortDirection::Desc,
                        SortDirection::Desc => SortDirection::Asc,
                    }
                });
            } else {
                set_sort_column.set(Some(column));
                set_sort_direction.set(SortDirection::Asc);
            }
        }
    };

    let sort_icon = move |column: &str| {
        if sort_column.get().as_ref() == Some(&column.to_string()) {
            match sort_direction.get() {
                SortDirection::Asc => "↑",
                SortDirection::Desc => "↓",
            }
        } else {
            ""
        }
    };

    view! {
        <div class=format!(
            "bg-white dark:bg-gray-800 rounded-lg shadow-md border border-gray-200 dark:border-gray-700 overflow-hidden {}",
            class
        )>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                    <thead class="bg-gray-50 dark:bg-gray-900">
                        <tr>
                            <th
                                scope="col"
                                class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                on:click=move |_| handle_sort("kode_barang".to_string())
                            >
                                "Kode Barang " {sort_icon("kode_barang")}
                            </th>
                            <th
                                scope="col"
                                class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                on:click=move |_| handle_sort("nama_barang".to_string())
                            >
                                "Nama Barang " {sort_icon("nama_barang")}
                            </th>
                            {show_satker.then(|| view! {
                                <th
                                    scope="col"
                                    class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                    on:click=move |_| handle_sort("satker".to_string())
                                >
                                    "Satker " {sort_icon("satker")}
                                </th>
                            })}
                            <th
                                scope="col"
                                class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                on:click=move |_| handle_sort("standard".to_string())
                            >
                                "Standar " {sort_icon("standard")}
                            </th>
                            <th
                                scope="col"
                                class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                on:click=move |_| handle_sort("existing".to_string())
                            >
                                "Existing " {sort_icon("existing")}
                            </th>
                            <th
                                scope="col"
                                class="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800"
                                on:click=move |_| handle_sort("gap".to_string())
                            >
                                "Gap " {sort_icon("gap")}
                            </th>
                            <th
                                scope="col"
                                class="px-6 py-3 text-center text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider"
                            >
                                "Status"
                            </th>
                        </tr>
                    </thead>
                    <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                        {move || sorted_data().into_iter().map(|row| {
                            let gap_class = if row.gap > 0 {
                                "text-red-600 dark:text-red-400 font-semibold"
                            } else {
                                "text-green-600 dark:text-green-400"
                            };

                            let status_badge = if row.gap > 0 {
                                view! {
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300">
                                        "Kurang"
                                    </span>
                                }
                            } else {
                                view! {
                                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">
                                        "Terpenuhi"
                                    </span>
                                }
                            };

                            view! {
                                <tr class="hover:bg-gray-50 dark:hover:bg-gray-800">
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900 dark:text-gray-100">
                                        {row.kode_barang}
                                    </td>
                                    <td class="px-6 py-4 text-sm text-gray-700 dark:text-gray-300">
                                        {row.nama_barang}
                                    </td>
                                    {show_satker.then(|| view! {
                                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700 dark:text-gray-300">
                                            {row.satker_name}
                                        </td>
                                    })}
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 dark:text-gray-100">
                                        {row.standard_quantity}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900 dark:text-gray-100">
                                        {row.existing_quantity}
                                    </td>
                                    <td class=format!("px-6 py-4 whitespace-nowrap text-sm text-right {}", gap_class)>
                                        {row.gap}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-center">
                                        {status_badge}
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>

                {is_empty.then(|| view! {
                    <div class="text-center py-12 text-gray-500 dark:text-gray-400">
                        <p>"Tidak ada data gap analysis"</p>
                    </div>
                })}
            </div>
        </div>
    }
}
