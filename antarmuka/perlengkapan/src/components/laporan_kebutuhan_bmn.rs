//! Laporan Kebutuhan BMN — report page with filters and export.
//!
//! Reads the real `/kebutuhan-bmn/laporan/rekap` endpoint (E-5). Previously
//! this page rendered a hard-coded mock list; the filters keyed a cache slot
//! but changed nothing, and the export buttons had no handler at all.
//!
//! Rows are scoped server-side (campaign visibility + row-level satker), so
//! an operator sees only their own satker's items and the exports carry
//! exactly the rows on screen.

use crate::api::{RekapLaporanRow, export_rekap_laporan, fetch_rekap_laporan};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{FILE_PDF, FILE_XLS};

/// Query wrapper keyed by `(tahun, status_kode)` so flipping either dropdown
/// refetches with the matching server-side filter.
async fn query_rekap_laporan(
    key: (Option<i32>, Option<i32>),
) -> Result<Vec<RekapLaporanRow>, String> {
    fetch_rekap_laporan(key.0, key.1)
        .await
        .map(|r| r.data)
        .map_err(|e| e.to_string())
}

/// Status codes for the kebutuhan-BMN workflow, from `ms_aktivitas_bmn`
/// (the 2000-series). Label = the human wording used elsewhere in the UI.
const STATUS_OPTIONS: [(i32, &str); 10] = [
    (2000, "Draft"),
    (2001, "Input Barang"),
    (2002, "Diajukan ke Validator"),
    (2003, "Revisi Satker"),
    (2004, "Analisis Kelayakan"),
    (2005, "Penyusunan Prioritas"),
    (2006, "Disetujui"),
    (2007, "Ditolak"),
    (2008, "Selesai"),
    (2009, "Dibatalkan"),
];

fn status_color(kode: i32) -> &'static str {
    match kode {
        2006 | 2008 => "#34d399",
        2007 | 2009 => "#f87171",
        2002 | 2004 | 2005 => "#fbbf24",
        _ => "#64748b",
    }
}

/// Fetch an export and hand the bytes to the browser as a download. The
/// endpoint needs the JWT, so a plain `<a href>` cannot be used — fetch with
/// auth, wrap in a blob, then click a synthetic anchor.
#[cfg(target_arch = "wasm32")]
fn download_export(format: &'static str, tahun: Option<i32>, status_kode: Option<i32>) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, Url};

    spawn_local(async move {
        let bytes = match export_rekap_laporan(format, tahun, status_kode).await {
            Ok(b) => b,
            Err(e) => {
                leptos::logging::error!("export {format} gagal: {e}");
                return;
            }
        };
        let mime = if format == "pdf" {
            "application/pdf"
        } else {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        };
        let array = js_sys::Uint8Array::from(bytes.as_slice());
        let parts = js_sys::Array::new();
        parts.push(&array);
        let opts = BlobPropertyBag::new();
        opts.set_type(mime);
        let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &opts) else {
            return;
        };
        let Ok(url) = Url::create_object_url_with_blob(&blob) else {
            return;
        };
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(a) = doc.create_element("a") {
                let a: web_sys::HtmlAnchorElement = a.unchecked_into();
                a.set_href(&url);
                a.set_download(&format!("Laporan_Kebutuhan_BMN.{}", format));
                a.click();
            }
        }
        let _ = Url::revoke_object_url(&url);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn download_export(_format: &'static str, _tahun: Option<i32>, _status_kode: Option<i32>) {}

#[component]
pub fn LaporanKebutuhanBmn() -> impl IntoView {
    // `None` = no filter (server returns everything in scope). Defaulting to
    // "all" keeps the page useful without guessing which year has data.
    let tahun = RwSignal::new(None::<i32>);
    let status_filter = RwSignal::new(None::<i32>);
    let client: QueryClient = expect_context();
    let data = client.local_resource(query_rekap_laporan, move || {
        (tahun.get(), status_filter.get())
    });

    // Year options span a window around the *actual* current year so a fresh
    // campaign is always selectable — the previous hard-coded 2023–2025 list
    // would have hidden a 2026 campaign entirely, and any fixed list rots.
    let current_year = chrono::Local::now()
        .naive_local()
        .date()
        .format("%Y")
        .to_string();
    let current_year: i32 = current_year.parse().unwrap_or(2026);
    let years: Vec<i32> = ((current_year - 3)..=(current_year + 1)).rev().collect();

    view! {
        <div style="max-width: 1100px; margin: 0 auto;">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px;">
                <div>
                    <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">
                        "Laporan Kebutuhan BMN"
                    </h1>
                    <p style="font-size: 0.82rem; color: #64748b; margin: 4px 0 0;">
                        "Rekap dan laporan analisis kebutuhan barang milik negara"
                    </p>
                </div>
                <div style="display: flex; gap: 8px;">
                    <button
                        style="padding: 8px 16px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #94a3b8; font-size: 0.78rem; font-weight: 600; cursor: pointer;"
                        on:click=move |_| download_export("xlsx", tahun.get(), status_filter.get())
                    >
                        <span style="margin-right: 6px; color: #34d399;">
                            <AppIcon icon=FILE_XLS />
                        </span>
                        "Export XLSX"
                    </button>
                    <button
                        style="padding: 8px 16px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #94a3b8; font-size: 0.78rem; font-weight: 600; cursor: pointer;"
                        on:click=move |_| download_export("pdf", tahun.get(), status_filter.get())
                    >
                        <span style="margin-right: 6px; color: #f87171;">
                            <AppIcon icon=FILE_PDF />
                        </span>
                        "Export PDF"
                    </button>
                </div>
            </div>

            // Filters
            <div style="display: flex; gap: 12px; margin-bottom: 20px;">
                <select
                    aria-label="Filter tahun"
                    style="padding: 8px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem;"
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        tahun.set(v.parse::<i32>().ok());
                    }
                >
                    <option value="">"Semua Tahun"</option>
                    {years
                        .into_iter()
                        .map(|y| view! { <option value=y.to_string()>{y.to_string()}</option> })
                        .collect_view()}
                </select>
                <select
                    aria-label="Filter status"
                    style="padding: 8px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem;"
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        status_filter.set(v.parse::<i32>().ok());
                    }
                >
                    <option value="">"Semua Status"</option>
                    {STATUS_OPTIONS
                        .iter()
                        .map(|(kode, label)| {
                            view! { <option value=kode.to_string()>{label.to_string()}</option> }
                        })
                        .collect_view()}
                </select>
            </div>

            <Suspense fallback=move || {
                view! { <div style="color: #64748b; padding: 20px;">"Memuat data..."</div> }
            }>
                {move || {
                    let items = data.get().and_then(|r| r.ok()).unwrap_or_default();
                    let total_baris = items.len();
                    let total_diusulkan: i32 = items.iter().map(|i| i.jumlah).sum();
                    let total_disetujui: i32 = items.iter().map(|i| i.jml_setuju).sum();

                    view! {
                        // Summary row — totals over the rows actually returned.
                        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 24px;">
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">
                                    "Total Baris"
                                </div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #e2e8f0; margin-top: 4px;">
                                    {total_baris}
                                </div>
                            </div>
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">
                                    "Jumlah Diusulkan"
                                </div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #fbbf24; margin-top: 4px;">
                                    {total_diusulkan}
                                </div>
                            </div>
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">
                                    "Jumlah Disetujui"
                                </div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #34d399; margin-top: 4px;">
                                    {total_disetujui}
                                </div>
                            </div>
                        </div>

                        // Data table
                        <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; overflow: hidden;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead>
                                    <tr style="border-bottom: 1px solid rgba(255,255,255,0.06);">
                                        <th style="padding: 12px 16px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">
                                            "No"
                                        </th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">
                                            "Nama Barang"
                                        </th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">
                                            "Satker"
                                        </th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">
                                            "Status"
                                        </th>
                                        <th style="padding: 12px 8px; text-align: right; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">
                                            "Jumlah"
                                        </th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {if items.is_empty() {
                                        view! {
                                            <tr>
                                                <td
                                                    colspan="5"
                                                    style="padding: 24px 16px; text-align: center; font-size: 0.82rem; color: #64748b;"
                                                >
                                                    "Tidak ada data untuk filter ini."
                                                </td>
                                            </tr>
                                        }
                                            .into_any()
                                    } else {
                                        items
                                            .iter()
                                            .enumerate()
                                            .map(|(i, item)| {
                                                let nama = item.nama_barang.clone();
                                                let satker = item
                                                    .satker_nama
                                                    .clone()
                                                    .unwrap_or_else(|| item.satker_id.clone());
                                                let status = item
                                                    .status_nama
                                                    .clone()
                                                    .unwrap_or_else(|| "—".to_string());
                                                let color = status_color(item.status_kode);
                                                let jumlah = item.jumlah;
                                                view! {
                                                    <tr style="border-bottom: 1px solid rgba(255,255,255,0.04);">
                                                        <td style="padding: 10px 16px; font-size: 0.78rem; color: #475569;">
                                                            {i + 1}
                                                        </td>
                                                        <td style="padding: 10px 8px; font-size: 0.8rem; color: #e2e8f0;">
                                                            {nama}
                                                        </td>
                                                        <td style="padding: 10px 8px; font-size: 0.78rem; color: #94a3b8;">
                                                            {satker}
                                                        </td>
                                                        <td style="padding: 10px 8px;">
                                                            <span style=format!(
                                                                "font-size: 0.7rem; font-weight: 600; padding: 3px 10px; border-radius: 6px; background: {}18; color: {};",
                                                                color,
                                                                color,
                                                            )>{status}</span>
                                                        </td>
                                                        <td style="padding: 10px 8px; font-size: 0.8rem; color: #e2e8f0; text-align: right; font-weight: 600;">
                                                            {jumlah}
                                                        </td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                            .into_any()
                                    }}
                                </tbody>
                            </table>
                        </div>
                    }
                }}
            </Suspense>
        </div>
    }
}
