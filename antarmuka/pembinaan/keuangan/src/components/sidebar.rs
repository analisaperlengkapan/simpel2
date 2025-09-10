use crate::components::sidebar_section::{MenuItem, SidebarSection};
use leptos::prelude::*;

#[component]
pub fn Sidebar(
    /// Sidebar open state signal
    _sidebar_open: RwSignal<bool>,
) -> impl IntoView {
    // Reactive signals for collapsible sections
    let dashboard_rw = RwSignal::new(false);
    let anggaran_rw = RwSignal::new(false);
    let pengeluaran_rw = RwSignal::new(false);
    let penerimaan_rw = RwSignal::new(false);
    let laporan_rw = RwSignal::new(false);
    let pengguna_rw = RwSignal::new(false);
    let bantuan_rw = RwSignal::new(false);

    view! {
        <div class="h-full bg-white border-r border-gray-200 w-64 overflow-y-auto">
            <div class="p-6 border-b border-gray-200">
                <div class="flex items-center space-x-3">
                    <div class="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center text-white font-bold">
                        "K"
                    </div>
                    <h1 class="text-xl font-bold text-gray-900">"SIMPEL KEUANGAN"</h1>
                </div>
                <p class="text-sm text-gray-500 mt-1">"Kejaksaan Republik Indonesia"</p>
            </div>

            <nav class="p-6 space-y-4">
                <SidebarSection
                    title="Dashboard".to_string()
                    icon="🏠".to_string()
                    is_expanded=dashboard_rw
                    items=vec![
                        MenuItem::new("/dashboard/overview", "Ringkasan"),
                        MenuItem::new("/dashboard/analytics", "Analitik"),
                        MenuItem::new("/dashboard/reports", "Laporan")
                    ]
                />

                <SidebarSection
                    title="Anggaran".to_string()
                    icon="💰".to_string()
                    is_expanded=anggaran_rw
                    items=vec![
                        MenuItem::new("/dashboard/anggaran/daftar", "Daftar Anggaran"),
                        MenuItem::new("/dashboard/anggaran/rencana", "Rencana Anggaran"),
                        MenuItem::new("/dashboard/anggaran/realisasi", "Realisasi Anggaran"),
                        MenuItem::new("/dashboard/anggaran/revisi", "Revisi Anggaran"),
                        MenuItem::new("/dashboard/anggaran/monitoring", "Monitoring Anggaran")
                    ]
                />

                <SidebarSection
                    title="Pengeluaran".to_string()
                    icon="📊".to_string()
                    is_expanded=pengeluaran_rw
                    items=vec![
                        MenuItem::new("/dashboard/pengeluaran/operasional", "Pengeluaran Operasional"),
                        MenuItem::new("/dashboard/pengeluaran/investasi", "Pengeluaran Investasi"),
                        MenuItem::new("/dashboard/pengeluaran/kapital", "Pengeluaran Kapital"),
                        MenuItem::new("/dashboard/pengeluaran/approval", "Approval Pengeluaran"),
                        MenuItem::new("/dashboard/pengeluaran/monitoring", "Monitoring Pengeluaran")
                    ]
                />

                <SidebarSection
                    title="Penerimaan".to_string()
                    icon="📈".to_string()
                    is_expanded=penerimaan_rw
                    items=vec![
                        MenuItem::new("/dashboard/penerimaan/pendapatan", "Penerimaan Pendapatan"),
                        MenuItem::new("/dashboard/penerimaan/hibah", "Penerimaan Hibah"),
                        MenuItem::new("/dashboard/penerimaan/dana", "Dana Cadangan"),
                        MenuItem::new("/dashboard/penerimaan/lainnya", "Penerimaan Lainnya"),
                        MenuItem::new("/dashboard/penerimaan/rekonsiliasi", "Rekonsiliasi")
                    ]
                />

                <SidebarSection
                    title="Laporan Keuangan".to_string()
                    icon="📋".to_string()
                    is_expanded=laporan_rw
                    items=vec![
                        MenuItem::new("/dashboard/laporan/keuangan", "Laporan Keuangan"),
                        MenuItem::new("/dashboard/laporan/pertanggungjawaban", "Laporan Pertanggungjawaban"),
                        MenuItem::new("/dashboard/laporan/audit", "Laporan Audit"),
                        MenuItem::new("/dashboard/laporan/bulanan", "Laporan Bulanan"),
                        MenuItem::new("/dashboard/laporan/tahunan", "Laporan Tahunan")
                    ]
                />

                <SidebarSection
                    title="Pengguna".to_string()
                    icon="👥".to_string()
                    is_expanded=pengguna_rw
                    items=vec![
                        MenuItem::new("/dashboard/pengguna/manajemen", "Manajemen User"),
                        MenuItem::new("/dashboard/pengguna/akses", "Hak Akses"),
                        MenuItem::new("/dashboard/pengguna/audit", "Audit Log"),
                        MenuItem::new("/dashboard/pengguna/profil", "Profil Pengguna"),
                        MenuItem::new("/dashboard/pengguna/aktivitas", "Log Aktivitas")
                    ]
                />

                <SidebarSection
                    title="Bantuan".to_string()
                    icon="❓".to_string()
                    is_expanded=bantuan_rw
                    items=vec![
                        MenuItem::new("/dashboard/bantuan/panduan", "Panduan Pengguna"),
                        MenuItem::new("/dashboard/bantuan/faq", "FAQ"),
                        MenuItem::new("/dashboard/bantuan/helpdesk", "Helpdesk"),
                        MenuItem::new("/dashboard/bantuan/kontak", "Kontak Support"),
                        MenuItem::new("/dashboard/bantuan/tutorial", "Tutorial")
                    ]
                />
            </nav>
        </div>
    }
}
