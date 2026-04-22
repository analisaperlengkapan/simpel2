//! Sidebar — dark navy Kejaksaan-branded navigation.
//!
//! Menu structure:
//!   Dashboard (single link)
//!   Bank Aset (Daftar Aset, Cetak QR)
//!   Kebutuhan BMN (Daftar, Buat Baru, Laporan)
//!   Pakaian Dinas (Jenis, Pengajuan, Ukuran, Laporan)
//!   Pengelolaan BMN (Pemakaian BMN, Penghapusan)
//!   Analitik (Roadmap Sarpras, Kodefikasi BMN)
//!   Admin (Pengguna, Otorisasi, Audit Log, Master Data)
//!   Bantuan (Panduan, FAQ, Helpdesk)

use crate::{navigation, routes};
use leptos::prelude::*;
use leptos_router::components::A;

// ══════════════════════════════════════════════════════════════════════
// Helper: section header
// ══════════════════════════════════════════════════════════════════════

#[component]
fn SectionHeader(label: &'static str) -> impl IntoView {
    view! {
        <div style="padding: 16px 18px 6px; font-size: 0.6rem; font-weight: 700; color: #475569; text-transform: uppercase; letter-spacing: 0.1em;">
            {label}
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════
// Helper: single nav link
// ══════════════════════════════════════════════════════════════════════

#[component]
fn NavLink(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    #[prop(default = false)] gold: bool,
) -> impl IntoView {
    let icon_color = if gold { "#d4a843" } else { "#64748b" };
    view! {
        <A
            href=href
            attr:style=format!(
                "display: flex; align-items: center; gap: 10px; padding: 8px 18px; text-decoration: none; font-size: 0.8rem; color: #94a3b8; transition: all 0.15s; border-left: 2px solid transparent;"
            )
            attr:class="hover:bg-white/[0.04] hover:text-white hover:border-l-gold-400"
        >
            <i class=icon style=format!("font-size: 0.7rem; color: {}; width: 18px; text-align: center;", icon_color)></i>
            <span>{label}</span>
        </A>
    }
}

// ══════════════════════════════════════════════════════════════════════
// Helper: collapsible section with sub-links
// ══════════════════════════════════════════════════════════════════════

#[component]
fn NavSection(group: navigation::NavGroup) -> impl IntoView {
    let is_open = RwSignal::new(false);

    view! {
        <div>
            <button
                on:click=move |_| is_open.update(|o| *o = !*o)
                style="width: 100%; display: flex; align-items: center; gap: 10px; padding: 9px 18px; border: none; background: none; color: #94a3b8; font-size: 0.8rem; cursor: pointer; transition: all 0.15s; text-align: left;"
                class="hover:bg-white/[0.04] hover:text-white"
            >
                <i class=group.icon style=format!("font-size: 0.72rem; color: {}; width: 18px; text-align: center;", group.accent)></i>
                <span style="flex: 1;">{group.label}</span>
                <i class="fas fa-chevron-right" style=move || format!(
                    "font-size: 0.55rem; color: #475569; transition: transform 0.2s; transform: rotate({}deg);",
                    if is_open.get() { 90 } else { 0 }
                )></i>
            </button>
            <div style=move || format!(
                "overflow: hidden; transition: max-height 0.25s ease; max-height: {};",
                if is_open.get() { "500px" } else { "0" }
            )>
                <div style="padding-left: 14px; border-left: 1px solid rgba(255,255,255,0.05); margin-left: 27px;">
                    {group
                        .items
                        .iter()
                        .map(|item| {
                            view! {
                                <NavLink href=item.href icon=item.icon label=item.label />
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════
// Sidebar component
// ══════════════════════════════════════════════════════════════════════

#[component]
pub fn Sidebar(sidebar_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <div
            class=move || format!(
                "transition-transform duration-300 transform lg:translate-x-0 lg:static lg:inset-0 {}",
                if sidebar_open.get() { "translate-x-0" } else { "-translate-x-full" }
            )
            style="position: fixed; inset-y: 0; left: 0; z-index: 40; width: 250px; background: linear-gradient(180deg, #0c1425 0%, #0f172a 40%, #0c1425 100%); border-right: 1px solid rgba(255,255,255,0.06); display: flex; flex-direction: column; overflow: hidden;"
        >
            // ── Brand ────────────────────────────────────────────
            <div style="padding: 20px 18px 16px; border-bottom: 1px solid rgba(255,255,255,0.05);">
                <A href=routes::path::DASHBOARD attr:style="display: flex; align-items: center; gap: 12px; text-decoration: none;">
                    <img
                        src="/perlengkapan/assets/kejaksaan-logo.png"
                        alt="Kejaksaan RI"
                        style="width: 36px; height: 36px; object-fit: contain;"
                    />
                    <div>
                        <div style="font-size: 1rem; font-weight: 800; color: #ffffff; letter-spacing: 0.03em; line-height: 1;">"SIMPEL"</div>
                        <div style="font-size: 0.6rem; color: #64748b; margin-top: 2px; letter-spacing: 0.05em; text-transform: uppercase;">"Manajemen Perlengkapan"</div>
                    </div>
                </A>
            </div>

            // ── Navigation ───────────────────────────────────────
            <nav style="flex: 1; overflow-y: auto; padding: 8px 0;">

                // Dashboard (single link, gold accent)
                <div style="padding: 2px 0 6px;">
                    <NavLink
                        href=navigation::DASHBOARD_ITEM.href
                        icon=navigation::DASHBOARD_ITEM.icon
                        label=navigation::DASHBOARD_ITEM.label
                        gold=true
                    />
                </div>

                {navigation::SIDEBAR_SECTIONS
                    .iter()
                    .enumerate()
                    .map(|(idx, section)| {
                        view! {
                            <div style=if idx == 0 {
                                "border-bottom: 1px solid rgba(255,255,255,0.04); margin: 4px 18px;".to_string()
                            } else {
                                "border-bottom: 1px solid rgba(255,255,255,0.04); margin: 6px 18px;".to_string()
                            }></div>
                            <SectionHeader label=section.title />
                            {section
                                .groups
                                .iter()
                                .map(|group| {
                                    view! {
                                        <NavSection group=*group />
                                    }
                                })
                                .collect_view()}
                        }
                    })
                    .collect_view()}

            </nav>

            // ── Footer ───────────────────────────────────────────
            <div style="padding: 12px 18px; border-top: 1px solid rgba(255,255,255,0.05); font-size: 0.6rem; color: #475569;">
                "v" {crate::APP_VERSION}
            </div>
        </div>
    }
}
