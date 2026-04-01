//! Profile page - User profile with data from /api/v1/auth/me

use crate::features::auth::AuthService;
use crate::utils::app_state::use_app_state;
use leptos::prelude::*;
use serde::Deserialize;

/// Profile response from /api/v1/auth/me
#[derive(Clone, Debug, Deserialize)]
struct ProfileData {
    #[allow(dead_code)]
    id: String,
    username: String,
    email: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    nip: Option<String>,
    #[serde(default)]
    nama: Option<String>,
    #[serde(default)]
    jabatan: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    satker_code: Option<String>,
    #[serde(default)]
    satuan_kerja: Option<String>,
    role: String,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    mfa_enabled: bool,
    #[allow(dead_code)]
    #[serde(default)]
    require_password_change: bool,
}

async fn fetch_profile() -> Result<ProfileData, String> {
    let token = AuthService::get_token().ok_or("Tidak terautentikasi")?;

    #[cfg(target_arch = "wasm32")]
    {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        let url = format!("{}/api/v1/auth/me", origin);

        let resp = gloo_net::http::Request::get(&url)
            .header("Authorization", &format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.ok() {
            return Err(format!("Gagal memuat profil ({})", resp.status()));
        }

        resp.json::<ProfileData>().await.map_err(|e| e.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = token;
        Err("Profile fetch not supported outside WASM".into())
    }
}

#[component]
pub fn ProfilePage() -> impl IntoView {
    let _state = use_app_state();
    let profile = LocalResource::new(move || async move { fetch_profile().await });

    view! {
        <div class="max-w-3xl mx-auto px-4 py-8">
            <div class="mb-6">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Profil Pengguna"</h1>
                <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">"Informasi akun dan data kepegawaian Anda"</p>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                    <span class="ml-3 text-sm text-gray-500 dark:text-gray-400">"Memuat profil..."</span>
                </div>
            }>
                {move || {
                    profile.get().map(|result| match result {
                        Ok(data) => {
                            let display_name = data.nama.clone()
                                .or(data.name.clone())
                                .unwrap_or_else(|| data.username.clone());
                            let display_nip = data.nip.clone()
                                .unwrap_or_else(|| data.username.clone());

                            view! {
                                <div class="space-y-5">
                                    // Identity
                                    <ProfileSection title="Identitas" icon_path="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z">
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                            <Field label="NIP" value=display_nip />
                                            <Field label="Nama Lengkap" value=display_name />
                                            <Field label="Jabatan" value=data.jabatan.unwrap_or_else(|| "-".into()) />
                                            <Field label="Satuan Kerja" value=data.satuan_kerja.unwrap_or_else(|| "-".into()) />
                                        </div>
                                    </ProfileSection>

                                    // Account
                                    <ProfileSection title="Akun" icon_path="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z">
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                            <Field label="Email" value=data.email />
                                            <Field label="Telepon" value=data.phone.unwrap_or_else(|| "-".into()) />
                                            <Field label="Role" value=data.role />
                                            <Field label="Email Terverifikasi" value=if data.email_verified { "Ya".into() } else { "Belum".into() } />
                                        </div>
                                    </ProfileSection>

                                    // Security
                                    <ProfileSection title="Keamanan" icon_path="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z">
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                            <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                                <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"MFA"</p>
                                                <div class="mt-1">
                                                    {if data.mfa_enabled {
                                                        view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900/50 dark:text-green-300">
                                                                "Aktif"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-300">
                                                                "Belum Aktif"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            </div>
                                            <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                                <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Password"</p>
                                                <a href="/portal/password" class="text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 font-medium mt-1 inline-block">
                                                    "Ubah Password"
                                                </a>
                                            </div>
                                        </div>
                                    </ProfileSection>
                                </div>
                            }.into_any()
                        },
                        Err(err) => view! {
                            <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-4">
                                <p class="text-sm text-red-700 dark:text-red-300">{format!("Gagal memuat profil: {}", err)}</p>
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ProfileSection(
    title: &'static str,
    icon_path: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-5">
            <div class="flex items-center gap-3 mb-4">
                <div class="w-9 h-9 bg-blue-50 dark:bg-blue-900/20 rounded-lg flex items-center justify-center">
                    <svg class="w-4.5 h-4.5 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
                    </svg>
                </div>
                <h2 class="text-base font-semibold text-gray-900 dark:text-white">{title}</h2>
            </div>
            {children()}
        </div>
    }
}

#[component]
fn Field(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">{label}</p>
            <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">{value}</p>
        </div>
    }
}
