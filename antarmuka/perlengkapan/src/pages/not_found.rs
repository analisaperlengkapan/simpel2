//! 404 Not Found page

use crate::routes;
use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{HOUSE};

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div style="min-height: 100vh; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, #0f172a, #1e3a5f, #0a1020); text-align: center; padding: 2rem;">
            <div>
                <div style="font-size: 5rem; font-weight: 900; color: rgba(255,255,255,0.06); line-height: 1;">"404"</div>
                <h1 style="font-size: 1.3rem; font-weight: 700; color: #e2e8f0; margin: 16px 0 8px;">"Halaman Tidak Ditemukan"</h1>
                <p style="font-size: 0.88rem; color: #64748b; margin: 0 0 24px;">"Halaman yang Anda cari tidak ada atau telah dipindahkan."</p>
                <a
                    href=routes::path::DASHBOARD
                    style="display: inline-flex; align-items: center; gap: 8px; padding: 10px 24px; background: linear-gradient(135deg, #d4a843, #facc15); color: #0f172a; font-weight: 700; font-size: 0.88rem; border-radius: 12px; text-decoration: none; transition: all 0.2s;"
                >
                    <AppIcon icon=HOUSE />
                    "Ke Dashboard"
                </a>
            </div>
        </div>
    }
}
