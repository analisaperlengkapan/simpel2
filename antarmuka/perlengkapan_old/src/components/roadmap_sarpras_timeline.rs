//! Predictive Analytics Dashboard Component
//!
//! Displays historical kebutuhan BMN data alongside forecasted predictions
//! with confidence intervals, trend indicators, and export capabilities.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── API response types ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YearlyData {
    pub tahun: i32,
    pub total_kebutuhan: i64,
    pub total_existing: i64,
    pub total_gap: i64,
    pub jumlah_satker: i64,
    pub estimasi_total_biaya: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedYear {
    pub tahun: i32,
    pub predicted_kebutuhan: f64,
    pub predicted_gap: f64,
    pub predicted_biaya: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    pub method: String,
    pub confidence_level: f64,
    pub historical: Vec<YearlyData>,
    pub predictions: Vec<PredictedYear>,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSummary {
    pub total_historical_years: i32,
    pub total_predicted_years: i32,
    pub avg_yearly_growth_pct: f64,
    pub total_predicted_kebutuhan: f64,
    pub total_predicted_biaya: f64,
    pub trend_direction: String,
}

// ── API calls ────────────────────────────────────────────────────

async fn fetch_forecast(method: &str) -> Result<ForecastResult, String> {
    let url = format!(
        "/api/pembinaan/perlengkapan/forecast?method={}&horizon=5&confidence=0.95",
        method
    );
    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<ForecastResult>()
        .await
        .map_err(|e| format!("JSON error: {}", e))
}

async fn fetch_summary() -> Result<ForecastSummary, String> {
    let resp = gloo_net::http::Request::get("/api/pembinaan/perlengkapan/forecast/summary")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    if !resp.ok() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.json::<ForecastSummary>()
        .await
        .map_err(|e| format!("JSON error: {}", e))
}

fn download_export() {
    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url("/api/pembinaan/perlengkapan/forecast/export?format=csv");
    }
}

// ── Main Dashboard Component ─────────────────────────────────────

#[component]
pub fn RoadmapSarprasTimeline() -> impl IntoView {
    let (method, set_method) = signal("sma".to_string());

    let forecast = LocalResource::new(move || {
        let m = method.get();
        async move { fetch_forecast(&m).await }
    });

    let summary = LocalResource::new(|| async move { fetch_summary().await });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            // Header
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-800">"Predictive Analytics — Sarpras"</h1>
                    <p class="text-sm text-gray-500 mt-1">
                        "Prediksi kebutuhan BMN berdasarkan data historis"
                    </p>
                </div>
                <div class="flex gap-2 items-center">
                    <select
                        class="border border-gray-300 rounded-lg px-3 py-2 text-sm"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_method.set(val);
                        }
                    >
                        <option value="sma" selected>"Simple Moving Average"</option>
                        <option value="wma">"Weighted Moving Average"</option>
                        <option value="exponential">"Exponential Smoothing"</option>
                    </select>
                    <button
                        class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 text-sm font-medium"
                        on:click=move |_| download_export()
                    >
                        "📥 Export CSV"
                    </button>
                </div>
            </div>

            // Summary Cards
            <Suspense fallback=move || view! { <div class="h-24 animate-pulse bg-gray-100 rounded-lg"></div> }>
                {move || summary.get().map(|res| match res {
                    Ok(s) => view! { <SummaryCards summary=s /> }.into_any(),
                    Err(_) => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            // Forecast Data
            <Suspense fallback=move || view! {
                <div class="flex justify-center items-center py-16">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            }>
                {move || forecast.get().map(|res| match res {
                    Ok(data) => view! {
                        <div>
                            <HistoricalTable data=data.historical.clone() />
                            <PredictionTable
                                predictions=data.predictions.clone()
                                method=data.method.clone()
                                confidence=data.confidence_level
                            />
                        </div>
                    }.into_any(),
                    Err(e) => view! {
                        <div class="text-center py-12 text-red-600">
                            {format!("Gagal memuat data: {}", e)}
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// ── Sub-components ───────────────────────────────────────────────

#[component]
fn SummaryCards(summary: ForecastSummary) -> impl IntoView {
    let trend_icon = match summary.trend_direction.as_str() {
        "increasing" => "📈",
        "decreasing" => "📉",
        _ => "➡️",
    };
    let trend_color = match summary.trend_direction.as_str() {
        "increasing" => "text-red-600",
        "decreasing" => "text-green-600",
        _ => "text-gray-600",
    };

    view! {
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
            <div class="p-4 bg-blue-50 rounded-lg">
                <p class="text-sm text-gray-600">"Data Historis"</p>
                <p class="text-2xl font-bold text-blue-600">
                    {format!("{} tahun", summary.total_historical_years)}
                </p>
            </div>
            <div class="p-4 bg-purple-50 rounded-lg">
                <p class="text-sm text-gray-600">"Prediksi"</p>
                <p class="text-2xl font-bold text-purple-600">
                    {format!("{} tahun", summary.total_predicted_years)}
                </p>
            </div>
            <div class="p-4 bg-orange-50 rounded-lg">
                <p class="text-sm text-gray-600">"Rata-rata Pertumbuhan"</p>
                <p class=format!("text-2xl font-bold {}", trend_color)>
                    {format!("{} {:.1}%", trend_icon, summary.avg_yearly_growth_pct)}
                </p>
            </div>
            <div class="p-4 bg-green-50 rounded-lg">
                <p class="text-sm text-gray-600">"Total Prediksi Biaya"</p>
                <p class="text-2xl font-bold text-green-600">
                    {format!("Rp {:.0}", summary.total_predicted_biaya)}
                </p>
            </div>
        </div>
    }
}

#[component]
fn HistoricalTable(data: Vec<YearlyData>) -> impl IntoView {
    view! {
        <div class="mb-8">
            <h2 class="text-lg font-semibold text-gray-800 mb-4">"Data Historis Kebutuhan BMN"</h2>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-gray-50">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">"Tahun"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Kebutuhan"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Existing"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Gap"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Satker"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">"Estimasi Biaya"</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        <For
                            each=move || data.clone()
                            key=|d| d.tahun
                            children=move |d| view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-4 py-3 text-sm font-medium text-gray-900">{d.tahun}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">{d.total_kebutuhan}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">{d.total_existing}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">{d.total_gap}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">{d.jumlah_satker}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">
                                        {format!("Rp {:.0}", d.estimasi_total_biaya)}
                                    </td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
fn PredictionTable(
    predictions: Vec<PredictedYear>,
    method: String,
    confidence: f64,
) -> impl IntoView {
    view! {
        <div>
            <div class="flex items-center gap-3 mb-4">
                <h2 class="text-lg font-semibold text-gray-800">"Prediksi Kebutuhan BMN"</h2>
                <span class="px-2 py-1 text-xs font-medium rounded-full bg-blue-100 text-blue-800">
                    {method}
                </span>
                <span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
                    {format!("CI {:.0}%", confidence * 100.0)}
                </span>
            </div>
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-gray-200">
                    <thead class="bg-indigo-50">
                        <tr>
                            <th class="px-4 py-3 text-left text-xs font-medium text-indigo-600 uppercase">"Tahun"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-indigo-600 uppercase">"Prediksi Kebutuhan"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-indigo-600 uppercase">"Prediksi Gap"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-indigo-600 uppercase">"Prediksi Biaya"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-indigo-600 uppercase">"CI Bawah"</th>
                            <th class="px-4 py-3 text-right text-xs font-medium text-indigo-600 uppercase">"CI Atas"</th>
                        </tr>
                    </thead>
                    <tbody class="bg-white divide-y divide-gray-200">
                        <For
                            each=move || predictions.clone()
                            key=|p| p.tahun
                            children=move |p| view! {
                                <tr class="hover:bg-indigo-50/50">
                                    <td class="px-4 py-3 text-sm font-medium text-indigo-900">{p.tahun}</td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">
                                        {format!("{:.0}", p.predicted_kebutuhan)}
                                    </td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">
                                        {format!("{:.0}", p.predicted_gap)}
                                    </td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-700">
                                        {format!("Rp {:.0}", p.predicted_biaya)}
                                    </td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-500">
                                        {format!("{:.0}", p.confidence_lower)}
                                    </td>
                                    <td class="px-4 py-3 text-sm text-right text-gray-500">
                                        {format!("{:.0}", p.confidence_upper)}
                                    </td>
                                </tr>
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}
