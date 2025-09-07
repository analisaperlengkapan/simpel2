use crate::types::*;
use chrono::Utc;
use leptos::prelude::*;

/// PIDSUS Dashboard Page
#[component]
pub fn PidsusDashboard() -> impl IntoView {
    // Sample statistics data
    let (stats, _set_stats) = signal(PidsusStatistics {
        total_cases: 156,
        active_cases: 23,
        closed_cases: 133,
        total_suspects: 89,
        convicted_suspects: 67,
        conviction_rate: 85.2,
        total_evidence: 432,
        total_recovered_assets: 125_000_000_000.0,
        average_case_duration: 180,
        success_rate: 91.5,
        international_cases: 12,
    });

    // Sample recent cases
    let (recent_cases, _set_recent_cases) = signal(vec![
        SpecialCase {
            id: "1".to_string(),
            case_number: "PIDSUS-2024-001".to_string(),
            crime_type: SpecialCrimeType::MoneyLaundering,
            title: "Pencucian Uang Lintas Negara".to_string(),
            description:
                "Kasus pencucian uang senilai 50 miliar yang melibatkan jaringan internasional"
                    .to_string(),
            status: SpecialCaseStatus::Investigation,
            priority: SpecialCasePriority::High,
            classification: ClassificationLevel::Secret,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            lead_investigator: "Jaksa Senior A. Rahman".to_string(),
            team_members: vec!["Tim Khusus PIDSUS".to_string()],
            related_agencies: vec![RelatedAgency::PPATK, RelatedAgency::Polri],
            location: "Jakarta".to_string(),
            estimated_loss: Some(50_000_000_000.0),
            suspects_count: 5,
            evidence_count: 23,
            witnesses_count: 12,
        },
        SpecialCase {
            id: "2".to_string(),
            case_number: "PIDSUS-2024-002".to_string(),
            crime_type: SpecialCrimeType::HumanTrafficking,
            title: "Jaringan Perdagangan Manusia".to_string(),
            description: "Kasus perdagangan manusia dengan modus kerja ke luar negeri".to_string(),
            status: SpecialCaseStatus::Prosecution,
            priority: SpecialCasePriority::Urgent,
            classification: ClassificationLevel::Confidential,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            lead_investigator: "Jaksa Senior B. Sari".to_string(),
            team_members: vec!["Tim Lintas Batas".to_string()],
            related_agencies: vec![RelatedAgency::Polri, RelatedAgency::BNN],
            location: "Batam".to_string(),
            estimated_loss: None,
            suspects_count: 8,
            evidence_count: 31,
            witnesses_count: 45,
        },
    ]);

    view! {
        <div class="space-y-6">
            <SpecialHeader
                title="Dashboard PIDSUS".to_string()
                subtitle="Penyidikan Pidana Khusus - Kejaksaan Agung RI".to_string()
                icon_class="fa-shield-alt".to_string()
            />

            // Quick Statistics
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <SpecialStatCard
                    title="Total Kasus".to_string()
                    value=stats.get().total_cases.to_string()
                    icon="fa-folder".to_string()
                    color="blue".to_string()
                    trend="Semua periode".to_string()
                />
                <SpecialStatCard
                    title="Kasus Aktif".to_string()
                    value=stats.get().active_cases.to_string()
                    icon="fa-folder-open".to_string()
                    color="yellow".to_string()
                    trend="Dalam proses".to_string()
                />
                <SpecialStatCard
                    title="Tersangka".to_string()
                    value=stats.get().total_suspects.to_string()
                    icon="fa-user-secret".to_string()
                    color="red".to_string()
                    trend=format!("{} terpidana", stats.get().convicted_suspects)
                />
                <SpecialStatCard
                    title="Tingkat Konviksi".to_string()
                    value=format!("{:.1}%", stats.get().conviction_rate)
                    icon="fa-gavel".to_string()
                    color="green".to_string()
                    trend="Rata-rata tahunan".to_string()
                />
            </div>

            // Action Buttons
            <div class="flex flex-wrap gap-3">
                <button class="inline-flex items-center px-4 py-2 bg-red-600 hover:bg-red-700 text-white text-sm font-medium rounded-md shadow-sm transition-colors">
                    <i class="fas fa-plus mr-2"></i>
                    "Kasus Baru"
                </button>
                <button class="inline-flex items-center px-4 py-2 bg-yellow-600 hover:bg-yellow-700 text-white text-sm font-medium rounded-md shadow-sm transition-colors">
                    <i class="fas fa-bullseye mr-2"></i>
                    "Operasi Khusus"
                </button>
                <button class="inline-flex items-center px-4 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-md shadow-sm transition-colors">
                    <i class="fas fa-microscope mr-2"></i>
                    "Analisis Forensik"
                </button>
                <button class="inline-flex items-center px-4 py-2 bg-gray-600 hover:bg-gray-700 text-white text-sm font-medium rounded-md shadow-sm transition-colors">
                    <i class="fas fa-handshake mr-2"></i>
                    "Kerjasama Internasional"
                </button>
            </div>

            // Recent Cases Section
            <div class="bg-white rounded-lg shadow-lg p-6">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-bold text-gray-900">"Kasus Terbaru"</h2>
                    <SpecialSearchInput
                        placeholder="Cari kasus khusus...".to_string()
                    />
                </div>

                <div class="space-y-4">
                    {move || recent_cases.get().into_iter().map(|case| view! {
                        <div class="border border-gray-200 rounded-lg p-6 hover:shadow-md transition-shadow">
                            <div class="flex items-start justify-between">
                                <div class="flex-1">
                                    <div class="flex items-center gap-3 mb-3">
                                        <h3 class="font-semibold text-lg text-gray-900">{case.case_number.clone()}</h3>
                                        <SpecialCaseStatusBadge status=case.status.clone() />
                                        <SpecialPriorityBadge priority=case.priority.clone() />
                                        <ClassificationBadge classification=case.classification.clone() />
                                    </div>
                                    <p class="text-gray-700 font-medium mb-2">{case.title.clone()}</p>
                                    <p class="text-gray-600 text-sm mb-4">{case.description.clone()}</p>
                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-gray-500">
                                        <div>
                                            <span class="font-medium">"Penyidik Utama:"</span><br/>
                                            <span class="text-gray-700">{case.lead_investigator.clone()}</span>
                                        </div>
                                        <div>
                                            <span class="font-medium">"Lokasi:"</span><br/>
                                            <span class="text-gray-700">{case.location.clone()}</span>
                                        </div>
                                        <div>
                                            <span class="font-medium">"Tersangka:"</span><br/>
                                            <span class="text-gray-700">{case.suspects_count} " orang"</span>
                                        </div>
                                        <div>
                                            <span class="font-medium">"Bukti:"</span><br/>
                                            <span class="text-gray-700">{case.evidence_count} " item"</span>
                                        </div>
                                    </div>
                                    {case.estimated_loss.map(|loss| view! {
                                        <div class="mt-3 p-3 bg-red-50 border border-red-200 rounded">
                                            <span class="text-sm font-medium text-red-800">
                                                "Perkiraan Kerugian: Rp " {format!("{:.0}", loss / 1_000_000.0)} " juta"
                                            </span>
                                        </div>
                                    })}
                                </div>
                                <div class="ml-4">
                                    <button class="inline-flex items-center px-3 py-2 bg-gray-600 hover:bg-gray-700 text-white text-xs font-medium rounded-md shadow-sm transition-colors">
                                        <i class="fas fa-eye mr-1"></i>
                                        "Detail"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Dashboard Overview Grid
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                // Kasus Prioritas Tinggi
                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 flex items-center">
                        <i class="fas fa-exclamation-triangle text-red-500 mr-2"></i>
                        "Kasus Prioritas Tinggi"
                    </h3>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDSUS-2024-002"</span>
                            <span class="text-xs bg-red-100 text-red-800 px-2 py-1 rounded">"Mendesak"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDSUS-2024-001"</span>
                            <span class="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">"Tinggi"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDSUS-2024-005"</span>
                            <span class="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">"Tinggi"</span>
                        </div>
                    </div>
                </div>

                // Progress Operasi Khusus
                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 flex items-center">
                        <i class="fas fa-bullseye text-blue-500 mr-2"></i>
                        "Progress Operasi"
                    </h3>
                    <div class="space-y-4">
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Operasi Anti Pencucian"</span>
                                <span class="text-sm text-gray-600">"85%"</span>
                            </div>
                            <SpecialProgressBar percentage=85 color="green".to_string() />
                        </div>
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Operasi Cyber Crime"</span>
                                <span class="text-sm text-gray-600">"60%"</span>
                            </div>
                            <SpecialProgressBar percentage=60 color="yellow".to_string() />
                        </div>
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Operasi Lintas Batas"</span>
                                <span class="text-sm text-gray-600">"30%"</span>
                            </div>
                            <SpecialProgressBar percentage=30 color="red".to_string() />
                        </div>
                    </div>
                </div>

                // Kerjasama Internasional
                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4 flex items-center">
                        <i class="fas fa-globe text-green-500 mr-2"></i>
                        "Kerjasama Internasional"
                    </h3>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Singapura"</span>
                            <span class="font-semibold text-green-600">"Aktif"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Malaysia"</span>
                            <span class="font-semibold text-blue-600">"Pending"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Australia"</span>
                            <span class="font-semibold text-green-600">"Aktif"</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Swiss"</span>
                            <span class="font-semibold text-yellow-600">"Review"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Header khusus untuk PIDSUS dengan tema merah Kejaksaan
#[component]
pub fn SpecialHeader(title: String, subtitle: String, icon_class: String) -> impl IntoView {
    view! {
        <div class="bg-gradient-to-r from-red-800 to-red-600 text-white p-6 rounded-lg shadow-lg mb-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <div class="bg-white/20 p-3 rounded-full">
                        <i class={format!("fas {icon_class} text-2xl")}></i>
                    </div>
                    <div>
                        <h1 class="text-2xl font-bold">{title}</h1>
                        <p class="text-red-100">{subtitle}</p>
                    </div>
                </div>
                <div class="text-right">
                    <div class="text-sm text-red-100">"Kejaksaan Agung RI"</div>
                    <div class="text-xs text-red-200">"Penyidikan Pidana Khusus"</div>
                </div>
            </div>
        </div>
    }
}

/// Badge untuk status kasus khusus
#[component]
pub fn SpecialCaseStatusBadge(status: SpecialCaseStatus) -> impl IntoView {
    let (bg_class, text_class, text) = match status {
        SpecialCaseStatus::Investigation => ("bg-blue-100", "text-blue-800", "Penyidikan"),
        SpecialCaseStatus::Evidence => ("bg-yellow-100", "text-yellow-800", "Pengumpulan Bukti"),
        SpecialCaseStatus::Analysis => ("bg-purple-100", "text-purple-800", "Analisis"),
        SpecialCaseStatus::Prosecution => ("bg-orange-100", "text-orange-800", "Penuntutan"),
        SpecialCaseStatus::Trial => ("bg-indigo-100", "text-indigo-800", "Persidangan"),
        SpecialCaseStatus::Appeal => ("bg-pink-100", "text-pink-800", "Banding"),
        SpecialCaseStatus::Execution => ("bg-green-100", "text-green-800", "Eksekusi"),
        SpecialCaseStatus::Closed => ("bg-gray-100", "text-gray-800", "Ditutup"),
    };

    view! {
        <span class={format!("px-3 py-1 rounded-full text-xs font-medium {bg_class} {text_class}")}>
            {text}
        </span>
    }
}

/// Badge untuk prioritas kasus khusus
#[component]
pub fn SpecialPriorityBadge(priority: SpecialCasePriority) -> impl IntoView {
    let (bg_class, text_class, text, icon) = match priority {
        SpecialCasePriority::Urgent => (
            "bg-red-100",
            "text-red-800",
            "Mendesak",
            "fa-exclamation-triangle",
        ),
        SpecialCasePriority::High => ("bg-orange-100", "text-orange-800", "Tinggi", "fa-arrow-up"),
        SpecialCasePriority::Medium => ("bg-yellow-100", "text-yellow-800", "Sedang", "fa-minus"),
        SpecialCasePriority::Low => ("bg-green-100", "text-green-800", "Rendah", "fa-arrow-down"),
    };

    view! {
        <span class={format!("inline-flex items-center px-3 py-1 rounded-full text-xs font-medium {bg_class} {text_class}")}>
            <i class={format!("fas {icon} mr-1")}></i>
            {text}
        </span>
    }
}

/// Badge untuk tingkat klasifikasi
#[component]
pub fn ClassificationBadge(classification: ClassificationLevel) -> impl IntoView {
    let (bg_class, text_class, text, icon) = match classification {
        ClassificationLevel::TopSecret => ("bg-black", "text-white", "Sangat Rahasia", "fa-lock"),
        ClassificationLevel::Secret => ("bg-red-600", "text-white", "Rahasia", "fa-user-secret"),
        ClassificationLevel::Confidential => {
            ("bg-orange-600", "text-white", "Terbatas", "fa-eye-slash")
        }
        ClassificationLevel::Internal => ("bg-blue-600", "text-white", "Internal", "fa-building"),
        ClassificationLevel::Public => ("bg-green-600", "text-white", "Publik", "fa-globe"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2 py-1 rounded text-xs font-bold {bg_class} {text_class}")}>
            <i class={format!("fas {icon} mr-1")}></i>
            {text}
        </span>
    }
}

/// Kartu statistik untuk PIDSUS
#[component]
pub fn SpecialStatCard(
    title: String,
    value: String,
    icon: String,
    color: String,
    trend: String,
) -> impl IntoView {
    let color_classes = match color.as_str() {
        "red" => ("bg-red-50", "text-red-600", "border-red-200"),
        "blue" => ("bg-blue-50", "text-blue-600", "border-blue-200"),
        "green" => ("bg-green-50", "text-green-600", "border-green-200"),
        "yellow" => ("bg-yellow-50", "text-yellow-600", "border-yellow-200"),
        "purple" => ("bg-purple-50", "text-purple-600", "border-purple-200"),
        _ => ("bg-gray-50", "text-gray-600", "border-gray-200"),
    };

    view! {
        <div class={format!("p-6 rounded-lg border-2 {} {}", color_classes.0, color_classes.2)}>
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-600">{title}</p>
                    <p class="text-3xl font-bold text-gray-900">{value}</p>
                    <p class="text-xs text-gray-500 mt-1">{trend}</p>
                </div>
                <div class={format!("p-3 rounded-full {} {}", color_classes.0, color_classes.1)}>
                    <i class={format!("fas {icon} text-xl")}></i>
                </div>
            </div>
        </div>
    }
}

/// Input pencarian khusus untuk PIDSUS
#[component]
pub fn SpecialSearchInput(placeholder: String) -> impl IntoView {
    let (search_term, set_search_term) = signal(String::new());

    view! {
        <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <i class="fas fa-search text-gray-400"></i>
            </div>
            <input
                type="text"
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-red-500 focus:border-red-500"
                placeholder=placeholder
                value=search_term.get()
                on:input=move |ev| {
                    set_search_term.set(event_target_value(&ev));
                }
            />
        </div>
    }
}

/// Progress bar untuk operasi khusus
#[component]
pub fn SpecialProgressBar(
    percentage: i32,
    #[prop(default = "blue".to_string())] color: String,
) -> impl IntoView {
    let color_class = match color.as_str() {
        "red" => "bg-red-600",
        "green" => "bg-green-600",
        "yellow" => "bg-yellow-600",
        "purple" => "bg-purple-600",
        _ => "bg-blue-600",
    };

    view! {
        <div class="w-full bg-gray-200 rounded-full h-2">
            <div
                class={format!("h-2 rounded-full transition-all duration-300 {color_class}")}
                style={format!("width: {percentage}%")}
            ></div>
        </div>
    }
}
