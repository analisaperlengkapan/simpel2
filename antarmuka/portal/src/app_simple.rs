//! # SIMPelv2 Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPelv2 dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

// Import shared components - Modern Leptos 0.7.8
use shared_microfrontend::prelude::*;

use crate::components::{Header, Footer};

/// Model data untuk sistem yang terintegrasi
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Nama sistem
    pub name: String,
    /// URL akses sistem
    pub url: String,
    /// Deskripsi singkat
    pub description: String,
    /// Status operasional
    pub status: SystemStatus,
    /// Icon sistem
    pub icon: String,
    /// Kategori sistem
    pub category: String,
    /// Level prioritas untuk tampilan
    pub priority: u8,
}

/// Status operasional sistem
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemStatus {
    /// Sistem beroperasi normal
    Online,
    /// Sistem dalam maintenance
    Maintenance,
    /// Sistem mengalami gangguan
    Offline,
}

impl SystemStatus {
    /// Mendapatkan CSS class berdasarkan status
    pub fn css_class(&self) -> &'static str {
        match self {
            SystemStatus::Online => "text-green-600",
            SystemStatus::Maintenance => "text-yellow-600",
            SystemStatus::Offline => "text-red-600",
        }
    }

    /// Mendapatkan label status
    pub fn label(&self) -> &'static str {
        match self {
            SystemStatus::Online => "Online",
            SystemStatus::Maintenance => "Maintenance",
            SystemStatus::Offline => "Offline",
        }
    }
}

/// Komponen utama aplikasi Portal SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    // Data sistem yang tersedia - Prioritas berdasarkan tingkat kepentingan
    let systems = create_system_data();

    view! {
        <Html attr:lang="id"/>
        <Title text="Portal SIMPelv2 - Kejaksaan Agung RI"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Portal Sistem Informasi Manajemen Pengelolaan Barang Milik Negara Kejaksaan Agung Republik Indonesia"/>
        <Meta name="keywords" content="kejaksaan, simpel, bmn, indonesia, government"/>
        <Meta property="og:title" content="Portal SIMPelv2"/>
        <Meta property="og:description" content="Gateway to Justice Technology - Kejaksaan Agung RI"/>

        <div class="min-h-screen bg-kejaksaan-bg">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <HomePage systems=systems/>
            </main>

            <Footer />
        </div>
    }
}

/// Generate data sistem yang tersedia
fn create_system_data() -> Vec<SystemInfo> {
    vec![
        SystemInfo {
            name: "PIDUM".to_string(),
            url: "/pidum".to_string(),
            description: "Sistem Informasi Pidana Umum - Manajemen kasus dan perkara pidana umum".to_string(),
            status: SystemStatus::Online,
            icon: "⚖️".to_string(),
            category: "Pidana".to_string(),
            priority: 1,
        },
        SystemInfo {
            name: "PIDSUS".to_string(),
            url: "/pidsus".to_string(),
            description: "Sistem Informasi Pidana Khusus - Penanganan kasus korupsi, narkoba, dan tindak pidana khusus".to_string(),
            status: SystemStatus::Online,
            icon: "🏛️".to_string(),
            category: "Pidana".to_string(),
            priority: 1,
        },
        SystemInfo {
            name: "PIDMIL".to_string(),
            url: "/pidmil".to_string(),
            description: "Sistem Informasi Pidana Militer - Penanganan perkara pidana militer".to_string(),
            status: SystemStatus::Online,
            icon: "🎖️".to_string(),
            category: "Pidana".to_string(),
            priority: 2,
        },
        SystemInfo {
            name: "DATUN".to_string(),
            url: "/datun".to_string(),
            description: "Sistem Informasi Data dan Tuntutan - Manajemen data perkara dan tuntutan hukum".to_string(),
            status: SystemStatus::Online,
            icon: "📊".to_string(),
            category: "Data".to_string(),
            priority: 1,
        },
        SystemInfo {
            name: "INTEL".to_string(),
            url: "/intel".to_string(),
            description: "Sistem Informasi Intelijen - Analisis dan monitoring keamanan nasional".to_string(),
            status: SystemStatus::Online,
            icon: "🔍".to_string(),
            category: "Intelijen".to_string(),
            priority: 1,
        },
        SystemInfo {
            name: "PENGAWASAN".to_string(),
            url: "/pengawasan".to_string(),
            description: "Sistem Informasi Pengawasan - Monitoring dan evaluasi kinerja internal".to_string(),
            status: SystemStatus::Online,
            icon: "👁️".to_string(),
            category: "Pengawasan".to_string(),
            priority: 2,
        },
        SystemInfo {
            name: "BADIKLAT".to_string(),
            url: "/badiklat".to_string(),
            description: "Sistem Informasi Pendidikan dan Pelatihan - Manajemen SDM dan kapasitas aparatur".to_string(),
            status: SystemStatus::Online,
            icon: "🎓".to_string(),
            category: "SDM".to_string(),
            priority: 2,
        },
        SystemInfo {
            name: "PEMBINAAN".to_string(),
            url: "/pembinaan".to_string(),
            description: "Sistem Informasi Pembinaan - Program pembinaan dan rehabilitasi".to_string(),
            status: SystemStatus::Online,
            icon: "🤝".to_string(),
            category: "Pembinaan".to_string(),
            priority: 3,
        },
        SystemInfo {
            name: "PEMULIHAN ASET".to_string(),
            url: "/pemulihan_aset".to_string(),
            description: "Sistem Informasi Pemulihan Aset - Pelacakan dan pemulihan aset negara".to_string(),
            status: SystemStatus::Maintenance,
            icon: "💰".to_string(),
            category: "Aset".to_string(),
            priority: 2,
        },
    ]
}

/// Halaman utama dengan dashboard sistem
#[component]
pub fn HomePage(
    /// Data sistem yang akan ditampilkan
    systems: Vec<SystemInfo>,
) -> impl IntoView {
    // Kelompokkan sistem berdasarkan kategori
    let grouped_systems = systems
        .into_iter()
        .fold(std::collections::HashMap::<String, Vec<SystemInfo>>::new(), |mut acc, system| {
            acc.entry(system.category.clone()).or_default().push(system);
            acc
        });

    view! {
        <div class="space-y-8">
            // Hero Section
            <div class="text-center py-12 bg-white rounded-lg shadow-sm">
                <div class="text-6xl mb-4">"⚖️"</div>
                <h1 class="text-4xl font-bold text-kejaksaan-text mb-4">
                    "Portal SIMPelv2"
                </h1>
                <p class="text-xl text-kejaksaan-text-muted max-w-2xl mx-auto">
                    "Sistem Informasi Manajemen Pengelolaan Barang Milik Negara"
                </p>
                <p class="text-lg text-kejaksaan-text-muted mt-2">
                    "Kejaksaan Agung Republik Indonesia"
                </p>
            </div>

            // System Categories
            {grouped_systems.into_iter().map(|(category, systems)| {
                view! {
                    <div class="bg-white rounded-lg shadow-sm p-6">
                        <h2 class="text-2xl font-bold text-kejaksaan-text mb-6 border-b pb-2">
                            "Sistem " {category}
                        </h2>
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {systems.into_iter().map(|system| {
                                view! {
                                    <KejCard class="hover:shadow-lg transition-shadow duration-300">
                                        <div class="text-center p-4">
                                            <div class="text-4xl mb-3">{system.icon}</div>
                                            <h3 class="text-lg font-semibold text-kejaksaan-text mb-2">
                                                {system.name}
                                            </h3>
                                            <p class="text-sm text-kejaksaan-text-muted mb-4 line-clamp-3">
                                                {system.description}
                                            </p>
                                            <div class=format!("text-sm font-medium mb-4 {}", system.status.css_class())>
                                                "Status: " {system.status.label()}
                                            </div>
                                            <KejButton
                                                variant=ButtonVariant::Primary
                                                class="w-full">
                                                "Akses Sistem"
                                            </KejButton>
                                        </div>
                                    </KejCard>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            }).collect_view()}

            // Quick Stats
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <KejCard>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-kejaksaan-primary mb-2">"9"</div>
                        <div class="text-kejaksaan-text-muted">"Sistem Terintegrasi"</div>
                    </div>
                </KejCard>
                <KejCard>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-green-600 mb-2">"8"</div>
                        <div class="text-kejaksaan-text-muted">"Sistem Online"</div>
                    </div>
                </KejCard>
                <KejCard>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-yellow-600 mb-2">"1"</div>
                        <div class="text-kejaksaan-text-muted">"Maintenance"</div>
                    </div>
                </KejCard>
            </div>

            // Support Information
            <KejCard class="bg-kejaksaan-bg-light">
                <div class="p-6">
                    <h2 class="text-xl font-semibold text-kejaksaan-text mb-4">
                        "📞 Dukungan Teknis"
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-kejaksaan-text-muted">
                        <div>
                            <p>"📧 Email: support@kejaksaan.go.id"</p>
                            <p>"📞 Telpon: (021) 7805000"</p>
                        </div>
                        <div>
                            <p>"🕒 Senin - Jumat: 08:00 - 17:00 WIB"</p>
                            <p>"📍 Jakarta, Indonesia"</p>
                        </div>
                    </div>
                </div>
            </KejCard>
        </div>
    }
}
