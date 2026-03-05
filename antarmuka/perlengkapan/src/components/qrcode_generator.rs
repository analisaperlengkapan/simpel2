//! QR Code Generator — batch QR code printing for BMN assets.

use leptos::prelude::*;
use crate::api::{Asset, fetch_assets};

#[component]
pub fn QrCodeGenerator() -> impl IntoView {
    let search = RwSignal::new(String::new());
    let selected = RwSignal::new(Vec::<String>::new());
    let assets = LocalResource::new(move || async move {
        match fetch_assets(1, 100, None).await {
            Ok(res) => Some(res),
            Err(e) => {
                leptos::logging::error!("Failed to fetch assets for QR: {:?}", e);
                None
            }
        }
    });

    view! {
        <div style="max-width: 1100px; margin: 0 auto;">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px;">
                <div>
                    <h1 style="font-size: 1.4rem; font-weight: 800; color: #e2e8f0; margin: 0;">"Cetak QR Code"</h1>
                    <p style="font-size: 0.8rem; color: #64748b; margin-top: 4px;">"Generate dan cetak label QR Code untuk aset BMN"</p>
                </div>
                <button
                    style="padding: 10px 20px; background: linear-gradient(135deg, #d4a843, #facc15); color: #0f172a; font-weight: 700; font-size: 0.82rem; border: none; border-radius: 10px; cursor: pointer;"
                    on:click=move |_| {
                        // Print selected
                        if let Some(window) = web_sys::window() {
                            let _ = window.print();
                        }
                    }
                >
                    <i class="fas fa-print" style="margin-right: 6px;"></i>
                    {move || format!("Cetak ({})", selected.get().len())}
                </button>
            </div>

            // Search + select all
            <div style="display: flex; gap: 12px; margin-bottom: 20px;">
                <div style="flex: 1; position: relative;">
                    <i class="fas fa-search" style="position: absolute; left: 12px; top: 50%; transform: translateY(-50%); color: #475569; font-size: 0.8rem;"></i>
                    <input
                        type="text"
                        placeholder="Cari aset berdasarkan kode atau nama..."
                        style="width: 100%; padding: 10px 10px 10px 36px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 10px; color: #e2e8f0; font-size: 0.82rem; outline: none;"
                        on:input=move |ev| search.set(event_target_value(&ev))
                    />
                </div>
            </div>

            // Asset list with checkboxes
            <Suspense fallback=move || view! {
                <div style="padding: 40px; text-align: center; color: #64748b;">"Memuat data aset..."</div>
            }>
                {move || {
                    let data = assets.get().flatten().map(|res| res.data).unwrap_or_default();
                    let q = search.get().to_lowercase();
                    let filtered: Vec<_> = data.iter()
                        .filter(|a| q.is_empty() || a.kode_barang.clone().unwrap_or_default().to_lowercase().contains(&q) || a.nama_aset.clone().unwrap_or_default().to_lowercase().contains(&q))
                        .collect();

                    view! {
                        <div style="background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; overflow: hidden;">
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead>
                                    <tr style="border-bottom: 1px solid rgba(255,255,255,0.06);">
                                        <th style="padding: 12px 16px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase; width: 40px;">
                                            <input type="checkbox" style="accent-color: #d4a843;" />
                                        </th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Kode Barang"</th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Nama Barang"</th>
                                        <th style="padding: 12px 8px; text-align: left; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"Satker"</th>
                                        <th style="padding: 12px 8px; text-align: center; font-size: 0.72rem; font-weight: 600; color: #64748b; text-transform: uppercase;">"QR"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {filtered.into_iter().map(|asset| {
                                        let kode = asset.kode_barang.clone().unwrap_or_default();
                                        let nama = asset.nama_aset.clone().unwrap_or_default();
                                        let satker = asset.satker.clone().unwrap_or_default();
                                        let id = asset.id.clone();
                                        view! {
                                            <tr style="border-bottom: 1px solid rgba(255,255,255,0.04);">
                                                <td style="padding: 10px 16px;">
                                                    <input type="checkbox" style="accent-color: #d4a843;"
                                                        on:change=move |_| {
                                                            selected.update(|s| {
                                                                if s.contains(&id) { s.retain(|x| x != &id); }
                                                                else { s.push(id.clone()); }
                                                            });
                                                        }
                                                    />
                                                </td>
                                                <td style="padding: 10px 8px; font-size: 0.8rem; color: #94a3b8; font-family: monospace;">{kode}</td>
                                                <td style="padding: 10px 8px; font-size: 0.8rem; color: #e2e8f0;">{nama}</td>
                                                <td style="padding: 10px 8px; font-size: 0.78rem; color: #64748b;">{satker}</td>
                                                <td style="padding: 10px 8px; text-align: center;">
                                                    <i class="fas fa-qrcode" style="color: #d4a843; font-size: 1rem;"></i>
                                                </td>
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
