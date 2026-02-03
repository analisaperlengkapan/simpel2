use crate::components::sidebar_section::{MenuItem, SidebarSection};
use leptos::prelude::*;

#[component]
pub fn Sidebar(
    /// Sidebar open state signal
    _sidebar_open: RwSignal<bool>,
) -> impl IntoView {
    // Reactive signals for collapsible sections
    let dashboard_rw = RwSignal::new(false);
    let bank_aset_rw = RwSignal::new(false);
    let analisis_rw = RwSignal::new(false);
    let pengadaan_rw = RwSignal::new(false);
    let pengelolaan_bmn_rw = RwSignal::new(false);
    let pakaian_dinas_rw = RwSignal::new(false);
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
                        MenuItem::new("/dashboard/overview", "Ringkasan"),
                        MenuItem::new("/dashboard/analytics", "Analitik"),
                        MenuItem::new("/dashboard/reports", "Laporan")
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
                        MenuItem::new("/dashboard/bank-aset/status", "Status Aset"),
                        MenuItem::new("/dashboard/bank-aset/penilaian", "Penilaian Aset"),
                        MenuItem::new("/dashboard/bank-aset/mutasi", "Mutasi Aset")
                    ]
                />

                <SidebarSection
                    title="Analisis Kebutuhan".to_string()
                    icon="📊".to_string()
                    is_expanded=analisis_rw
                    items=vec![
                        MenuItem::new("/dashboard/analisis/perencanaan", "Perencanaan Kebutuhan"),
                        MenuItem::new("/dashboard/analisis/evaluasi", "Evaluasi Kebutuhan"),
                        MenuItem::new("/dashboard/analisis/rekomendasi", "Rekomendasi"),
                        MenuItem::new("/dashboard/analisis/trend", "Analisis Trend"),
                        MenuItem::new("/dashboard/analisis/prediksi", "Prediksi Kebutuhan")
                    ]
                />

                <SidebarSection
                    title="Pengadaan".to_string()
                    icon="🛒".to_string()
                    is_expanded=pengadaan_rw
                    items=vec![
                        MenuItem::new("/dashboard/pengadaan/rencana", "Rencana Pengadaan"),
                        MenuItem::new("/dashboard/pengadaan/proses", "Proses Pengadaan"),
                        MenuItem::new("/dashboard/pengadaan/monitoring", "Monitoring"),
                        MenuItem::new("/dashboard/pengadaan/kontrak", "Kontrak"),
                        MenuItem::new("/dashboard/pengadaan/vendor", "Vendor/Supplier"),
                        MenuItem::new("/dashboard/pengadaan/evaluasi", "Evaluasi Pengadaan")
                    ]
                />

                <SidebarSection
                    title="Pengelolaan BMN".to_string()
                    icon="⚙️".to_string()
                    is_expanded=pengelolaan_bmn_rw
                    items=vec![
                        MenuItem::new("/dashboard/bmn/inventarisasi", "Inventarisasi"),
                        MenuItem::new("/dashboard/bmn/pemeliharaan", "Pemeliharaan"),
                        MenuItem::new("/dashboard/bmn/pemanfaatan", "Pemanfaatan"),
                        MenuItem::new("/dashboard/bmn/pemusnahan", "Pemusnahan"),
                        MenuItem::new("/dashboard/bmn/penghapusan", "Penghapusan"),
                        MenuItem::new("/dashboard/bmn/pengamanan", "Pengamanan"),
                        MenuItem::new("/dashboard/bmn/distribusi", "Distribusi"),
                        MenuItem::new("/dashboard/bmn/pengembalian", "Pengembalian")
                    ]
                />

                <SidebarSection
                    title="Pakaian Dinas".to_string()
                    icon="👔".to_string()
                    is_expanded=pakaian_dinas_rw
                    items=vec![
                        MenuItem::new("/dashboard/pakaian-dinas/jenis", "Jenis Pakaian"),
                        MenuItem::new("/dashboard/pakaian-dinas/pengajuan", "Pengajuan"),
                        MenuItem::new("/dashboard/pakaian-dinas/ukuran-saya", "Ukuran Saya"),
                        MenuItem::new("/dashboard/pakaian-dinas/laporan", "Laporan"),
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
