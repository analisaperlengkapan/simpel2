//! # Admin User Management Component
//!
//! Admin panel for managing perlengkapan users and their roles.
//! Data is fetched from MySIMKARI via the integrasi gRPC service
//! through the perlengkapan backend REST API.

use leptos::prelude::*;

use crate::api;
use crate::components::role_switcher::{get_active_role, PerlengkapanRole};

/// User role assignment data
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct UserRoleAssignment {
    pub nip: String,
    pub nama: String,
    pub jabatan: String,
    pub golongan: String,
    pub satker_code: String,
    pub satker_name: String,
    pub assigned_roles: Vec<String>,
    pub active_role: Option<String>,
    pub status: String,
}

/// Admin Users Management Page
#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let active_role = get_active_role();
    let is_admin = active_role == "admin";

    if !is_admin {
        return view! {
            <div class="bg-red-50 border border-red-200 rounded-xl p-8 text-center">
                <i class="fas fa-lock text-4xl text-red-400 mb-4"></i>
                <h2 class="text-xl font-bold text-red-700 mb-2">"Akses Ditolak"</h2>
                <p class="text-red-600">"Anda tidak memiliki akses Admin."</p>
            </div>
        }.into_any();
    }

    let (search, set_search) = signal(String::new());
    let (selected_role_filter, set_role_filter) = signal::<Option<String>>(None);
    let (show_assign_modal, set_show_assign_modal) = signal(false);
    let (selected_user_nip, set_selected_user_nip) = signal::<Option<String>>(None);

    // Simulated user data (in production, fetched from /api/pembinaan/perlengkapan/admin/users)
    let users_resource = LocalResource::new(move || {
        let _search = search.get();
        let _role = selected_role_filter.get();
        async move {
            // In production: api::fetch_admin_users(search, role_filter).await
            vec![
                UserRoleAssignment {
                    nip: "199203142014031001".to_string(),
                    nama: "Admin".to_string(),
                    jabatan: "Kasubag Perlengkapan".to_string(),
                    golongan: "III/c".to_string(),
                    satker_code: "0100000".to_string(),
                    satker_name: "Kejaksaan Agung RI".to_string(),
                    assigned_roles: vec![
                        "operator_satker".to_string(),
                        "validator_wilayah".to_string(),
                        "validator_pusat".to_string(),
                        "admin".to_string(),
                    ],
                    active_role: Some("admin".to_string()),
                    status: "active".to_string(),
                },
            ]
        }
    });

    let all_roles = PerlengkapanRole::all_roles();

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 flex items-center gap-3">
                        <div class="w-10 h-10 bg-gradient-to-br from-red-500 to-red-600 rounded-xl flex items-center justify-center shadow-lg">
                            <i class="fas fa-users-cog text-white"></i>
                        </div>
                        "Manajemen Pengguna"
                    </h1>
                    <p class="text-gray-600 mt-1">"Kelola pengguna dan role akses perlengkapan"</p>
                </div>
                <button
                    class="px-4 py-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg hover:shadow-lg transition-all flex items-center gap-2"
                    on:click=move |_| {
                        set_selected_user_nip.set(None);
                        set_show_assign_modal.set(true);
                    }
                >
                    <i class="fas fa-user-plus"></i>
                    "Tambah Pengguna"
                </button>
            </div>

            // Filters
            <div class="bg-white rounded-xl shadow-sm border p-4">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    // Search by NIP/Nama
                    <div class="relative">
                        <i class="fas fa-search absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
                        <input
                            type="text"
                            placeholder="Cari NIP atau Nama..."
                            class="w-full pl-10 pr-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            prop:value=search
                            on:input=move |ev| set_search.set(event_target_value(&ev))
                        />
                    </div>

                    // Role filter
                    <select
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500"
                        on:change=move |ev| {
                            let v = event_target_value(&ev);
                            set_role_filter.set(if v.is_empty() { None } else { Some(v) });
                        }
                    >
                        <option value="">"Semua Role"</option>
                        {all_roles.iter().map(|r| {
                            let key = r.key.clone();
                            let label = r.label.clone();
                            view! { <option value=key>{label}</option> }
                        }).collect::<Vec<_>>()}
                    </select>

                    // Stats
                    <div class="flex items-center gap-4 text-sm text-gray-600">
                        <span class="flex items-center gap-1">
                            <i class="fas fa-users text-blue-500"></i>
                            "Total: 1 pengguna"
                        </span>
                    </div>
                </div>
            </div>

            // Users table
            <div class="bg-white rounded-xl shadow-sm border overflow-hidden">
                <div class="overflow-x-auto">
                    <table class="w-full">
                        <thead class="bg-gray-50 border-b">
                            <tr>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"NIP"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"Nama"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"Jabatan"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"Satker"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"Role"</th>
                                <th class="px-4 py-3 text-left text-xs font-semibold text-gray-600 uppercase">"Status"</th>
                                <th class="px-4 py-3 text-center text-xs font-semibold text-gray-600 uppercase">"Aksi"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-100">
                            <Suspense fallback=move || view! {
                                <tr><td colspan="7" class="px-4 py-8 text-center text-gray-500">
                                    <i class="fas fa-spinner fa-spin mr-2"></i>"Memuat data..."
                                </td></tr>
                            }>
                                {move || users_resource.get().map(|users| {
                                    users.into_iter().map(|user| {
                                        let nip = user.nip.clone();
                                        let nip_for_edit = user.nip.clone();
                                        let nip_display = user.nip.clone();
                                        let nama = user.nama.clone();
                                        let jabatan = user.jabatan.clone();
                                        let satker_name = user.satker_name.clone();
                                        let assigned_roles = user.assigned_roles.clone();
                                        view! {
                                            <tr class="hover:bg-gray-50 transition-colors">
                                                <td class="px-4 py-3 text-sm font-mono text-gray-800">{nip_display}</td>
                                                <td class="px-4 py-3 text-sm font-medium text-gray-900">{nama}</td>
                                                <td class="px-4 py-3 text-sm text-gray-600">{jabatan}</td>
                                                <td class="px-4 py-3 text-sm text-gray-600">{satker_name}</td>
                                                <td class="px-4 py-3">
                                                    <div class="flex flex-wrap gap-1">
                                                        {assigned_roles.iter().map(|role| {
                                                            let (bg, text) = match role.as_str() {
                                                                "operator_satker" => ("bg-blue-100", "text-blue-700"),
                                                                "validator_wilayah" => ("bg-amber-100", "text-amber-700"),
                                                                "validator_pusat" => ("bg-emerald-100", "text-emerald-700"),
                                                                "admin" => ("bg-red-100", "text-red-700"),
                                                                _ => ("bg-gray-100", "text-gray-700"),
                                                            };
                                                            let short_label = match role.as_str() {
                                                                "operator_satker" => "Op. Satker".to_string(),
                                                                "validator_wilayah" => "Val. Wilayah".to_string(),
                                                                "validator_pusat" => "Val. Pusat".to_string(),
                                                                "admin" => "Admin".to_string(),
                                                                _ => role.clone(),
                                                            };
                                                            view! {
                                                                <span class=format!("px-2 py-0.5 text-[10px] font-semibold rounded-full {} {}", bg, text)>
                                                                    {short_label}
                                                                </span>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </td>
                                                <td class="px-4 py-3">
                                                    <span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-700">
                                                        "Aktif"
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-center">
                                                    <button
                                                        class="px-3 py-1 text-xs bg-blue-50 text-blue-600 rounded-lg hover:bg-blue-100 transition-colors"
                                                        on:click=move |_| {
                                                            set_selected_user_nip.set(Some(nip_for_edit.clone()));
                                                            set_show_assign_modal.set(true);
                                                        }
                                                    >
                                                        <i class="fas fa-edit mr-1"></i>"Edit Role"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()
                                })}
                            </Suspense>
                        </tbody>
                    </table>
                </div>
            </div>

            // Role assignment modal
            {move || show_assign_modal.get().then(|| view! {
                <div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4">
                    <div class="bg-white rounded-2xl shadow-2xl w-full max-w-lg">
                        <div class="px-6 py-4 border-b flex items-center justify-between">
                            <h3 class="text-lg font-bold text-gray-900">"Atur Role Pengguna"</h3>
                            <button
                                class="p-2 hover:bg-gray-100 rounded-lg"
                                on:click=move |_| set_show_assign_modal.set(false)
                            >
                                <i class="fas fa-times text-gray-500"></i>
                            </button>
                        </div>
                        <div class="p-6 space-y-4">
                            <p class="text-sm text-gray-600">"Pilih role yang akan diberikan kepada pengguna ini."</p>
                            {PerlengkapanRole::all_roles().into_iter().map(|role| {
                                view! {
                                    <label class="flex items-center gap-3 p-3 rounded-lg border hover:bg-gray-50 cursor-pointer transition-colors">
                                        <input type="checkbox" checked=true class="w-4 h-4 text-blue-600 rounded" />
                                        <div>
                                            <span class="text-sm font-medium text-gray-800">{role.label}</span>
                                            <p class="text-xs text-gray-500">{role.description}</p>
                                        </div>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <div class="px-6 py-4 border-t flex justify-end gap-3">
                            <button
                                class="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                                on:click=move |_| set_show_assign_modal.set(false)
                            >
                                "Batal"
                            </button>
                            <button
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                                on:click=move |_| set_show_assign_modal.set(false)
                            >
                                "Simpan"
                            </button>
                        </div>
                    </div>
                </div>
            })}
        </div>
    }.into_any()
}

/// Admin Roles Configuration Page
#[component]
pub fn AdminRolesPage() -> impl IntoView {
    let roles = PerlengkapanRole::all_roles();

    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 flex items-center gap-3">
                    <div class="w-10 h-10 bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl flex items-center justify-center shadow-lg">
                        <i class="fas fa-user-shield text-white"></i>
                    </div>
                    "Pengaturan Role"
                </h1>
                <p class="text-gray-600 mt-1">"Konfigurasi role dan hak akses perlengkapan"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                {roles.into_iter().map(|role| {
                    let color = role.color.clone();
                    let bg_gradient = match color.as_str() {
                        "blue" => "from-blue-500 to-blue-600",
                        "amber" => "from-amber-500 to-amber-600",
                        "emerald" => "from-emerald-500 to-emerald-600",
                        "red" => "from-red-500 to-red-600",
                        _ => "from-gray-500 to-gray-600",
                    };
                    view! {
                        <div class="bg-white rounded-xl shadow-sm border overflow-hidden hover:shadow-md transition-shadow">
                            <div class=format!("bg-gradient-to-r {} px-6 py-4 text-white", bg_gradient)>
                                <div class="flex items-center gap-3">
                                    <i class=format!("{} text-xl", role.icon)></i>
                                    <div>
                                        <h3 class="font-bold">{role.label}</h3>
                                        <p class="text-sm opacity-90">{role.description}</p>
                                    </div>
                                </div>
                            </div>
                            <div class="p-4">
                                <h4 class="text-sm font-semibold text-gray-700 mb-2">"Hak Akses:"</h4>
                                <div class="space-y-1 text-xs text-gray-600">
                                    {match role.key.as_str() {
                                        "operator_satker" => view! {
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Kebutuhan BMN: Buat, Edit, Hapus, Submit"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Pemakaian BMN: Buat, Edit, Submit"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Penghapusan BMN: Buat, Submit"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Pakaian Dinas: Input Ukuran"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Pemeliharaan: Buat, Edit"</div>
                                        }.into_any(),
                                        "validator_wilayah" => view! {
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Kebutuhan BMN: Validasi, Kembalikan, Teruskan"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Pemakaian BMN: Setujui, Tolak"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Penghapusan BMN: Validasi, Teruskan"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Laporan: Baca, Ekspor"</div>
                                        }.into_any(),
                                        "validator_pusat" => view! {
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Kebutuhan BMN: Persetujuan Akhir"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Penghapusan BMN: SK, Persetujuan"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Mapping Kodefikasi: Approve/Reject"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Roadmap Sarpras: Approve"</div>
                                        }.into_any(),
                                        "admin" => view! {
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Semua akses Operator, Validator Wilayah, Validator Pusat"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Manajemen Pengguna & Role"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Konfigurasi Sistem"</div>
                                            <div><i class="fas fa-check text-green-500 mr-2"></i>"Master Data & Audit Log"</div>
                                        }.into_any(),
                                        _ => view! { <div>"N/A"</div> }.into_any(),
                                    }}
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
