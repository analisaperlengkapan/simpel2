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
    division: Option<String>,
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
        <div class="container mx-auto px-4 py-8 max-w-4xl">
            // Header
            <div class="mb-8">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2">
                    "Profil Pengguna"
                </h1>
                <p class="text-gray-600 dark:text-gray-400">
                    "Informasi akun dan data kepegawaian Anda"
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="flex items-center justify-center py-12">
                    <div class="animate-spin w-8 h-8 border-4 border-primary-500 border-t-transparent rounded-full"></div>
                    <span class="ml-3 text-gray-600 dark:text-gray-400">"Memuat profil..."</span>
                </div>
            }>
                {move || {
                    profile.get().map(|result| match result {
                        Ok(data) => view! {
                            <div class="space-y-6">
                                // Informasi Pegawai (Read-only)
                                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                                    <div class="flex items-center mb-6">
                                        <div class="w-12 h-12 bg-gradient-to-br from-blue-500 to-blue-600 rounded-lg flex items-center justify-center mr-4">
                                            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-xl font-bold text-gray-900 dark:text-white">"Informasi Pegawai"</h2>
                                            <p class="text-sm text-gray-500 dark:text-gray-400">"Data dari MySIMKARI (hanya baca)"</p>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <ProfileField label="NIP" value=data.nip.clone().or_else(|| Some(data.username.clone())).unwrap_or_default() />
                                        <ProfileField label="Nama Lengkap" value=data.nama.clone().or(data.name.clone()).unwrap_or("-".into()) />
                                        <ProfileField label="Jabatan" value=data.jabatan.clone().unwrap_or("-".into()) />
                                        <ProfileField label="Satuan Kerja" value=data.division.clone().unwrap_or("-".into()) />
                                    </div>
                                </div>

                                // Informasi Akun
                                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                                    <div class="flex items-center mb-6">
                                        <div class="w-12 h-12 bg-gradient-to-br from-green-500 to-green-600 rounded-lg flex items-center justify-center mr-4">
                                            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-xl font-bold text-gray-900 dark:text-white">"Informasi Akun"</h2>
                                            <p class="text-sm text-gray-500 dark:text-gray-400">"Detail kontak dan akun"</p>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <ProfileField label="Email" value=data.email.clone() />
                                        <ProfileField label="Telepon" value=data.phone.clone().unwrap_or("-".into()) />
                                        <ProfileField label="Role" value=data.role.clone() />
                                        <ProfileField label="Email Terverifikasi" value=if data.email_verified { "Ya".into() } else { "Belum".into() } />
                                    </div>
                                </div>

                                // Keamanan
                                <div class="bg-white dark:bg-gray-800 rounded-2xl shadow-lg p-6 border border-gray-100 dark:border-gray-700">
                                    <div class="flex items-center mb-6">
                                        <div class="w-12 h-12 bg-gradient-to-br from-red-500 to-red-600 rounded-lg flex items-center justify-center mr-4">
                                            <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                            </svg>
                                        </div>
                                        <div>
                                            <h2 class="text-xl font-bold text-gray-900 dark:text-white">"Keamanan"</h2>
                                            <p class="text-sm text-gray-500 dark:text-gray-400">"Status keamanan akun Anda"</p>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"MFA"</p>
                                            <div class="flex items-center mt-1">
                                                {if data.mfa_enabled {
                                                    view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">
                                                            "Aktif"
                                                        </span>
                                                    }
                                                } else {
                                                    view! {
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300">
                                                            "Belum Aktif"
                                                        </span>
                                                    }
                                                }}
                                            </div>
                                        </div>
                                        <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
                                            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">"Ganti Password"</p>
                                            <div class="mt-1">
                                                <a href="/password" class="text-sm text-primary-600 hover:text-primary-700 dark:text-primary-400 font-medium">
                                                    "Ubah Password"
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),
                        Err(err) => view! {
                            <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-xl p-4">
                                <p class="text-red-700 dark:text-red-300">{format!("Gagal memuat profil: {}", err)}</p>
                            </div>
                        }.into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

/// A read-only profile field
#[component]
fn ProfileField(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <p class="text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">{label}</p>
            <p class="text-sm font-semibold text-gray-900 dark:text-white mt-1">{value}</p>
        </div>
    }
}
