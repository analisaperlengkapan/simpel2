//! SIMPelv2 Portal - Simple Version with Shared Library
//!
//! Simplified portal implementation using shared components

use leptos::prelude::*;
use leptos::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

// Import shared components
use shared_microfrontend::{KejaksaanHeader, KejaksaanFooter, NavItem, LoadingSpinner, PageLayout};

// System Data Models for Portal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub description: String,
    pub status: String,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub username: String,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub last_login: Option<String>,
}

// Simple Portal App using shared components
#[component]
pub fn App() -> impl IntoView {
    // System information for Portal
    let systems = vec![
        SystemInfo {
            name: "Badiklat".to_string(),
            url: "/badiklat".to_string(),
            icon: "fa-graduation-cap".to_string(),
            description: "Sistem Pendidikan dan Pelatihan".to_string(),
            status: "active".to_string(),
            color: "blue".to_string(),
        },
        SystemInfo {
            name: "Intel".to_string(),
            url: "/intel".to_string(),
            icon: "fa-search".to_string(),
            description: "Sistem Intelijen Kejaksaan".to_string(),
            status: "active".to_string(),
            color: "green".to_string(),
        },
        SystemInfo {
            name: "Datun".to_string(),
            url: "/datun".to_string(),
            icon: "fa-gavel".to_string(),
            description: "Sistem Perdata dan Tata Usaha Negara".to_string(),
            status: "active".to_string(),
            color: "purple".to_string(),
        },
    ];

    view! {
        <Html lang="id" />
        <Title text="Portal SIMPelv2" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Meta name="description" content="Portal Sistem Informasi Manajemen Pengelolaan Barang Milik Negara v2.0" />

        <body class="bg-gray-50 min-h-screen">
            <KejaksaanHeader />

            <main class="container mx-auto px-4 py-8">
                <div class="text-center mb-12">
                    <h1 class="text-4xl font-bold text-gray-900 mb-4">
                        "Portal SIMPelv2"
                    </h1>
                    <p class="text-lg text-gray-600 max-w-2xl mx-auto">
                        "Sistem Informasi Manajemen Pengelolaan Barang Milik Negara"<br/>
                        "Kejaksaan Agung Republik Indonesia"
                    </p>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {systems.into_iter().map(|system| {
                        view! {
                            <SystemCard system=system />
                        }
                    }).collect::<Vec<_>>()}
                </div>

                <div class="mt-16 text-center">
                    <div class="bg-white rounded-lg shadow-sm border p-8">
                        <div class="flex items-center justify-center mb-4">
                            <div class="w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center">
                                <i class="fas fa-info-circle text-blue-600 text-xl"></i>
                            </div>
                        </div>
                        <h3 class="text-lg font-semibold text-gray-900 mb-2">"Informasi Sistem"</h3>
                        <p class="text-gray-600 text-sm">
                            "SIMPelv2 menggunakan teknologi modern dengan Rust dan Leptos untuk"<br/>
                            "performance optimal dan keamanan tinggi."
                        </p>
                        <div class="mt-4 flex justify-center space-x-4 text-xs text-gray-500">
                            <span>"Leptos 0.7.8"</span>
                            <span>"•"</span>
                            <span>"Rust WASM"</span>
                            <span>"•"</span>
                            <span>"TailwindCSS"</span>
                        </div>
                    </div>
                </div>
            </main>

            <KejaksaanFooter />
        </body>
    }
}

#[component]
pub fn SystemCard(system: SystemInfo) -> impl IntoView {
    let url = system.url.clone();
    let color = system.color.clone();

    view! {
        <a
            href=url
            target="_blank"
            rel="noopener noreferrer"
            class="block bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md hover:border-blue-300 transition-all duration-200 p-6"
        >
            <div class={format!("w-12 h-12 bg-{}-100 rounded-lg flex items-center justify-center mb-4", color)}>
                <i class={format!("fas {} text-{}-600 text-xl", system.icon, color)}></i>
            </div>
            <h3 class="text-lg font-semibold text-gray-900 mb-2">{system.name}</h3>
            <p class="text-gray-600 text-sm">{system.description}</p>
            <div class="mt-4 inline-flex items-center text-sm text-blue-600">
                "Akses Sistem" <i class="fas fa-external-link-alt ml-1"></i>
            </div>
        </a>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="text-center py-16">
            <h1 class="text-6xl font-bold text-gray-400">"404"</h1>
            <p class="text-xl text-gray-600 mt-4">"Halaman tidak ditemukan"</p>
            <a href="/" class="inline-block mt-6 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors">
                "Kembali ke Beranda"
            </a>
        </div>
    }
}
