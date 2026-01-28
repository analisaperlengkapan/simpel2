//! # SIMPelv2 Datun - Criminal Prosecution System
//!
//! Sistem Informasi Direktorat Tindak Pidana Umum Kejaksaan RI dengan:
//! - **Case Management**: Manajemen perkara tindak pidana umum
//! - **Investigation Tracking**: Pelacakan proses penyidikan dan penuntutan
//! - **Evidence Management**: Manajemen barang bukti dan dokumentasi
//! - **Legal Analysis**: Analisis hukum dan strategi penuntutan
//! - **Government Compliance**: Sesuai standar sistem peradilan pidana

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};
use serde::{Deserialize, Serialize};

// Import shared components - Modern Leptos 0.7.8
use lib_ui::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use lib_ui::prelude::*;

// Import local components
use crate::components::DatunFooter;

/// Model data untuk perkara tindak pidana umum
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CriminalCase {
    /// ID unik perkara
    pub id: String,
    /// Nomor perkara
    pub case_number: String,
    /// Judul/nama perkara
    pub title: String,
    /// Deskripsi singkat
    pub description: String,
    /// Kategori tindak pidana
    pub category: CrimeCategory,
    /// Status perkara
    pub status: CaseStatus,
    /// Jaksa penuntut
    pub prosecutor: String,
    /// Tanggal mulai penanganan
    pub start_date: String,
    /// Target selesai
    pub target_date: Option<String>,
    /// Tersangka/terdakwa
    pub suspects: Vec<String>,
    /// Tingkat prioritas
    pub priority: PriorityLevel,
    /// Progress (0-100%)
    pub progress: u8,
}

/// Kategori tindak pidana umum
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CrimeCategory {
    /// Tindak pidana umum biasa
    General,
    /// Tindak pidana ekonomi
    Economic,
    /// Tindak pidana lingkungan
    Environmental,
    /// Tindak pidana narkotika
    Narcotics,
    /// Tindak pidana kekerasan
    Violence,
    /// Tindak pidana properti
    Property,
    /// Tindak pidana teknologi informasi
    Cybercrime,
}

impl CrimeCategory {
    /// Mendapatkan icon kategori
    pub fn icon(&self) -> &'static str {
        match self {
            CrimeCategory::General => "⚖️",
            CrimeCategory::Economic => "💰",
            CrimeCategory::Environmental => "🌱",
            CrimeCategory::Narcotics => "💊",
            CrimeCategory::Violence => "🚨",
            CrimeCategory::Property => "🏠",
            CrimeCategory::Cybercrime => "💻",
        }
    }

    /// Mendapatkan label kategori
    pub fn label(&self) -> &'static str {
        match self {
            CrimeCategory::General => "Tindak Pidana Umum",
            CrimeCategory::Economic => "Tindak Pidana Ekonomi",
            CrimeCategory::Environmental => "Tindak Pidana Lingkungan",
            CrimeCategory::Narcotics => "Tindak Pidana Narkotika",
            CrimeCategory::Violence => "Tindak Pidana Kekerasan",
            CrimeCategory::Property => "Tindak Pidana Properti",
            CrimeCategory::Cybercrime => "Tindak Pidana Siber",
        }
    }

    /// Mendapatkan CSS class warna
    pub fn color_class(&self) -> &'static str {
        match self {
            CrimeCategory::General => "text-blue-600 bg-blue-50",
            CrimeCategory::Economic => "text-green-600 bg-green-50",
            CrimeCategory::Environmental => "text-emerald-600 bg-emerald-50",
            CrimeCategory::Narcotics => "text-red-600 bg-red-50",
            CrimeCategory::Violence => "text-orange-600 bg-orange-50",
            CrimeCategory::Property => "text-purple-600 bg-purple-50",
            CrimeCategory::Cybercrime => "text-cyan-600 bg-cyan-50",
        }
    }
}

/// Status perkara
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CaseStatus {
    /// Penyidikan
    Investigation,
    /// Penuntutan
    Prosecution,
    /// Persidangan
    Trial,
    /// Putusan
    Verdict,
    /// Banding
    Appeal,
    /// Selesai
    Closed,
    /// Ditunda
    Suspended,
}

impl CaseStatus {
    /// Mendapatkan label status
    pub fn label(&self) -> &'static str {
        match self {
            CaseStatus::Investigation => "Penyidikan",
            CaseStatus::Prosecution => "Penuntutan",
            CaseStatus::Trial => "Persidangan",
            CaseStatus::Verdict => "Putusan",
            CaseStatus::Appeal => "Banding",
            CaseStatus::Closed => "Selesai",
            CaseStatus::Suspended => "Ditunda",
        }
    }

    /// Mendapatkan CSS class berdasarkan status
    pub fn css_class(&self) -> &'static str {
        match self {
            CaseStatus::Investigation => "text-yellow-600 bg-yellow-50",
            CaseStatus::Prosecution => "text-blue-600 bg-blue-50",
            CaseStatus::Trial => "text-purple-600 bg-purple-50",
            CaseStatus::Verdict => "text-green-600 bg-green-50",
            CaseStatus::Appeal => "text-orange-600 bg-orange-50",
            CaseStatus::Closed => "text-gray-600 bg-gray-50",
            CaseStatus::Suspended => "text-red-600 bg-red-50",
        }
    }
}

/// Tingkat prioritas perkara
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PriorityLevel {
    /// Prioritas rendah
    Low,
    /// Prioritas normal
    Normal,
    /// Prioritas tinggi
    High,
    /// Prioritas kritikal
    Critical,
}

impl PriorityLevel {
    /// Mendapatkan label prioritas
    pub fn label(&self) -> &'static str {
        match self {
            PriorityLevel::Low => "Rendah",
            PriorityLevel::Normal => "Normal",
            PriorityLevel::High => "Tinggi",
            PriorityLevel::Critical => "Kritikal",
        }
    }

    /// Mendapatkan CSS class berdasarkan prioritas
    pub fn css_class(&self) -> &'static str {
        match self {
            PriorityLevel::Low => "text-gray-600 bg-gray-50",
            PriorityLevel::Normal => "text-blue-600 bg-blue-50",
            PriorityLevel::High => "text-orange-600 bg-orange-50",
            PriorityLevel::Critical => "text-red-600 bg-red-50",
        }
    }
}

/// Komponen utama aplikasi Datun SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Html attr:lang="id"/>
        <Title text="Datun SIMPelv2 - Direktorat Tindak Pidana Umum Kejaksaan RI"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Sistem Informasi Direktorat Tindak Pidana Umum Kejaksaan Agung Republik Indonesia"/>
        <Meta name="keywords" content="datun, tindak pidana umum, kejaksaan, penuntutan, perkara"/>
        <Meta property="og:title" content="Datun SIMPelv2"/>
        <Meta property="og:description" content="Criminal Prosecution Excellence - Kejaksaan Agung RI"/>

        <Router>
            <Routes fallback=|| "Page not found".into_view()>
                <Route path=StaticSegment("") view=LoginRedirectPage />
                <Route path=StaticSegment("dashboard") view=DashboardPage />
            </Routes>
        </Router>
    }
}

/// Dashboard page with authentication
#[component]
fn DashboardPage() -> impl IntoView {
    let criminal_cases = create_case_data();

    let content = move || {
        view! {
            <div class="min-h-screen bg-kejaksaan-bg">
                <DatunHeaderWithAuth />
                <main class="container mx-auto px-4 py-8">
                    <DatunDashboard cases=criminal_cases.clone()/>
                </main>
                <DatunFooter />
            </div>
        }
    };

    view! {
        <ProtectedRoute children=content />
    }
}

/// Header with auth controls
#[component]
fn DatunHeaderWithAuth() -> impl IntoView {
    view! {
        <header class="bg-kejaksaan-primary text-white shadow-lg">
            <div class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <a href="/dashboard" class="flex items-center space-x-3">
                            <div class="w-12 h-12 bg-white rounded-lg flex items-center justify-center">
                                <span class="text-2xl">"⚖️"</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold">"Datun SIMPelv2"</h1>
                                <p class="text-sm text-kejaksaan-primary-light">"Tindak Pidana Umum"</p>
                            </div>
                        </a>
                    </div>
                    <div class="flex items-center space-x-4">
                        <UserProfile class="text-white".to_string() />
                        <LogoutButton class="text-white hover:bg-kejaksaan-primary-dark".to_string() />
                    </div>
                </div>
            </div>
        </header>
    }
}

/// Generate data perkara tindak pidana umum
fn create_case_data() -> Vec<CriminalCase> {
    vec![
        CriminalCase {
            id: "CASE001".to_string(),
            case_number: "Reg. Perkara No. 456/Pid.Um/2025/PN.Jkt.Sel".to_string(),
            title: "Kasus Penipuan Skema Investasi Bodong".to_string(),
            description: "Penipuan berkedok investasi saham dengan keuntungan tidak wajar merugikan 847 nasabah".to_string(),
            category: CrimeCategory::Economic,
            status: CaseStatus::Prosecution,
            prosecutor: "Drs. Ahmad Wijaya, S.H., M.H.".to_string(),
            start_date: "2025-06-15".to_string(),
            target_date: Some("2025-10-15".to_string()),
            suspects: vec!["John Doe".to_string(), "Jane Smith".to_string()],
            priority: PriorityLevel::High,
            progress: 65,
        },
        CriminalCase {
            id: "CASE002".to_string(),
            case_number: "Reg. Perkara No. 123/Pid.Sus/2025/PN.Jkt.Pst".to_string(),
            title: "Kasus Perdagangan Narkoba Jaringan Internasional".to_string(),
            description: "Jaringan perdagangan narkoba lintas negara dengan barang bukti 50kg shabu-shabu".to_string(),
            category: CrimeCategory::Narcotics,
            status: CaseStatus::Trial,
            prosecutor: "Dr. Siti Nurhaliza, S.H., M.H.".to_string(),
            start_date: "2025-05-20".to_string(),
            target_date: Some("2025-12-20".to_string()),
            suspects: vec!["Robert Johnson".to_string(), "Maria Garcia".to_string(), "David Lee".to_string()],
            priority: PriorityLevel::Critical,
            progress: 80,
        },
        CriminalCase {
            id: "CASE003".to_string(),
            case_number: "Reg. Perkara No. 789/Pid.Sus/2025/PN.Jkt.Utara".to_string(),
            title: "Kasus Pencemaran Lingkungan Industri Kimia".to_string(),
            description: "Pencemaran sungai oleh limbah industri kimia tanpa izin yang mencemari air bersih warga".to_string(),
            category: CrimeCategory::Environmental,
            status: CaseStatus::Investigation,
            prosecutor: "Ir. Budi Santoso, S.H., M.H.".to_string(),
            start_date: "2025-07-10".to_string(),
            target_date: Some("2025-11-10".to_string()),
            suspects: vec!["PT. Chemical Industries".to_string(), "Direktur Produksi".to_string()],
            priority: PriorityLevel::High,
            progress: 35,
        },
        CriminalCase {
            id: "CASE004".to_string(),
            case_number: "Reg. Perkara No. 234/Pid.Sus/2025/PN.Tangerang".to_string(),
            title: "Kasus Cyber Crime Pembobolan Rekening Bank".to_string(),
            description: "Pembobolan sistem perbankan digital menggunakan malware dengan kerugian Rp 15 miliar".to_string(),
            category: CrimeCategory::Cybercrime,
            status: CaseStatus::Prosecution,
            prosecutor: "Dr. Maria Indah, S.H., M.H.".to_string(),
            start_date: "2025-07-25".to_string(),
            target_date: Some("2025-12-25".to_string()),
            suspects: vec!["Hacker Group Alpha".to_string(), "System Administrator".to_string()],
            priority: PriorityLevel::Critical,
            progress: 55,
        },
        CriminalCase {
            id: "CASE005".to_string(),
            case_number: "Reg. Perkara No. 567/Pid.Um/2025/PN.Bekasi".to_string(),
            title: "Kasus Kekerasan Dalam Rumah Tangga dengan Pemberatan".to_string(),
            description: "KDRT berulang dengan kekerasan fisik dan psikis terhadap istri dan anak di bawah umur".to_string(),
            category: CrimeCategory::Violence,
            status: CaseStatus::Verdict,
            prosecutor: "Drs. Bambang Susilo, S.H., M.H.".to_string(),
            start_date: "2025-04-15".to_string(),
            target_date: Some("2025-08-15".to_string()),
            suspects: vec!["Pelaku KDRT".to_string()],
            priority: PriorityLevel::High,
            progress: 95,
        },
    ]
}

/// Dashboard utama Datun
#[component]
pub fn DatunDashboard(
    /// Daftar perkara tindak pidana
    cases: Vec<CriminalCase>,
) -> impl IntoView {
    // Kelompokkan perkara berdasarkan status
    let grouped_cases = cases.into_iter().fold(
        std::collections::HashMap::<String, Vec<CriminalCase>>::new(),
        |mut acc, case| {
            acc.entry(case.status.label().to_string())
                .or_default()
                .push(case);
            acc
        },
    );

    view! {
        <div class="space-y-8">
            // Hero Section
            <div class="text-center py-12 bg-white rounded-lg shadow-sm">
                <div class="text-6xl mb-4">"⚖️"</div>
                <h1 class="text-4xl font-bold text-kejaksaan-text mb-4">
                    "Datun SIMPelv2"
                </h1>
                <p class="text-xl text-kejaksaan-text-muted max-w-2xl mx-auto">
                    "Direktorat Tindak Pidana Umum"
                </p>
                <p class="text-lg text-kejaksaan-text-muted mt-2">
                    "Kejaksaan Agung Republik Indonesia"
                </p>
            </div>

            // Quick Stats
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-kejaksaan-primary mb-2">"127"</div>
                        <div class="text-kejaksaan-text-muted">"Perkara Aktif"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-green-600 mb-2">"89"</div>
                        <div class="text-kejaksaan-text-muted">"Selesai Bulan Ini"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-orange-600 mb-2">"23"</div>
                        <div class="text-kejaksaan-text-muted">"Prioritas Tinggi"</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-4">
                        <div class="text-3xl font-bold text-blue-600 mb-2">"94%"</div>
                        <div class="text-kejaksaan-text-muted">"Tingkat Penyelesaian"</div>
                    </div>
                </Card>
            </div>

            // Case Categories by Status
            {grouped_cases.into_iter().map(|(status, cases)| {
                view! {
                    <div class="bg-white rounded-lg shadow-sm p-6">
                        <h2 class="text-2xl font-bold text-kejaksaan-text mb-6 border-b pb-2">
                            "Perkara " {status}
                        </h2>
                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                            {cases.into_iter().map(|case| {
                                view! {
                                    <Card class="hover:shadow-lg transition-shadow duration-300">
                                        <div class="p-6">
                                            <div class="flex items-start justify-between mb-4">
                                                <div class="flex items-center space-x-3">
                                                    <div class="text-2xl">{case.category.icon()}</div>
                                                    <div class="flex-1">
                                                        <h3 class="text-lg font-semibold text-kejaksaan-text mb-1">
                                                            {case.title}
                                                        </h3>
                                                        <div class=format!("text-xs px-2 py-1 rounded-full inline-block {}", case.category.color_class())>
                                                            {case.category.label()}
                                                        </div>
                                                    </div>
                                                </div>
                                                <div class=format!("text-xs px-2 py-1 rounded-full {}", case.priority.css_class())>
                                                    {case.priority.label()}
                                                </div>
                                            </div>

                                            <p class="text-sm text-kejaksaan-text-muted mb-3 line-clamp-2">
                                                {case.description}
                                            </p>

                                            <div class="text-xs text-kejaksaan-text-muted mb-3">
                                                <strong>"No. Perkara: "</strong> {case.case_number}
                                            </div>

                                            <div class="grid grid-cols-2 gap-4 text-sm text-kejaksaan-text-muted mb-4">
                                                <div>
                                                    <span class="font-medium">"Jaksa: "</span>
                                                    <div class="text-xs">{case.prosecutor}</div>
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Tersangka: "</span>
                                                    <div class="text-xs">{case.suspects.len()} " orang"</div>
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Mulai: "</span>
                                                    {case.start_date}
                                                </div>
                                                <div>
                                                    <span class="font-medium">"Target: "</span>
                                                    {case.target_date.unwrap_or_else(|| "TBD".to_string())}
                                                </div>
                                            </div>

                                            // Progress Bar
                                            <div class="mb-4">
                                                <div class="flex justify-between text-sm text-kejaksaan-text-muted mb-1">
                                                    <span>"Progress"</span>
                                                    <span>{case.progress}"%"</span>
                                                </div>
                                                <div class="w-full bg-gray-200 rounded-full h-2">
                                                    <div
                                                        class="bg-kejaksaan-primary h-2 rounded-full transition-all duration-300"
                                                        style=format!("width: {}%", case.progress)>
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="flex space-x-2">
                                                <Button
                                                    variant=ButtonVariant::Primary
                                                    class="flex-1">
                                                    "Detail Perkara"
                                                </Button>
                                                <Button
                                                    variant=ButtonVariant::Secondary
                                                    class="flex-1">
                                                    "Timeline"
                                                </Button>
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
                        "📞 Bantuan Datun"
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-kejaksaan-text-muted">
                        <div>
                            <p>"📧 Email: datun@kejaksaan.go.id"</p>
                            <p>"📞 Telpon: (021) 7805001 ext. 234"</p>
                        </div>
                        <div>
                            <p>"🕒 Senin - Jumat: 08:00 - 16:00 WIB"</p>
                            <p>"📍 Gedung Datun, Jl. Sultan Hasanudin"</p>
                        </div>
                    </div>
                </div>
            </Card>
        </div>
    }
}
