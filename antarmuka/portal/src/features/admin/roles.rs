//! Roles Management Page (Admin)
//!
//! Read-only view of the platform's fixed, seeded role model (admin +
//! operator_satker + validator_wilayah + validator_pusat), backed by
//! `GET /api/v1/iam/roles`. Role create/delete was removed with the #45
//! IAM trim — the role set is seeded configuration, and per-user
//! assignment lives on the user detail page ("Pemetaan Peran").
//! REQ-PORTAL-013

use crate::components::feedback::{EmptyPanel, ErrorBanner, LoadingPanel};
use crate::components::layout::main_layout::MainLayout;
use crate::utils::app_state::{use_api_client, use_main_layout_session_and_logout};
use crate::utils::async_load::load_vec_once;
use crate::utils::authenc_api::RoleInfo;
use leptos::prelude::*;

/// Roles management page
#[component]
pub fn RolesManagementPage() -> impl IntoView {
    let api = use_api_client();

    let (roles, set_roles) = signal(Vec::<RoleInfo>::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal(Option::<String>::None);

    {
        let api = api.clone();
        Effect::new(move || {
            let api = api.clone();
            load_vec_once(
                move || {
                    let api = api.clone();
                    async move { api.iam_list_roles().await }
                },
                set_roles,
                set_error,
                set_loading,
                "Gagal memuat peran",
            );
        });
    }

    let (session, on_logout) = use_main_layout_session_and_logout();

    view! {
        <MainLayout user_session=session.clone() on_logout=on_logout>
            <div class="max-w-7xl mx-auto px-4 py-8">
                <nav class="text-sm text-gray-500 mb-1">
                    <a href="/portal/admin" class="hover:text-primary-600">
                        "Admin"
                    </a>
                    " / Peran"
                </nav>
                <div class="flex items-center justify-between mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">"Manajemen Peran & Hak Akses"</h1>
                    <p class="text-sm text-gray-500">
                        "Penetapan peran per pengguna dilakukan dari halaman detail pengguna"
                    </p>
                </div>

                {move || error.get().map(|msg| view! { <ErrorBanner message=msg /> })}

                <Show
                    when=move || !loading.get()
                    fallback=|| view! { <LoadingPanel message="Memuat data peran..." /> }
                >
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                        <For
                            each=move || roles.get()
                            key=|r| r.id.clone()
                            children=move |role| {
                                let perm_count = role.permissions.len();
                                let user_count = role.user_count.unwrap_or(0);
                                view! {
                                    <div
                                        class="bg-white rounded-xl border border-gray-200 p-5"
                                        data-role-name=role.name.clone()
                                    >
                                        <h3 class="font-semibold text-gray-900 mb-2">
                                            {role.name.clone()}
                                        </h3>
                                        <p class="text-sm text-gray-500 mb-3">
                                            {role
                                                .description
                                                .clone()
                                                .unwrap_or_else(|| "Tanpa deskripsi".to_string())}
                                        </p>
                                        <div class="flex items-center gap-4 text-xs text-gray-400">
                                            <span>
                                                "🛡️ " {format!("{} permission", perm_count)}
                                            </span>
                                            <span>"👤 " {format!("{} pengguna", user_count)}</span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                    <Show when=move || roles.get().is_empty()>
                        <EmptyPanel
                            title="Belum ada peran"
                            message="Data peran belum tersedia pada realm aktif."
                        />
                    </Show>
                </Show>
            </div>
        </MainLayout>
    }
}
