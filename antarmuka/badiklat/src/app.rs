//! # SIMPelv2 Badiklat - Training & Education System
//!
//! Sistem Informasi Pendidikan dan Pelatihan Kejaksaan RI dengan:
//! - **Training Management**: Manajemen program pelatihan dan kurikulum
//! - **Competency Tracking**: Pelacakan kompetensi dan sertifikasi
//! - **Performance Analytics**: Analisis kinerja dan evaluasi pelatihan
//! - **Digital Learning**: Platform pembelajaran digital terintegrasi
//! - **Government Compliance**: Sesuai standar pendidikan aparatur

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use serde::{Deserialize, Serialize};

// Import shared components - Modern Leptos 0.7.8
use lib_ui::components::auth::{LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile};
use lib_ui::prelude::*;

// Import local components
use crate::components::BadiklatFooter;

/// Model data untuk program pelatihan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingProgram {
    /// ID unik program
    pub id: String,
    /// Nama program pelatihan
    pub title: String,
    /// Deskripsi program
    pub description: String,
    /// Kategori pelatihan
    pub category: TrainingCategory,
    /// Durasi pelatihan (dalam hari)
    pub duration_days: u8,
    /// Kapasitas peserta
    pub capacity: u16,
    /// Jumlah peserta terdaftar
    pub enrolled: u16,
    /// Status program
    pub status: ProgramStatus,
    /// Instruktur utama
    pub instructor: String,
    /// Tanggal mulai
    pub start_date: String,
    /// Level/tingkat pelatihan
    pub level: TrainingLevel,
    /// Persyaratan khusus
    pub requirements: Vec<String>,
}

/// Kategori program pelatihan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrainingCategory {
    /// Pelatihan kepemimpinan
    Leadership,
    /// Pelatihan teknis/fungsional
    Technical,
    /// Pelatihan administrasi
    Administrative,
    /// Pelatihan hukum
    Legal,
    /// Pelatihan teknologi informasi
    Information,
    /// Pelatihan manajemen
    Management,
}

impl TrainingCategory {
    /// Mendapatkan icon kategori
    pub fn icon(&self) -> &'static str {
        match self {
            TrainingCategory::Leadership => "👑",
            TrainingCategory::Technical => "🔧",
            TrainingCategory::Administrative => "📋",
            TrainingCategory::Legal => "⚖️",
            TrainingCategory::Information => "💻",
            TrainingCategory::Management => "📊",
        }
    }

    /// Mendapatkan label kategori
    pub fn label(&self) -> &'static str {
        match self {
            TrainingCategory::Leadership => "Kepemimpinan",
            TrainingCategory::Technical => "Teknis",
            TrainingCategory::Administrative => "Administrasi",
            TrainingCategory::Legal => "Hukum",
            TrainingCategory::Information => "Teknologi Informasi",
            TrainingCategory::Management => "Manajemen",
        }
    }

    /// Mendapatkan CSS class warna
    pub fn color_class(&self) -> &'static str {
        match self {
            TrainingCategory::Leadership => "text-purple-600 bg-purple-50",
            TrainingCategory::Technical => "text-blue-600 bg-blue-50",
            TrainingCategory::Administrative => "text-green-600 bg-green-50",
            TrainingCategory::Legal => "text-red-600 bg-red-50",
            TrainingCategory::Information => "text-cyan-600 bg-cyan-50",
            TrainingCategory::Management => "text-orange-600 bg-orange-50",
        }
    }
}

/// Status program pelatihan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProgramStatus {
    /// Pendaftaran terbuka
    Open,
    /// Sedang berlangsung
    Ongoing,
    /// Sudah selesai
    Completed,
    /// Ditunda
    Postponed,
    /// Dibatalkan
    Cancelled,
}

impl ProgramStatus {
    /// Mendapatkan label status
    pub fn label(&self) -> &'static str {
        match self {
            ProgramStatus::Open => "Pendaftaran Terbuka",
            ProgramStatus::Ongoing => "Sedang Berlangsung",
            ProgramStatus::Completed => "Selesai",
            ProgramStatus::Postponed => "Ditunda",
            ProgramStatus::Cancelled => "Dibatalkan",
        }
    }

    /// Mendapatkan CSS class berdasarkan status
    pub fn css_class(&self) -> &'static str {
        match self {
            ProgramStatus::Open => "text-green-600 bg-green-50",
            ProgramStatus::Ongoing => "text-blue-600 bg-blue-50",
            ProgramStatus::Completed => "text-gray-600 bg-gray-50",
            ProgramStatus::Postponed => "text-yellow-600 bg-yellow-50",
            ProgramStatus::Cancelled => "text-red-600 bg-red-50",
        }
    }
}

/// Level pelatihan
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrainingLevel {
    /// Level dasar
    Basic,
    /// Level menengah
    Intermediate,
    /// Level lanjutan
    Advanced,
    /// Level ekspertis
    Expert,
}

impl TrainingLevel {
    /// Mendapatkan label level
    pub fn label(&self) -> &'static str {
        match self {
            TrainingLevel::Basic => "Dasar",
            TrainingLevel::Intermediate => "Menengah",
            TrainingLevel::Advanced => "Lanjutan",
            TrainingLevel::Expert => "Ekspertis",
        }
    }
}

/// Komponen utama aplikasi Badiklat SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Html attr:lang="id"/>
        <Title text="Badiklat SIMPelv2 - Pendidikan dan Pelatihan Kejaksaan RI"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Sistem Informasi Pendidikan dan Pelatihan Kejaksaan Agung Republik Indonesia"/>
        <Meta name="keywords" content="badiklat, pelatihan, pendidikan, kejaksaan, aparatur, kompetensi"/>
        <Meta property="og:title" content="Badiklat SIMPelv2"/>
        <Meta property="og:description" content="Training & Education Excellence - Kejaksaan Agung RI"/>

        <Router>
            <Routes fallback=|| "Page not found".into_view()>
                // Public route - Login redirect page
                <Route path=StaticSegment("") view=|| view! { <LoginRedirectPage /> } />

                // Protected routes - require authentication
                <Route path=StaticSegment("dashboard") view=DashboardPage />
            </Routes>
        </Router>
    }
}

/// Dashboard page with authentication protection
#[component]
fn DashboardPage() -> impl IntoView {
    let training_programs = create_training_data();

    let content = move || {
        view! {
            <div class="min-h-screen bg-kejaksaan-bg">
                <BadiklatHeaderWithAuth />

                <main class="container mx-auto px-4 py-8">
                    <BadiklatDashboard programs=training_programs.clone()/>
                </main>

                <BadiklatFooter />
            </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

/// Header with authentication controls
#[component]
fn BadiklatHeaderWithAuth() -> impl IntoView {
    let _auth = use_auth();

    view! {
        <header class="bg-kejaksaan-primary text-white shadow-lg">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between py-4">
                    // Logo dan Brand
                    <div class="flex items-center space-x-4">
                        <a href="/dashboard" class="flex items-center space-x-3 hover:opacity-80 transition-opacity">
                            <div class="w-12 h-12 bg-white rounded-lg flex items-center justify-center">
                                <span class="text-2xl">"🎓"</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold">"Badiklat SIMPelv2"</h1>
                                <p class="text-sm text-kejaksaan-primary-light">"Pendidikan & Pelatihan"</p>
                            </div>
                        </a>
                    </div>

                    // Navigation Menu
                    <nav class=":flex space-x-1">
                        <NavigationLink href="/dashboard" text="Dashboard" icon="🏠" />
                        <NavigationLink href="/programs" text="Program" icon="📚" />
                        <NavigationLink href="/schedule" text="Jadwal" icon="📅" />
                        <NavigationLink href="/participants" text="Peserta" icon="👥" />
                        <NavigationLink href="/instructors" text="Instruktur" icon="👨‍🏫" />
                        <NavigationLink href="/certificates" text="Sertifikat" icon="🏆" />
                        <NavigationLink href="/reports" text="Laporan" icon="📊" />
                    </nav>

                    // User Profile and Logout
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-kejaksaan-primary-dark".to_string() />
                    </div>

                    // Mobile Menu Button
                    <button class="md:hidden p-2 hover:bg-kejaksaan-primary-dark rounded-lg transition-colors">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </button>
                </div>

                // Training Status Bar
                <div class="border-t border-kejaksaan-primary-light py-2">
                    <div class="flex items-center justify-between text-sm">
                        <div class="flex items-center space-x-6">
                            <StatusItem icon="📈" label="Program Aktif" value="5" />
                            <StatusItem icon="👥" label="Peserta" value="148" />
                            <StatusItem icon="🎯" label="Completion Rate" value="95%" />
                        </div>
                        <div class="text-kejaksaan-primary-light">
                            "Last Updated: " <span class="text-white">"09:30 WIB"</span>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    }
}

/// Component untuk navigation link
#[component]
fn NavigationLink(
    /// URL tujuan
    href: &'static str,
    /// Teks link
    text: &'static str,
    /// Icon emoji
    icon: &'static str,
) -> impl IntoView {
    view! {
        <a href=href
           class="flex items-center space-x-2 px-3 py-2 rounded-lg hover:bg-kejaksaan-primary-dark transition-colors duration-200">
            <span>{icon}</span>
            <span class="text-sm font-medium">{text}</span>
        </a>
    }
}

/// Component untuk status item di bar
#[component]
fn StatusItem(
    /// Icon emoji
    icon: &'static str,
    /// Label item
    label: &'static str,
    /// Nilai yang ditampilkan
    value: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center space-x-2">
            <span>{icon}</span>
            <span class="text-kejaksaan-primary-light">{label}":"</span>
            <span class="text-white font-medium">{value}</span>
        </div>
    }
}

/// Generate data program pelatihan
fn create_training_data() -> Vec<TrainingProgram> {
    vec![
        TrainingProgram {
            id: "TRN001".to_string(),
            title: "Program Kepemimpinan Eksekutif Jaksa".to_string(),
            description: "Pelatihan kepemimpinan untuk jaksa tingkat eksekutif dalam manajemen strategis dan pengambilan keputusan".to_string(),
            category: TrainingCategory::Leadership,
            duration_days: 5,
            capacity: 30,
            enrolled: 25,
            status: ProgramStatus::Open,
            instructor: "Dr. Ahmad Wijaya, S.H., M.H.".to_string(),
            start_date: "2025-09-15".to_string(),
            level: TrainingLevel::Advanced,
            requirements: vec![
                "Jabatan minimal Eselon III".to_string(),
                "Pengalaman kerja minimal 10 tahun".to_string(),
            ],
        },
        TrainingProgram {
            id: "TRN002".to_string(),
            title: "Pelatihan Teknis Penuntutan Pidana Korupsi".to_string(),
            description: "Pelatihan khusus untuk penanganan kasus korupsi dengan teknik penuntutan yang efektif".to_string(),
            category: TrainingCategory::Legal,
            duration_days: 7,
            capacity: 25,
            enrolled: 23,
            status: ProgramStatus::Ongoing,
            instructor: "Prof. Dr. Siti Nurhaliza, S.H., M.H.".to_string(),
            start_date: "2025-08-20".to_string(),
            level: TrainingLevel::Expert,
            requirements: vec![
                "Jaksa Fungsional".to_string(),
                "Pengalaman menangani kasus korupsi".to_string(),
                "Sertifikat Penyidik Tindak Pidana Korupsi".to_string(),
            ],
        },
        TrainingProgram {
            id: "TRN003".to_string(),
            title: "Digitalisasi Administrasi Perkara".to_string(),
            description: "Pelatihan penggunaan sistem informasi digital untuk manajemen administrasi perkara yang efisien".to_string(),
            category: TrainingCategory::Information,
            duration_days: 3,
            capacity: 40,
            enrolled: 35,
            status: ProgramStatus::Open,
            instructor: "Ir. Budi Santoso, M.Kom.".to_string(),
            start_date: "2025-09-01".to_string(),
            level: TrainingLevel::Intermediate,
            requirements: vec![
                "Kemampuan dasar komputer".to_string(),
                "Staff administrasi perkara".to_string(),
            ],
        },
        TrainingProgram {
            id: "TRN004".to_string(),
            title: "Manajemen Keuangan Negara untuk Jaksa".to_string(),
            description: "Pelatihan pengelolaan keuangan negara dan pengawasan anggaran sektor publik".to_string(),
            category: TrainingCategory::Management,
            duration_days: 4,
            capacity: 35,
            enrolled: 20,
            status: ProgramStatus::Open,
            instructor: "Drs. Bambang Susilo, M.Si., Ak.".to_string(),
            start_date: "2025-09-10".to_string(),
            level: TrainingLevel::Intermediate,
            requirements: vec![
                "Dasar akuntansi".to_string(),
                "Pengalaman pengelolaan anggaran".to_string(),
            ],
        },
        TrainingProgram {
            id: "TRN005".to_string(),
            title: "Workshop Komunikasi Efektif di Persidangan".to_string(),
            description: "Pelatihan teknik komunikasi dan presentasi yang efektif dalam persidangan".to_string(),
            category: TrainingCategory::Technical,
            duration_days: 2,
            capacity: 50,
            enrolled: 45,
            status: ProgramStatus::Completed,
            instructor: "Dr. Maria Indah, S.Psi., M.M.".to_string(),
            start_date: "2025-08-01".to_string(),
            level: TrainingLevel::Basic,
            requirements: vec![
                "Semua level jabatan".to_string(),
            ],
        },
    ]
}

/// Dashboard utama Badiklat
#[component]
pub fn BadiklatDashboard(
    /// Daftar program pelatihan
    programs: Vec<TrainingProgram>,
) -> impl IntoView {
    // Kelompokkan program berdasarkan kategori
    let grouped_programs = programs.into_iter().fold(
        std::collections::HashMap::<String, Vec<TrainingProgram>>::new(),
        |mut acc, program| {
            acc.entry(program.category.label().to_string())
                .or_default()
                .push(program);
            acc
        },
    );

    view! {
        <div class="space-y-8">
            // Hero Section
            <div class="text-center py-12 bg-white rounded-lg shadow-sm">
                <div class="text-6xl mb-4">"🎓"</div>
                <h1 class="text-4xl font-bold text-kejaksaan-text mb-4">
                    "Badiklat SIMPelv2"
                </h1>
                <p class="text-xl text-kejaksaan-text-muted max-w-2xl mx-auto">
                    "Sistem Informasi Pendidikan dan Pelatihan"
                </p>
                <p class="text-lg text-kejaksaan-text-muted mt-2">
                    "Kejaksaan Agung Republik Indonesia"
                </p>
            </div>

            // Quick Stats
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-kejaksaan-primary mb-2">"5"</div>
                        <div class="text-kejaksaan-text-muted">"Program Aktif"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-green-600 mb-2">"148"</div>
                        <div class="text-kejaksaan-text-muted">"Peserta Terdaftar"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-blue-600 mb-2">"23"</div>
                        <div class="text-kejaksaan-text-muted">"Sedang Berlangsung"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-purple-600 mb-2">"12"</div>
                        <div class="text-kejaksaan-text-muted">"Instruktur Aktif"</div>
                    </div>
                </Card>
            </div>

            // Training Categories
            {grouped_programs.into_iter().map(|(category, programs)| {
                view! {
                    <div class="bg-white rounded-lg shadow-sm p-6">
                        <h2 class="text-2xl font-bold text-kejaksaan-text mb-6 border-b pb-2">
                            "Program " {category}
                        </h2>
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            {programs.into_iter().map(|program| {
                                view! {
                                    <Card class="hover:shadow-lg transition-shadow duration-300">
                                        <div class="p-6">
                                            <div class="flex items-start justify-between mb-4">
                                                <div class="flex items-center space-x-3">
                                                    <div class="text-2xl">{program.category.icon()}</div>
                                                    <div>
                                                        <h3 class="text-lg font-semibold text-kejaksaan-text mb-1">
                                                            {program.title}
                                                        </h3>
                                                        <div class=format!("text-xs px-2 py-1 rounded-full inline-block {}", program.category.color_class())>
                                                            {program.category.label()}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class=format!("text-xs px-2 py-1 rounded-full {}", program.status.css_class())>
                                                    {program.status.label()}
                                                </div>
                                            </div>

                                            <p class="text-sm text-kejaksaan-text-muted mb-4 line-clamp-2">
                                                {program.description}
                                            </p>

                                            <div class="grid grid-cols-2 gap-4 text-sm text-kejaksaan-text-muted mb-4">
                                                <div>
                                                    <span class="font-medium">"Durasi: "</span>
                                                    {program.duration_days} " hari"
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Level: "</span>
                                                    {program.level.label()}
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Kapasitas: "</span>
                                                    {program.enrolled} "/" {program.capacity}
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Mulai: "</span>
                                                    {program.start_date}
                                                </div>
                                            </div>

                                            <div class="mb-4">
                                                <p class="text-sm font-medium text-kejaksaan-text mb-1">"Instruktur:"</p>
                                                <p class="text-sm text-kejaksaan-text-muted">{program.instructor}</p>
                                            </div>

                                            <div class="flex space-x-2">
                                                <Button
                                                    variant=ButtonVariant::Primary
                                                    class="flex-1">
                                                    "Lihat Detail"
                                                </Button>
                                                {if matches!(program.status, ProgramStatus::Open) {
                                                    view! {
                                                        <Button
                                                            variant=ButtonVariant::Secondary
                                                            class="flex-1">
                                                            "Daftar"
                                                        </Button>
                                                    }
                                                } else {
                                                    view! {
                                                        <Button
                                                            variant=ButtonVariant::Ghost
                                                            class="flex-1">
                                                            "Info"
                                                        </Button>
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </Card>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }
            }).collect_view()}

            // Support Information
            <Card class="bg-kejaksaan-bg-light">
                <div class="p-6">
                    <h2 class="text-xl font-semibold text-kejaksaan-text mb-4">
                        "📞 Bantuan Badiklat"
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-kejaksaan-text-muted">
                        <div>
                            <p>"📧 Email: badiklat@kejaksaan.go.id"</p>
                            <p>"📞 Telpon: (021) 7805001"</p>
                        </div>
                        <div>
                            <p>"🕒 Senin - Jumat: 08:00 - 16:00 WIB"</p>
                            <p>"📍 Gedung Badiklat, Jl. Sultan Hasanudin"</p>
                        </div>
                    </div>
                </div>
            </Card>
        </div>
    }
}
