use leptos::prelude::*;
use crate::components::sidebar_section::{SidebarSection, MenuItem};

#[component]
pub fn Sidebar() -> impl IntoView {
    // Reactive signals for collapsible sections
    let dashboard_rw = RwSignal::new(false);
    let bank_aset_rw = RwSignal::new(false);
    let analisis_rw = RwSignal::new(false);
    let pengadaan_rw = RwSignal::new(false);
    let bmn_rw = RwSignal::new(false);
    let pengguna_rw = RwSignal::new(false);
    let bantuan_rw = RwSignal::new(false);

    view! {
        <div class="h-full bg-white border-r border-gray-200 w-64 overflow-y-auto">
            <div class="p-6 border-b border-gray-200">
                <div class="flex items-center space-x-3">
                    <div class="w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold">
                        "S"
                    </div>
                    <h1 class="text-xl font-bold text-gray-900">"SIMPEL PERLENGKAPAN"</h1>
                </div>
                <p class="text-sm text-gray-500 mt-1">"Kejaksaan Republik Indonesia"</p>
            </div>

            <nav class="p-6 space-y-4">
                <SidebarSection
                    title="Dashboard".to_string()
                    icon="🏠".to_string()
                    is_expanded=dashboard_rw
                    items=vec![
                        MenuItem::new("/dashboard/overview", "Overview"),
                        MenuItem::new("/dashboard/analytics", "Analytics")
                    ]
                />

                <SidebarSection
                    title="Bank Aset".to_string()
                    icon="🏦".to_string()
                    is_expanded=bank_aset_rw
                    items=vec![
                        MenuItem::new("/dashboard/bank-aset/daftar", "Daftar Aset"),
                        MenuItem::new("/dashboard/bank-aset/kategori", "Kategori Aset"),
                        MenuItem::new("/dashboard/bank-aset/lokasi", "Lokasi Aset"),
                        MenuItem::new("/dashboard/bank-aset/status", "Status Aset")
                    ]
                />

                <SidebarSection
                    title="Analisis Kebutuhan".to_string()
                    icon="📊".to_string()
                    is_expanded=analisis_rw
                    items=vec![
                        MenuItem::new("/dashboard/analisis/perencanaan", "Perencanaan"),
                        MenuItem::new("/dashboard/analisis/evaluasi", "Evaluasi"),
                        MenuItem::new("/dashboard/analisis/rekomendasi", "Rekomendasi")
                    ]
                />

                <SidebarSection
                    title="Pengadaan".to_string()
                    icon="🛒".to_string()
                    is_expanded=pengadaan_rw
                    items=vec![
                        MenuItem::new("/dashboard/pengadaan/rencana", "Rencana Pengadaan"),
                        MenuItem::new("/dashboard/pengadaan/proses", "Proses Pengadaan"),
                        MenuItem::new("/dashboard/pengadaan/monitoring", "Monitoring")
                    ]
                />

                <SidebarSection
                    title="Pengelolaan BMN".to_string()
                    icon="⚙️".to_string()
                    is_expanded=bmn_rw
                    items=vec![
                        MenuItem::new("/dashboard/bmn/inventarisasi", "Inventarisasi"),
                        MenuItem::new("/dashboard/bmn/pemeliharaan", "Pemeliharaan"),
                        MenuItem::new("/dashboard/bmn/pemusnahan", "Pemusnahan"),
                        MenuItem::new("/dashboard/bmn/pemanfaatan", "Pemanfaatan")
                    ]
                />

                <SidebarSection
                    title="Pengguna".to_string()
                    icon="👥".to_string()
                    is_expanded=pengguna_rw
                    items=vec![
                        MenuItem::new("/dashboard/pengguna/manajemen", "Manajemen User"),
                        MenuItem::new("/dashboard/pengguna/akses", "Hak Akses"),
                        MenuItem::new("/dashboard/pengguna/audit", "Audit Log")
                    ]
                />

                <SidebarSection
                    title="Bantuan".to_string()
                    icon="❓".to_string()
                    is_expanded=bantuan_rw
                    items=vec![
                        MenuItem::new("/dashboard/bantuan/panduan", "Panduan"),
                        MenuItem::new("/dashboard/bantuan/faq", "FAQ"),
                        MenuItem::new("/dashboard/bantuan/helpdesk", "Helpdesk")
                    ]
                />
            </nav>
        </div>
    }
}
