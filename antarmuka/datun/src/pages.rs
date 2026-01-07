//! # SIMPelv2 Datun - Perdata dan Tata Usaha Negara System
//!
//! Sistem Informasi Bidang Perdata dan Tata Usaha Negara Kejaksaan RI:
//! - **Bantuan Hukum**: Litigasi dan Non-Litigasi (SKK)
//! - **Pertimbangan Hukum**: Legal Opinion (LO), Legal Assistance (LA), Legal Audit
//! - **Penegakan Hukum**: Pemulihan Keuangan Negara
//! - **Tindakan Hukum Lain**: Mediator/Fasilitator
//! - **Pelayanan Hukum**: Konsultasi Publik

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

// Import shared components
use shared_microfrontend::components::auth::{
    LoginRedirectPage, LogoutButton, ProtectedRoute, UserProfile,
};
use shared_microfrontend::prelude::*;

// Import local types and api
use crate::types::*;
use crate::api::fetch_cases;
use crate::components::DatunFooter;

/// Komponen utama aplikasi Datun SIMPelv2
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Html attr:lang="id"/>
        <Title text="Datun SIMPelv2 - Bidang Perdata dan Tata Usaha Negara"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="Sistem Informasi Bidang Perdata dan Tata Usaha Negara Kejaksaan RI"/>
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
    // Fetch cases from API using local resource (CSR)
    let cases_resource = LocalResource::new(|| fetch_cases());

    let content = move || {
        view! {
            <div class="min-h-screen bg-kejaksaan-bg">
                <DatunHeaderWithAuth />
                <main class="container mx-auto px-4 py-8">
                    <Suspense fallback=move || view! { <div class="text-center p-8">"Memuat data perkara..."</div> }>
                        {move || {
                            cases_resource.get().map(|result| match result {
                                Ok(cases) => view! { <DatunDashboard cases=cases/> }.into_any(),
                                Err(e) => view! {
                                    <div class="text-red-500 text-center p-8">
                                        "Gagal memuat data: " {e}
                                    </div>
                                }.into_any()
                            })
                        }}
                    </Suspense>
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
                                <p class="text-sm text-kejaksaan-primary-light">"Perdata & Tata Usaha Negara"</p>
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

/// Dashboard utama Datun
#[component]
pub fn DatunDashboard(
    /// Daftar perkara
    cases: Vec<DatunCase>,
) -> impl IntoView {
    // Stats calculation
    let total_cases = cases.len();
    let total_pemulihan: f64 = cases.iter().filter_map(|c| c.nilai_pemulihan).sum();
    let on_process = cases.iter().filter(|c| c.status == CaseStatus::Proses).count();

    // Group by Service Type
    let grouped_cases = cases.clone().into_iter().fold(
        std::collections::HashMap::<String, Vec<DatunCase>>::new(),
        |mut acc, case| {
            acc.entry(case.jenis_layanan.label().to_string())
                .or_default()
                .push(case);
            acc
        },
    );

    view! {
        <div class="space-y-8">
            // Hero Section
            <div class="text-center py-8 bg-white rounded-lg shadow-sm border-b-4 border-kejaksaan-primary relative">
                <div class="absolute top-4 right-4">
                    <Button
                        variant=ButtonVariant::Secondary
                        on:click=move |_| { window().location().reload().unwrap(); }
                        class="text-sm">
                        "🔄 Refresh Data"
                    </Button>
                </div>
                <div class="text-5xl mb-2">"⚖️"</div>
                <h1 class="text-3xl font-bold text-kejaksaan-text mb-2">
                    "Datun Center"
                </h1>
                <p class="text-lg text-kejaksaan-text-muted">
                    "Portal Layanan Hukum Perdata dan Tata Usaha Negara"
                </p>
            </div>

            // Quick Stats
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <Card>
                    <div class="text-center p-6">
                        <div class="text-sm text-kejaksaan-text-muted uppercase tracking-wider mb-1">"Total SKK / Permohonan"</div>
                        <div class="text-4xl font-bold text-kejaksaan-primary">{total_cases}</div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-6">
                        <div class="text-sm text-kejaksaan-text-muted uppercase tracking-wider mb-1">"Potensi Pemulihan (Rp)"</div>
                        <div class="text-4xl font-bold text-green-600">
                             {format!("{:.2} M", total_pemulihan / 1_000_000_000.0)}
                        </div>
                    </div>
                </Card>
                <Card>
                    <div class="text-center p-6">
                        <div class="text-sm text-kejaksaan-text-muted uppercase tracking-wider mb-1">"Sedang Berjalan"</div>
                        <div class="text-4xl font-bold text-blue-600">{on_process}</div>
                    </div>
                </Card>
            </div>

            // Main Content Area
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                // Left Column: List of Cases grouped by Service
                <div class="lg:col-span-2 space-y-6">
                    {grouped_cases.into_iter().map(|(service, cases)| {
                        view! {
                            <div class="bg-white rounded-lg shadow-sm overflow-hidden">
                                <div class="px-6 py-4 bg-gray-50 border-b flex justify-between items-center">
                                    <h2 class="text-lg font-bold text-gray-800">{service}</h2>
                                    <span class="bg-gray-200 text-gray-600 px-2 py-1 rounded text-xs">
                                        {cases.len()} " Perkara"
                                    </span>
                                </div>
                                <div class="divide-y divide-gray-100">
                                    {cases.into_iter().map(|case| {
                                        view! {
                                            <div class="p-6 hover:bg-blue-50 transition-colors duration-200">
                                                <div class="flex justify-between items-start mb-2">
                                                    <div class="flex-1">
                                                        <h3 class="font-semibold text-kejaksaan-primary mb-1">
                                                            {case.judul_perkara}
                                                        </h3>
                                                        <p class="text-sm text-gray-600 mb-1">
                                                            <span class="font-medium">"Pemohon: "</span> {case.instansi_pemohon}
                                                        </p>
                                                        <p class="text-xs text-gray-500">
                                                            "Lawan: " {case.pihak_lawan}
                                                        </p>
                                                    </div>
                                                    <span class=format!("px-3 py-1 rounded-full text-xs font-medium {}", case.status.css_class())>
                                                        {case.status.label()}
                                                    </span>
                                                </div>
                                                <div class="mt-3 flex items-center justify-between text-xs text-gray-500">
                                                    <span>"SKK: " {case.no_skk}</span>
                                                    <span>{case.updated_at}</span>
                                                </div>
                                                {if let Some(nilai) = case.nilai_pemulihan {
                                                    view! {
                                                        <div class="mt-2 text-sm font-medium text-green-600">
                                                            "Potensi: Rp " {format!("{:.0}", nilai)}
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div class="hidden"></div> }.into_any()
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>

                // Right Column: Actions & Info
                <div class="space-y-6">
                     <Card class="bg-gradient-to-br from-kejaksaan-primary to-kejaksaan-primary-dark text-white">
                        <div class="p-6">
                            <h3 class="text-xl font-bold mb-4">"Layanan Baru"</h3>
                            <div class="space-y-3">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    class="w-full justify-start bg-white/10 hover:bg-white/20 border-none text-white text-left">
                                    "➕ Input SKK Baru"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Secondary
                                    class="w-full justify-start bg-white/10 hover:bg-white/20 border-none text-white text-left">
                                    "📝 Permohonan LO / LA"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Secondary
                                    class="w-full justify-start bg-white/10 hover:bg-white/20 border-none text-white text-left">
                                    "📊 Laporan Bulanan"
                                </Button>
                            </div>
                        </div>
                    </Card>

                    <Card>
                        <div class="p-6">
                            <h3 class="font-bold text-gray-800 mb-4 border-b pb-2">"Jadwal Sidang / Agenda"</h3>
                            <div class="space-y-4">
                                <div class="flex items-start space-x-3 text-sm">
                                    <div class="bg-red-100 text-red-600 px-2 py-1 rounded text-xs font-bold">
                                        "10 Feb"
                                    </div>
                                    <div>
                                        <div class="font-medium">"Sidang PN Surabaya"</div>
                                        <div class="text-gray-500 text-xs">"PT. PLN vs PT. Energi"</div>
                                    </div>
                                </div>
                                <div class="flex items-start space-x-3 text-sm">
                                    <div class="bg-blue-100 text-blue-600 px-2 py-1 rounded text-xs font-bold">
                                        "12 Feb"
                                    </div>
                                    <div>
                                        <div class="font-medium">"Rapat Ekspose LO"</div>
                                        <div class="text-gray-500 text-xs">"Pemkot Surabaya - Aset"</div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </Card>
                </div>
            </div>
        </div>
    }
}
