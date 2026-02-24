use crate::components::role_switcher::get_active_role;
use crate::components::sidebar_section::{MenuItem, SidebarSection};
use leptos::prelude::*;

/// All four perlengkapan roles as string slices for convenience
const ALL_ROLES: &[&str] = &[
    "operator_satker",
    "validator_wilayah",
    "validator_pusat",
    "admin",
];

fn roles(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|s| s.to_string()).collect()
}

#[component]
pub fn Sidebar(
    /// Sidebar open state signal
    sidebar_open: RwSignal<bool>,
) -> impl IntoView {
    let active_role = get_active_role();

    // Reactive signals for collapsible sections
    let dashboard_rw = RwSignal::new(false);
    let bank_aset_rw = RwSignal::new(false);
    let kebutuhan_bmn_rw = RwSignal::new(false);
    let pemakaian_bmn_rw = RwSignal::new(false);
    let pengelolaan_bmn_rw = RwSignal::new(false);
    let pakaian_dinas_rw = RwSignal::new(false);
    let pemeliharaan_rw = RwSignal::new(false);
    let admin_rw = RwSignal::new(false);
    let bantuan_rw = RwSignal::new(false);

    view! {
        // Sidebar panel – fixed left on large screens, slide-over on mobile
        <div
            class=move || format!(
                "fixed inset-y-0 left-0 z-40 w-64 bg-gradient-to-b from-gray-900 via-gray-800 to-gray-900 shadow-xl transition-transform duration-300 transform lg:translate-x-0 lg:static lg:inset-0 flex flex-col {}",
                if sidebar_open.get() { "translate-x-0" } else { "-translate-x-full" }
            )
        >
            // Brand header
            <div class="px-5 py-4 border-b border-white/10 flex-shrink-0">
                <a href="/perlengkapan/dashboard" class="flex items-center gap-3 group">
                    <div class="w-9 h-9 bg-red-600 rounded-lg flex items-center justify-center group-hover:scale-105 transition-transform">
                        <span class="text-white text-sm">"⚖️"</span>
                    </div>
                    <div>
                        <h1 class="text-sm font-bold text-white leading-none tracking-tight">"SIMPEL"</h1>
                        <p class="text-[10px] text-gray-400 leading-none mt-0.5">"Perlengkapan"</p>
                    </div>
                </a>
            </div>

            // Scrollable nav
            <nav class="flex-1 overflow-y-auto px-3 py-4 space-y-1" aria-label="Menu utama perlengkapan">
                // ── Dashboard ── all roles
                <SidebarSection
                    title="Dashboard".to_string()
                    icon=r#"<i class="fas fa-home text-gray-400"></i>"#.to_string()
                    is_expanded=dashboard_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard", "Ringkasan"),
                    ]
                />

                // ── Bank Aset ── all roles
                <SidebarSection
                    title="Bank Aset".to_string()
                    icon=r#"<i class="fas fa-boxes text-red-400"></i>"#.to_string()
                    is_expanded=bank_aset_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/bank-aset/daftar", "Daftar Aset"),
                        MenuItem::new("/perlengkapan/dashboard/bank-aset/peta", "Peta Sebaran"),
                        MenuItem::new("/perlengkapan/dashboard/bank-aset/qr-code", "Cetak QR Code"),
                    ]
                />

                // ── Kebutuhan BMN ── all roles
                <SidebarSection
                    title="Kebutuhan BMN".to_string()
                    icon=r#"<i class="fas fa-clipboard-list text-orange-400"></i>"#.to_string()
                    is_expanded=kebutuhan_bmn_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/kebutuhan-bmn/dashboard", "Dashboard"),
                        MenuItem::new("/perlengkapan/dashboard/kebutuhan-bmn/daftar", "Daftar Kebutuhan"),
                        MenuItem::new("/perlengkapan/dashboard/kebutuhan-bmn/baru", "Buat Baru"),
                    ]
                />

                // ── Pemakaian BMN ── all roles
                <SidebarSection
                    title="Pemakaian BMN".to_string()
                    icon=r#"<i class="fas fa-file-signature text-blue-400"></i>"#.to_string()
                    is_expanded=pemakaian_bmn_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/pemakaian-bmn/daftar", "Daftar Pemakaian"),
                        MenuItem::new("/perlengkapan/dashboard/pemakaian-bmn/baru", "Buat Pengajuan"),
                    ]
                />

                // ── Pengelolaan BMN ── validator_wilayah, validator_pusat, admin
                <SidebarSection
                    title="Pengelolaan BMN".to_string()
                    icon=r#"<i class="fas fa-cogs text-amber-300"></i>"#.to_string()
                    is_expanded=pengelolaan_bmn_rw
                    allowed_roles=roles(&["validator_wilayah", "validator_pusat", "admin"])
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/pengelolaan/pemakaian", "Pemakaian"),
                        MenuItem::new("/perlengkapan/dashboard/pengelolaan/hibah", "Hibah"),
                        MenuItem::new("/perlengkapan/dashboard/pengelolaan/pengalihan", "Pengalihan"),
                        MenuItem::new("/perlengkapan/dashboard/pengelolaan/mutasi", "Mutasi"),
                        MenuItem::new("/perlengkapan/dashboard/pengelolaan/penghapusan", "Penghapusan"),
                    ]
                />

                // ── Pakaian Dinas ── all roles
                <SidebarSection
                    title="Pakaian Dinas".to_string()
                    icon=r#"<i class="fas fa-tshirt text-purple-300"></i>"#.to_string()
                    is_expanded=pakaian_dinas_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/pakaian-dinas/jenis", "Jenis Pakaian"),
                        MenuItem::new("/perlengkapan/dashboard/pakaian-dinas/pengajuan", "Pengajuan"),
                        MenuItem::new("/perlengkapan/dashboard/pakaian-dinas/laporan", "Laporan"),
                    ]
                />

                // ── Pemeliharaan ── operator_satker, admin
                <SidebarSection
                    title="Pemeliharaan".to_string()
                    icon=r#"<i class="fas fa-tools text-teal-300"></i>"#.to_string()
                    is_expanded=pemeliharaan_rw
                    allowed_roles=roles(&["operator_satker", "admin"])
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/pemeliharaan/daftar", "Daftar Pemeliharaan"),
                        MenuItem::new("/perlengkapan/dashboard/pemeliharaan/baru", "Buat Baru"),
                    ]
                />

                // ── Admin ── admin only
                <SidebarSection
                    title="Admin".to_string()
                    icon=r#"<i class="fas fa-user-shield text-red-300"></i>"#.to_string()
                    is_expanded=admin_rw
                    allowed_roles=roles(&["admin"])
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/admin/users", "Manajemen User"),
                        MenuItem::new("/perlengkapan/dashboard/admin/roles", "Manajemen Role"),
                        MenuItem::new("/perlengkapan/dashboard/admin/audit", "Audit Log"),
                        MenuItem::new("/perlengkapan/dashboard/admin/config", "Konfigurasi"),
                        MenuItem::new("/perlengkapan/dashboard/admin/master", "Master Data"),
                    ]
                />

                // ── Bantuan ── all roles
                <SidebarSection
                    title="Bantuan".to_string()
                    icon=r#"<i class="fas fa-question-circle text-gray-300"></i>"#.to_string()
                    is_expanded=bantuan_rw
                    allowed_roles=roles(ALL_ROLES)
                    active_role=active_role.clone()
                    items=vec![
                        MenuItem::new("/perlengkapan/dashboard/bantuan/panduan", "Panduan"),
                        MenuItem::new("/perlengkapan/dashboard/bantuan/faq", "FAQ"),
                        MenuItem::new("/perlengkapan/dashboard/bantuan/helpdesk", "Helpdesk"),
                    ]
                />
            </nav>

            // Footer inside sidebar
            <div class="px-4 py-3 border-t border-white/10 flex-shrink-0">
                <div class="flex items-center gap-2 text-[10px] text-gray-500">
                    <div class="w-1.5 h-1.5 bg-green-400 rounded-full" aria-hidden="true"></div>
                    <span>"SIMPelv2 v2.0"</span>
                </div>
            </div>
        </div>

        // Mobile overlay backdrop
        {move || sidebar_open.get().then(|| view! {
            <div
                class="fixed inset-0 bg-black/50 z-30 lg:hidden"
                on:click=move |_| sidebar_open.set(false)
            ></div>
        })}
    }
}
