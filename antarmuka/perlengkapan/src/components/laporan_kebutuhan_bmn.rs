//! Laporan Kebutuhan BMN — report page with filters and export.

use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{FILE_PDF, FILE_XLS};

/// `()`-keyed query wrapper around the mock fetch — keeps the
/// signature compatible with `client.local_resource` so swapping
/// the mock for a real endpoint later doesn't ripple into render
/// code.
async fn query_kebutuhan_bmn_list(_: ()) -> Result<Vec<KebutuhanBmnItem>, String> {
    mock_fetch_kebutuhan_bmn_list().await
}

#[derive(Clone, Debug, PartialEq)]
pub struct KebutuhanBmnItem {
    pub nama_barang: String,
    pub nama_satker: Option<String>,
    pub status_label: Option<String>,
    pub jumlah: i32,
}

pub async fn mock_fetch_kebutuhan_bmn_list() -> Result<Vec<KebutuhanBmnItem>, String> {
    Ok(vec![
        KebutuhanBmnItem {
            nama_barang: "Laptop Pengadaan 2024".to_string(),
            nama_satker: Some("Satker A".to_string()),
            status_label: Some("Disetujui".to_string()),
            jumlah: 10,
        },
        KebutuhanBmnItem {
            nama_barang: "Meja Kantor".to_string(),
            nama_satker: Some("Satker B".to_string()),
            status_label: Some("Diajukan".to_string()),
            jumlah: 5,
        },
    ])
}

#[component]
pub fn LaporanKebutuhanBmn() -> impl IntoView {
    let tahun = RwSignal::new("2025".to_string());
    let status_filter = RwSignal::new("semua".to_string());
    let client: QueryClient = expect_context();
    let data = client.local_resource(query_kebutuhan_bmn_list, || ());

    view! {
        <div style="max-width: 1100px; margin: 0 auto;">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px;">
                <div>
                    <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">"Laporan Kebutuhan BMN"</h1>
                    <p style="font-size: 0.8rem; color: #64748b; margin-top: 4px;">"Rekap dan laporan analisis kebutuhan barang milik negara"</p>
                </div>
                <div style="display: flex; gap: 8px;">
                    <button style="padding: 8px 16px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #94a3b8; font-size: 0.78rem; font-weight: 600; cursor: pointer;">
                        <span style="margin-right: 6px; color: #34d399;"><AppIcon icon=FILE_XLS /></span> "Export XLSX"
                    </button>
                    <button style="padding: 8px 16px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #94a3b8; font-size: 0.78rem; font-weight: 600; cursor: pointer;">
                        <span style="margin-right: 6px; color: #f87171;"><AppIcon icon=FILE_PDF /></span> "Export PDF"
                    </button>
                </div>
            </div>

            // Filters
            <div style="display: flex; gap: 12px; margin-bottom: 20px;">
                <select
                    style="padding: 8px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem;"
                    on:change=move |ev| tahun.set(event_target_value(&ev))
                >
                    <option value="2025">"2025"</option>
                    <option value="2024">"2024"</option>
                    <option value="2023">"2023"</option>
                </select>
                <select
                    style="padding: 8px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; color: #e2e8f0; font-size: 0.82rem;"
                    on:change=move |ev| status_filter.set(event_target_value(&ev))
                >
                    <option value="semua">"Semua Status"</option>
                    <option value="draft">"Draft"</option>
                    <option value="diajukan">"Diajukan"</option>
                    <option value="disetujui">"Disetujui"</option>
                    <option value="ditolak">"Ditolak"</option>
                </select>
            </div>

            // Summary cards
            <Suspense fallback=move || view! { <div style="color: #64748b; padding: 20px;">"Memuat data..."</div> }>
                {move || {
                    let items = data.get().and_then(|r| r.ok()).unwrap_or_default();
                    let total = items.len();
                    let disetujui = items.iter().filter(|i| i.status_label.as_deref() == Some("Disetujui")).count();
                    let pending = items.iter().filter(|i| i.status_label.as_deref() == Some("Diajukan")).count();

                    view! {
                        // Summary row
                        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-bottom: 24px;">
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">"Total Pengajuan"</div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #e2e8f0; margin-top: 4px;">{total}</div>
                            </div>
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">"Disetujui"</div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #34d399; margin-top: 4px;">{disetujui}</div>
                            </div>
                            <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 14px; padding: 18px;">
                                <div style="font-size: 0.72rem; color: #64748b; font-weight: 500;">"Menunggu"</div>
                                <div style="font-size: 1.5rem; font-weight: 800; color: #fbbf24; margin-top: 4px;">{pending}</div>
                            </div>
                        </div>

                        // Data table
                        <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; overflow: hidden;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead>
                                    <tr style="border-bottom: 1px solid rgba(255,255,255,0.06);">
                                        <th style="padding: 12px 16px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"No"</th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Nama Barang"</th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Satker"</th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Status"</th>
                                        <th style="padding: 12px 8px; text-align: right; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Jumlah"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {items.iter().enumerate().map(|(i, item)| {
                                        let nama = item.nama_barang.clone();
                                        let satker = item.nama_satker.clone().unwrap_or_default();
                                        let status = item.status_label.clone().unwrap_or_else(|| "—".to_string());
                                        let jumlah = item.jumlah;
                                        let status_color = match status.as_str() {
                                            "Disetujui" => "#34d399",
                                            "Ditolak" => "#f87171",
                                            "Diajukan" => "#fbbf24",
                                            _ => "#64748b",
                                        };
                                        view! {
                                            <tr style="border-bottom: 1px solid rgba(255,255,255,0.04);">
                                                <td style="padding: 10px 16px; font-size: 0.78rem; color: #475569;">{i + 1}</td>
                                                <td style="padding: 10px 8px; font-size: 0.8rem; color: #e2e8f0;">{nama}</td>
                                                <td style="padding: 10px 8px; font-size: 0.78rem; color: #94a3b8;">{satker}</td>
                                                <td style="padding: 10px 8px;">
                                                    <span style=format!("font-size: 0.7rem; font-weight: 600; padding: 3px 10px; border-radius: 6px; background: {}18; color: {};", status_color, status_color)>{status}</span>
                                                </td>
                                                <td style="padding: 10px 8px; font-size: 0.8rem; color: #e2e8f0; text-align: right; font-weight: 600;">{jumlah}</td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>
                    }
                }}
            </Suspense>
        </div>
    }
}
