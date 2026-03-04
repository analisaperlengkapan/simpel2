//! Dashboard Home — premium glassmorphism stat cards + quick navigation grid.
//! Stat cards integrate with `GET /dashboard/stats` backend endpoint.

use crate::api::dashboard::fetch_dashboard_stats;
use crate::components::role_switcher::get_active_role;
use leptos::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// Stat Card
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn StatCard(
    icon: &'static str,
    label: &'static str,
    #[prop(into)] value: String,
    subtitle: &'static str,
    glow_color: &'static str,
) -> impl IntoView {
    view! {
        <div style=format!(
            "position: relative; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 20px; padding: 24px; overflow: hidden; cursor: default;"
        )>
            <div style=format!(
                "position: absolute; top: -10px; right: -10px; width: 80px; height: 80px; background: {}; border-radius: 50%; filter: blur(30px); opacity: 0.25;",
                glow_color
            )></div>
            <div style="position: relative; z-index: 1;">
                <div style="margin-bottom: 16px;">
                    <div style="width: 44px; height: 44px; display: flex; align-items: center; justify-content: center; border-radius: 14px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08);">
                        <i class=icon style=format!("font-size: 1.1rem; color: {};", glow_color)></i>
                    </div>
                </div>
                <div style="font-size: 0.78rem; color: #94a3b8; font-weight: 500; margin-bottom: 6px;">{label}</div>
                <div style="font-size: 1.8rem; font-weight: 800; color: #ffffff; line-height: 1; margin-bottom: 8px;">{value}</div>
                <div style="font-size: 0.7rem; color: #64748b;">{subtitle}</div>
            </div>
        </div>
    }
}

#[component]
fn StatCardSkeleton() -> impl IntoView {
    view! {
        <div style="background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); border-radius: 20px; padding: 24px;">
            <div style="width: 44px; height: 44px; background: rgba(255,255,255,0.06); border-radius: 14px; margin-bottom: 16px;"></div>
            <div style="width: 80px; height: 12px; background: rgba(255,255,255,0.06); border-radius: 4px; margin-bottom: 10px;"></div>
            <div style="width: 50px; height: 28px; background: rgba(255,255,255,0.06); border-radius: 6px; margin-bottom: 10px;"></div>
            <div style="width: 100px; height: 10px; background: rgba(255,255,255,0.06); border-radius: 4px;"></div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Quick Nav Card
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn QuickNav(
    href: &'static str,
    icon: &'static str,
    label: &'static str,
    description: &'static str,
    accent: &'static str,
) -> impl IntoView {
    view! {
        <a
            href=href
            style="display: block; text-decoration: none; background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 16px; padding: 20px;"
            class="hover:bg-white/[0.06]"
        >
            <div style="display: flex; align-items: center; gap: 14px;">
                <div style=format!(
                    "width: 42px; height: 42px; display: flex; align-items: center; justify-content: center; border-radius: 12px; background: {}15; flex-shrink: 0;",
                    accent
                )>
                    <i class=icon style=format!("font-size: 1rem; color: {};", accent)></i>
                </div>
                <div style="min-width: 0;">
                    <div style="font-size: 0.85rem; font-weight: 700; color: #e2e8f0; margin-bottom: 2px;">{label}</div>
                    <div style="font-size: 0.7rem; color: #64748b; line-height: 1.3;">{description}</div>
                </div>
            </div>
        </a>
    }
}

fn format_number(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ═══════════════════════════════════════════════════════════════════════════
// Dashboard Home
// ═══════════════════════════════════════════════════════════════════════════

#[component]
pub fn DashboardHome() -> impl IntoView {
    let active_role = get_active_role();
    let role_label = match active_role.as_str() {
        "validator_wilayah" => "Validator Wilayah",
        "validator_pusat" => "Validator Pusat",
        "admin" => "Administrator",
        _ => "Operator Satker",
    };

    let stats_resource = LocalResource::new(move || fetch_dashboard_stats());
    let is_admin = active_role == "admin";

    view! {
        <div style="max-width: 1200px; margin: 0 auto;">

            // ── Welcome Banner ───────────────────────────────────────
            <div style="position: relative; overflow: hidden; background: linear-gradient(135deg, #0f172a 0%, #1e3a5f 50%, #0a1020 100%); border: 1px solid rgba(255,255,255,0.08); border-radius: 24px; padding: 36px 32px; margin-bottom: 28px;">
                <div style="position: absolute; top: -40px; right: -40px; width: 200px; height: 200px; background: radial-gradient(circle, rgba(212,168,67,0.15) 0%, transparent 70%); border-radius: 50%; pointer-events: none;"></div>
                <div style="position: absolute; bottom: -30px; left: 30%; width: 150px; height: 150px; background: radial-gradient(circle, rgba(96,165,250,0.1) 0%, transparent 70%); border-radius: 50%; pointer-events: none;"></div>

                <div style="position: relative; z-index: 1;">
                    <div style="display: inline-flex; align-items: center; gap: 8px; padding: 5px 14px; background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); border-radius: 999px; font-size: 0.7rem; font-weight: 600; color: #d4a843; margin-bottom: 16px;">
                        <span style="width: 7px; height: 7px; background: #22c55e; border-radius: 50%; display: inline-block;"></span>
                        "Portal Perlengkapan Kejaksaan"
                    </div>
                    <h1 style="font-size: 2rem; font-weight: 800; color: #ffffff; margin: 0 0 8px 0; line-height: 1.2;">
                        "Ringkasan "
                        <span style="background: linear-gradient(90deg, #d4a843, #facc15); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">"Sistem Manajemen"</span>
                    </h1>
                    <p style="font-size: 0.88rem; color: #94a3b8; max-width: 600px; line-height: 1.5; margin: 0;">
                        "Akses dan kelola seluruh modul operasional aset dan infrastruktur Kejaksaan secara terintegrasi."
                    </p>

                    <div style="display: flex; flex-wrap: wrap; gap: 10px; margin-top: 22px;">
                        <div style="display: flex; align-items: center; gap: 8px; padding: 6px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 12px; font-size: 0.78rem; color: #94a3b8;">
                            <i class="fas fa-shield-alt" style="color: #22c55e; font-size: 0.75rem;"></i>
                            "Role: " <strong style="color: #ffffff;">{role_label}</strong>
                        </div>
                        <div style="display: flex; align-items: center; gap: 8px; padding: 6px 14px; background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.06); border-radius: 12px; font-size: 0.78rem; color: #94a3b8;">
                            <i class="fas fa-calendar" style="color: #60a5fa; font-size: 0.75rem;"></i>
                            "Tahun Anggaran 2025"
                        </div>
                    </div>
                </div>
            </div>

            // ── Stats Cards ──────────────────────────────────────────
            <Suspense fallback=move || view! {
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 32px;">
                    <StatCardSkeleton /> <StatCardSkeleton /> <StatCardSkeleton /> <StatCardSkeleton />
                </div>
            }>
                {move || {
                    let stats = stats_resource.get().and_then(|r| r.ok());
                    let (total, baik, rusak, satker) = match &stats {
                        Some(s) => (
                            format_number(s.total_aset),
                            format_number(s.aset_baik),
                            format_number(s.aset_rusak),
                            format_number(s.total_satker),
                        ),
                        None => ("—".into(), "—".into(), "—".into(), "—".into()),
                    };
                    view! {
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 32px;">
                            <StatCard icon="fas fa-box"                  label="Total Aset BMN"   value=total   subtitle="Terintegrasi SIMAN"   glow_color="#60a5fa" />
                            <StatCard icon="fas fa-check-circle"         label="Kondisi Baik"     value=baik    subtitle="Siap Pakai"            glow_color="#34d399" />
                            <StatCard icon="fas fa-exclamation-triangle" label="Perlu Perbaikan"  value=rusak   subtitle="Tindakan Diperlukan"   glow_color="#d4a843" />
                            <StatCard icon="fas fa-building"             label="Satuan Kerja"     value=satker  subtitle="Unit Kerja Aktif"      glow_color="#818cf8" />
                        </div>
                    }
                }}
            </Suspense>

            // ── Modul Utama ──────────────────────────────────────────
            <div style="margin-bottom: 28px;">
                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 18px;">
                    <div style="width: 4px; height: 28px; background: linear-gradient(180deg, #d4a843, #facc15); border-radius: 2px;"></div>
                    <h2 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0;">"Modul Utama"</h2>
                </div>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 12px;">
                    <QuickNav href="/perlengkapan/dashboard/bank-aset/daftar"         icon="fas fa-boxes"          label="Bank Aset"        description="Katalog dan registrasi BMN"              accent="#34d399" />
                    <QuickNav href="/perlengkapan/dashboard/kebutuhan-bmn/daftar"     icon="fas fa-clipboard-list" label="Kebutuhan BMN"    description="Analisis kebutuhan dan perencanaan"      accent="#fb923c" />
                    <QuickNav href="/perlengkapan/dashboard/pakaian-dinas/pengajuan"  icon="fas fa-tshirt"         label="Pakaian Dinas"    description="Pengajuan dan distribusi atribut"         accent="#c084fc" />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/pemakaian"    icon="fas fa-file-signature" label="Pemakaian BMN"    description="Izin pemakaian dan monitoring"            accent="#60a5fa" />
                    <QuickNav href="/perlengkapan/dashboard/pengelolaan/penghapusan"  icon="fas fa-trash-alt"      label="Penghapusan BMN"  description="Disposal dan penghapusan BMN"             accent="#f87171" />
                </div>
            </div>

            // ── Analitik ─────────────────────────────────────────────
            <div style="margin-bottom: 28px;">
                <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 18px;">
                    <div style="width: 4px; height: 28px; background: linear-gradient(180deg, #2dd4bf, #14b8a6); border-radius: 2px;"></div>
                    <h2 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0;">"Analitik"</h2>
                </div>
                <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 12px;">
                    <QuickNav href="/perlengkapan/dashboard/analitik/roadmap"     icon="fas fa-road"     label="Roadmap Sarpras"  description="Prediksi kebutuhan sarana prasarana"     accent="#2dd4bf" />
                    <QuickNav href="/perlengkapan/dashboard/analitik/kodefikasi"  icon="fas fa-barcode"  label="Kodefikasi BMN"   description="Mapping kode barang standar"             accent="#a78bfa" />
                </div>
            </div>

            // ── Panel Admin (admin only) ──────────────────────────────
            {is_admin.then(|| view! {
                <div style="margin-bottom: 28px;">
                    <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 18px;">
                        <div style="width: 4px; height: 28px; background: linear-gradient(180deg, #ef4444, #f87171); border-radius: 2px;"></div>
                        <h2 style="font-size: 1.1rem; font-weight: 700; color: #e2e8f0; margin: 0; display: flex; align-items: center; gap: 8px;">
                            "Panel Administrator"
                            <i class="fas fa-shield-alt" style="font-size: 0.75rem; color: #ef4444;"></i>
                        </h2>
                    </div>
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 12px;">
                        <QuickNav href="/perlengkapan/dashboard/admin/users"  icon="fas fa-users-cog"  label="Manajemen Pengguna"  description="Data pengguna dan sesi aktif"         accent="#f87171" />
                        <QuickNav href="/perlengkapan/dashboard/admin/roles"  icon="fas fa-user-tag"   label="Otorisasi (RBAC)"    description="Konfigurasi hak akses peran"          accent="#fb923c" />
                        <QuickNav href="/perlengkapan/dashboard/admin/audit"  icon="fas fa-history"    label="Audit Log"           description="Jejak audit seluruh aktivitas"        accent="#94a3b8" />
                        <QuickNav href="/perlengkapan/dashboard/admin/master" icon="fas fa-database"   label="Master Data"         description="Pengelolaan data referensi"           accent="#2dd4bf" />
                    </div>
                </div>
            })}

        </div>
    }
}
