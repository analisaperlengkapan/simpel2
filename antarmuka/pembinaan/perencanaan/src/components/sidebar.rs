use crate::components::sidebar_section::{MenuItem, SidebarSection};
use leptos::prelude::*;

#[component]
pub fn Sidebar(
    /// Sidebar open state signal
    _sidebar_open: RwSignal<bool>,
) -> impl IntoView {
    // Reactive signals for collapsible sections
    let dashboard_rw = RwSignal::new(false);
    let pengadaan_rw = RwSignal::new(false);
    let pemeliharaan_rw = RwSignal::new(false);
    let pengembangan_rw = RwSignal::new(false);
    let laporan_rw = RwSignal::new(false);
    let pengguna_rw = RwSignal::new(false);
    let bantuan_rw = RwSignal::new(false);

    view! {
        <div class="h-full bg-white border-r border-gray-200 w-64 overflow-y-auto">
            <div class="p-6 border-b border-gray-200">
                <div class="flex items-center space-x-3">
                    <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold">
                        "P"
                    </div>
                    <h1 class="text-xl font-bold text-gray-900">"SIMPEL PERENCANAAN"</h1>
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
                    title="Rencana Pengadaan".to_string()
                    icon="🛒".to_string()
                    is_expanded=pengadaan_rw
                    items=vec![
                        MenuItem::new("/pengadaan/daftar", "Daftar Pengadaan"),
                        MenuItem::new("/pengadaan/rencana", "Rencana Pengadaan"),
                        MenuItem::new("/pengadaan/prioritas", "Prioritas Pengadaan"),
                        MenuItem::new("/pengadaan/anggaran", "Anggaran Pengadaan"),
                        MenuItem::new("/pengadaan/monitoring", "Monitoring Pengadaan")
                    ]
                />

                <SidebarSection
                    title="Rencana Pemeliharaan".to_string()
                    icon="🔧".to_string()
                    is_expanded=pemeliharaan_rw
                    items=vec![
                        MenuItem::new("/pemeliharaan/rutin", "Pemeliharaan Rutin"),
                        MenuItem::new("/pemeliharaan/berkala", "Pemeliharaan Berkala"),
                        MenuItem::new("/pemeliharaan/darurat", "Pemeliharaan Darurat"),
                        MenuItem::new("/pemeliharaan/jadwal", "Jadwal Pemeliharaan"),
                        MenuItem::new("/pemeliharaan/laporan", "Laporan Pemeliharaan")
                    ]
                />

                <SidebarSection
                    title="Rencana Pengembangan".to_string()
                    icon="📈".to_string()
                    is_expanded=pengembangan_rw
                    items=vec![
                        MenuItem::new("/pengembangan/strategis", "Rencana Strategis"),
                        MenuItem::new("/pengembangan/taktis", "Rencana Taktis"),
                        MenuItem::new("/pengembangan/operasional", "Rencana Operasional"),
                        MenuItem::new("/pengembangan/kapabilitas", "Pengembangan Kapabilitas"),
                        MenuItem::new("/pengembangan/evaluasi", "Evaluasi Pengembangan")
                    ]
                />

                <SidebarSection
                    title="Laporan Perencanaan".to_string()
                    icon="📋".to_string()
                    is_expanded=laporan_rw
                    items=vec![
                        MenuItem::new("/laporan/perencanaan", "Laporan Perencanaan"),
                        MenuItem::new("/laporan/kinerja", "Laporan Kinerja"),
                        MenuItem::new("/laporan/audit", "Laporan Audit"),
                        MenuItem::new("/laporan/bulanan", "Laporan Bulanan"),
                        MenuItem::new("/laporan/tahunan", "Laporan Tahunan")
                    ]
                />

                <SidebarSection
                    title="Pengguna".to_string()
                    icon="👥".to_string()
                    is_expanded=pengguna_rw
                    items=vec![
                        MenuItem::new("/pengguna/manajemen", "Manajemen User"),
                        MenuItem::new("/pengguna/akses", "Hak Akses"),
                        MenuItem::new("/pengguna/audit", "Audit Log"),
                        MenuItem::new("/pengguna/profil", "Profil Pengguna"),
                        MenuItem::new("/pengguna/aktivitas", "Log Aktivitas")
                    ]
                />

                <SidebarSection
                    title="Bantuan".to_string()
                    icon="❓".to_string()
                    is_expanded=bantuan_rw
                    items=vec![
                        MenuItem::new("/bantuan/panduan", "Panduan Pengguna"),
                        MenuItem::new("/bantuan/faq", "FAQ"),
                        MenuItem::new("/bantuan/helpdesk", "Helpdesk"),
                        MenuItem::new("/bantuan/kontak", "Kontak Support"),
                        MenuItem::new("/bantuan/tutorial", "Tutorial")
                    ]
                />
            </nav>
        </div>
    }
}
