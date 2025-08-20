//! # SIMPelv2 Portal Utama - Modern Government Portal
//!
//! Portal utama yang mengintegrasikan semua layanan SIMPelv2 dengan:
//! - **Modern UI/UX**: Design system berbasis Tailwind CSS
//! - **Performance Optimized**: Code splitting dan lazy loading
//! - **Accessibility**: WCAG 2.1 AA compliance
//! - **Government Branding**: Konsisten dengan identitas Kejaksaan RI

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Router, Routes, Route};
use serde::{Deserialize, Serialize};

// Import shared components - Modern Leptos 0.7.8
use shared_microfrontend::prelude::*;

use crate::components::{Header, Footer};

/// Model data untuk sistem yang terintegrasi
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub name: &'static str,
    pub url: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub status: SystemStatus,
    pub priority: u8,
}

/// Status sistem dalam portal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemStatus {
    Active,
    Maintenance,
    Beta,
    Coming,
}

impl SystemStatus {
    pub fn badge_class(&self) -> &'static str {
        match self {
            SystemStatus::Active => "bg-green-100 text-green-800",
            SystemStatus::Maintenance => "bg-yellow-100 text-yellow-800",
            SystemStatus::Beta => "bg-blue-100 text-blue-800",
            SystemStatus::Coming => "bg-gray-100 text-gray-800",
        }
    }
    
    pub fn text(&self) -> &'static str {
        match self {
            SystemStatus::Active => "Aktif",
            SystemStatus::Maintenance => "Maintenance",
            SystemStatus::Beta => "Beta",
            SystemStatus::Coming => "Segera",
        }
    }
}

/// Komponen utama Portal SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    // Provide context untuk meta tags
    provide_meta_context();
    
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
        
        <Router>
            <div class="min-h-screen bg-kejaksaan-bg">
                <Header />
                
                <main class="container mx-auto px-4 py-8">
                    <Routes fallback=move || view! { <NotFoundPage/> }>
                        <Route path="/" view=move || view! { <HomePage systems=systems.clone()/> }/>
                        <Route path="/about" view=AboutPage/>
                        <Route path="/help" view=HelpPage/>
                    </Routes>
                </main>
                
                <Footer />
            </div>
        </Router>
    }
}

/// Generate data sistem yang tersedia
fn create_system_data() -> Vec<SystemInfo> {
    vec![
        SystemInfo {
            name: "Pidana Umum",
            url: "/pidum",
            icon: "⚖️",
            description: "Sistem Penanganan Perkara Pidana Umum",
            status: SystemStatus::Active,
            priority: 1,
        },
        SystemInfo {
            name: "Pidana Khusus", 
            url: "/pidsus",
            icon: "🔍",
            description: "Sistem Penanganan Perkara Pidana Khusus",
            status: SystemStatus::Active,
            priority: 2,
        },
        SystemInfo {
            name: "Pidana Militer",
            url: "/pidmil", 
            icon: "🎖️",
            description: "Sistem Penanganan Perkara Pidana Militer",
            status: SystemStatus::Active,
            priority: 3,
        },
        SystemInfo {
            name: "Perdata & TUN",
            url: "/datun",
            icon: "📋",
            description: "Sistem Perdata dan Tata Usaha Negara",
            status: SystemStatus::Active,
            priority: 4,
        },
        SystemInfo {
            name: "Intelijen",
            url: "/intel",
            icon: "🕵️",
            description: "Sistem Intelijen Kejaksaan",
            status: SystemStatus::Active,
            priority: 5,
        },
        SystemInfo {
            name: "Pengawasan",
            url: "/pengawasan",
            icon: "👁️",
            description: "Sistem Pengawasan Internal",
            status: SystemStatus::Active,
            priority: 6,
        },
        SystemInfo {
            name: "Badiklat",
            url: "/badiklat",
            icon: "🎓",
            description: "Sistem Pendidikan dan Pelatihan",
            status: SystemStatus::Active,
            priority: 7,
        },
        SystemInfo {
            name: "Pembinaan",
            url: "/pembinaan",
            icon: "👥",
            description: "Sistem Pembinaan SDM",
            status: SystemStatus::Beta,
            priority: 8,
        },
        SystemInfo {
            name: "Pemulihan Aset",
            url: "/pemulihan-aset",
            icon: "💰",
            description: "Sistem Pemulihan Aset Negara",
            status: SystemStatus::Beta,
            priority: 9,
        },
    ]
}

/// Halaman utama portal
#[component]
pub fn HomePage(systems: Vec<SystemInfo>) -> impl IntoView {
    view! {
        <div class="space-y-8">
            // Hero Section
            <div class="text-center space-y-4">
                <h1 class="text-4xl font-bold text-kejaksaan-text mb-4">
                    "Portal SIMPelv2"
                </h1>
                <p class="text-lg text-kejaksaan-text-muted max-w-2xl mx-auto">
                    "Sistem Informasi Manajemen Pengelolaan Barang Milik Negara"
                    <br/>
                    "Kejaksaan Agung Republik Indonesia"
                </p>
                <div class="flex justify-center space-x-4 mt-6">
                    <KejButton variant=ButtonVariant::Primary>
                        "Panduan Pengguna"
                    </KejButton>
                    <KejButton variant=ButtonVariant::Secondary>
                        "Kontak Support"
                    </KejButton>
                </div>
            </div>

            // Systems Grid
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {systems.into_iter().map(|system| {
                    view! { <SystemCard system=system /> }
                }).collect::<Vec<_>>()}
            </div>

            // Quick Stats
            <QuickStats />
        </div>
    }
}

/// Komponen card untuk setiap sistem
#[component]
pub fn SystemCard(system: SystemInfo) -> impl IntoView {
    view! {
        <KejCard>
            <div class="p-6 space-y-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-3">
                        <span class="text-2xl">{system.icon}</span>
                        <h3 class="text-lg font-semibold text-kejaksaan-text">
                            {system.name}
                        </h3>
                    </div>
                    <span class=format!("px-2 py-1 text-xs font-medium rounded-full {}", system.status.badge_class())>
                        {system.status.text()}
                    </span>
                </div>
                
                <p class="text-sm text-kejaksaan-text-muted">
                    {system.description}
                </p>
                
                <div class="flex space-x-2">
                    <KejButton variant=ButtonVariant::Primary class="flex-1">
                        "Buka Sistem"
                    </KejButton>
                    <KejButton variant=ButtonVariant::Secondary>
                        "Info"
                    </KejButton>
                </div>
            </div>
        </KejCard>
    }
}

/// Quick Stats untuk dashboard
#[component]
pub fn QuickStats() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
            <KejCard>
                <div class="p-4 text-center">
                    <div class="text-2xl font-bold text-kejaksaan-primary">9</div>
                    <div class="text-sm text-kejaksaan-text-muted">"Sistem Aktif"</div>
                </div>
            </KejCard>
            <KejCard>
                <div class="p-4 text-center">
                    <div class="text-2xl font-bold text-green-600">24/7</div>
                    <div class="text-sm text-kejaksaan-text-muted">"Availability"</div>
                </div>
            </KejCard>
            <KejCard>
                <div class="p-4 text-center">
                    <div class="text-2xl font-bold text-blue-600">1000+</div>
                    <div class="text-sm text-kejaksaan-text-muted">"Pengguna"</div>
                </div>
            </KejCard>
            <KejCard>
                <div class="p-4 text-center">
                    <div class="text-2xl font-bold text-purple-600">v2.0</div>
                    <div class="text-sm text-kejaksaan-text-muted">"Versi Sistem"</div>
                </div>
            </KejCard>
        </div>
    }
}

/// Halaman About
#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto space-y-8">
            <h1 class="text-3xl font-bold text-kejaksaan-text">"Tentang SIMPelv2"</h1>
            
            <KejCard>
                <div class="p-6 space-y-4">
                    <h2 class="text-xl font-semibold text-kejaksaan-text">"Visi & Misi"</h2>
                    <p class="text-kejaksaan-text-muted">
                        "SIMPelv2 adalah sistem informasi terintegrasi untuk mendukung tugas-tugas 
                        Kejaksaan Agung Republik Indonesia dalam penegakan hukum dan keadilan."
                    </p>
                </div>
            </KejCard>
        </div>
    }
}

/// Halaman Help
#[component]
pub fn HelpPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto space-y-8">
            <h1 class="text-3xl font-bold text-kejaksaan-text">"Bantuan & Dukungan"</h1>
            
            <KejCard>
                <div class="p-6 space-y-4">
                    <h2 class="text-xl font-semibold text-kejaksaan-text">"Kontak Support"</h2>
                    <div class="space-y-2 text-kejaksaan-text-muted">
                        <p>"📧 Email: support@kejaksaan.go.id"</p>
                        <p>"📞 Telpon: (021) 7805000"</p>
                    </div>
                </div>
            </KejCard>
        </div>
    }
}

/// Halaman 404 - Not Found
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="text-center py-16">
            <KejCard class="max-w-md mx-auto">
                <div class="text-center">
                    <div class="text-6xl text-kejaksaan-primary mb-4">"404"</div>
                    <h1 class="text-2xl font-bold text-kejaksaan-text mb-4">
                        "Halaman Tidak Ditemukan"
                    </h1>
                    <p class="text-kejaksaan-text-muted mb-6">
                        "Halaman yang Anda cari tidak tersedia atau telah dipindahkan."
                    </p>
                    <KejButton variant=ButtonVariant::Primary 
                               onclick="() => window.history.back()">
                        "Kembali"
                    </KejButton>
                </div>
            </KejCard>
        </div>
    }
}
