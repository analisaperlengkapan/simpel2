//! # Admin Roles Reference
//!
//! Static reference page describing the perlengkapan role model.
//!
//! NOTE: the user-management page that used to live here has been removed. It
//! called `/admin/users*`, which queried `v_user_role_summary`,
//! `perlengkapan_users` and `perlengkapan_user_roles` — relations no migration
//! creates — so it rendered an empty table in every environment (the fetch
//! error was swallowed into an empty list, which is why it read as "no users"
//! rather than "broken"). It is not being rebuilt here because user and role
//! administration belongs to authenc, the IAM source of truth; portal already
//! provides it against the real backend.

use leptos::prelude::*;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{CHECK, USER_CHECK};

use crate::components::role_switcher::PerlengkapanRole;

/// Admin Roles Configuration Page
#[component]
pub fn AdminRolesPage() -> impl IntoView {
    let roles = PerlengkapanRole::all_roles();

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-slate-100 flex items-center gap-3">
                    <div class="w-10 h-10 bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl flex items-center justify-center shadow-lg">
                        <span class="text-white">
                            <AppIcon icon=USER_CHECK />
                        </span>
                    </div>
                    "Pengaturan Role"
                </h1>
                <p class="text-slate-400 mt-1">"Konfigurasi role dan hak akses perlengkapan"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                {roles
                    .into_iter()
                    .map(|role| {
                        let color = role.color.clone();
                        let bg_gradient = match color.as_str() {
                            "blue" => "from-blue-500 to-blue-600",
                            "amber" => "from-amber-500 to-amber-600",
                            "emerald" => "from-emerald-500 to-emerald-600",
                            "red" => "from-red-500 to-red-600",
                            _ => "from-gray-500 to-gray-600",
                        };
                        view! {
                            <div class="overflow-hidden rounded-xl border border-white/[0.06] bg-surface-panel shadow-card transition-shadow hover:border-white/10">
                                <div class=format!(
                                    "bg-gradient-to-r {} px-6 py-4 text-white",
                                    bg_gradient,
                                )>
                                    <div class="flex items-center gap-3">
                                        <AppIcon icon=icon_from_fa_class(&role.icon) size=20 />
                                        <div>
                                            <h3 class="font-bold">{role.label}</h3>
                                            <p class="text-sm opacity-90">{role.description}</p>
                                        </div>
                                    </div>
                                </div>
                                <div class="p-4">
                                    <h4 class="text-sm font-semibold text-slate-200 mb-2">
                                        "Hak Akses:"
                                    </h4>
                                    <div class="space-y-1 text-xs text-slate-400">
                                        {match role.key.as_str() {
                                            "operator_satker" => {
                                                view! {
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Kebutuhan BMN: Buat, Edit, Hapus, Submit"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Pemakaian BMN: Buat, Edit, Submit"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Penghapusan BMN: Buat, Submit"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Pakaian Dinas: Input Ukuran"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Pemeliharaan: Buat, Edit"
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            "validator_wilayah" => {
                                                view! {
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Kebutuhan BMN: Validasi, Kembalikan, Teruskan"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Pemakaian BMN: Monitor (read-only)"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Penghapusan BMN: Validasi, Teruskan; Generate & Upload Signed SK (kewenangan Wilayah)"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Laporan: Baca, Ekspor"
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            "validator_pusat" => {
                                                view! {
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Kebutuhan BMN: Persetujuan Akhir (Pengguna Barang)"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Pemakaian BMN: Monitor (read-only)"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Penghapusan BMN: Generate Konsep SK, Upload Signed SK (kewenangan Pusat)"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Analisis Kebutuhan: Approve"
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            "admin" => {
                                                view! {
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Semua akses Operator, Validator Wilayah, Validator Pusat"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Manajemen Pengguna & Role"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Konfigurasi Sistem"
                                                    </div>
                                                    <div>
                                                        <span class="text-success-400 mr-2">
                                                            <AppIcon icon=CHECK />
                                                        </span>
                                                        "Master Data & Audit Log"
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            _ => view! { <div>"N/A"</div> }.into_any(),
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}
