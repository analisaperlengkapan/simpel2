use leptos::prelude::*;
use leptos_meta::*;
use serde::{Deserialize, Serialize};

// Intelligence Operation Models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelOperation {
    pub id: String,
    pub operation_name: String,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub priority: PriorityLevel,
    pub start_date: String,
    pub end_date: Option<String>,
    pub target_description: String,
    pub assigned_agents: Vec<String>,
    pub classification_level: ClassificationLevel,
    pub progress_percentage: u8,
    pub collected_data_count: u32,
    pub reports_generated: u32,
    pub budget_allocated: f64,
    pub budget_used: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    SurveillanceOperation,
    InformationGathering,
    CounterIntelligence,
    CriminalInvestigation,
    CorruptionMonitoring,
    TerrorismPrevention,
    CyberSecurity,
    FinancialCrimeTracking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationStatus {
    Planning,
    Active,
    OnHold,
    Completed,
    Terminated,
    UnderReview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PriorityLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClassificationLevel {
    TopSecret,
    Secret,
    Confidential,
    Restricted,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntelReport {
    pub id: String,
    pub operation_id: String,
    pub title: String,
    pub report_type: ReportType,
    pub created_date: String,
    pub author: String,
    pub classification: ClassificationLevel,
    pub summary: String,
    pub key_findings: Vec<String>,
    pub recommendations: Vec<String>,
    pub attachments_count: u32,
    pub review_status: ReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    SituationReport,
    IntelligenceAssessment,
    ThreatAnalysis,
    OperationalUpdate,
    FinalReport,
    IncidentReport,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    RequiresRevision,
}

// Sample Intelligence Operations Data
fn get_sample_operations() -> Vec<IntelOperation> {
    vec![
        IntelOperation {
            id: "OPS-2025-001".to_string(),
            operation_name: "Operasi Mata Elang".to_string(),
            operation_type: OperationType::CorruptionMonitoring,
            status: OperationStatus::Active,
            priority: PriorityLevel::Critical,
            start_date: "2025-01-15".to_string(),
            end_date: Some("2025-08-30".to_string()),
            target_description: "Monitoring aktivitas korupsi di sektor pengadaan pemerintah"
                .to_string(),
            assigned_agents: vec![
                "Agent-001".to_string(),
                "Agent-007".to_string(),
                "Agent-012".to_string(),
            ],
            classification_level: ClassificationLevel::Secret,
            progress_percentage: 65,
            collected_data_count: 847,
            reports_generated: 23,
            budget_allocated: 2500000000.0,
            budget_used: 1625000000.0,
        },
        IntelOperation {
            id: "OPS-2025-002".to_string(),
            operation_name: "Operasi Cyber Shield".to_string(),
            operation_type: OperationType::CyberSecurity,
            status: OperationStatus::Active,
            priority: PriorityLevel::High,
            start_date: "2025-02-01".to_string(),
            end_date: None,
            target_description:
                "Monitoring dan pencegahan serangan siber terhadap infrastruktur kejaksaan"
                    .to_string(),
            assigned_agents: vec!["Cyber-Team-A".to_string(), "Cyber-Team-B".to_string()],
            classification_level: ClassificationLevel::TopSecret,
            progress_percentage: 80,
            collected_data_count: 1532,
            reports_generated: 15,
            budget_allocated: 1800000000.0,
            budget_used: 900000000.0,
        },
        IntelOperation {
            id: "OPS-2025-003".to_string(),
            operation_name: "Operasi Keuangan Bersih".to_string(),
            operation_type: OperationType::FinancialCrimeTracking,
            status: OperationStatus::Planning,
            priority: PriorityLevel::High,
            start_date: "2025-09-01".to_string(),
            end_date: Some("2026-02-28".to_string()),
            target_description: "Pelacakan aliran dana ilegal dan pencucian uang skala besar"
                .to_string(),
            assigned_agents: vec!["Financial-Unit".to_string()],
            classification_level: ClassificationLevel::Confidential,
            progress_percentage: 15,
            collected_data_count: 234,
            reports_generated: 2,
            budget_allocated: 3200000000.0,
            budget_used: 320000000.0,
        },
        IntelOperation {
            id: "OPS-2025-004".to_string(),
            operation_name: "Operasi Anti Terror".to_string(),
            operation_type: OperationType::TerrorismPrevention,
            status: OperationStatus::Active,
            priority: PriorityLevel::Critical,
            start_date: "2024-12-01".to_string(),
            end_date: None,
            target_description:
                "Pencegahan dan monitoring aktivitas teroris domestik dan internasional".to_string(),
            assigned_agents: vec![
                "Terror-Unit-Alpha".to_string(),
                "Terror-Unit-Beta".to_string(),
                "Intel-Support".to_string(),
            ],
            classification_level: ClassificationLevel::TopSecret,
            progress_percentage: 90,
            collected_data_count: 2145,
            reports_generated: 45,
            budget_allocated: 5000000000.0,
            budget_used: 4250000000.0,
        },
        IntelOperation {
            id: "OPS-2025-005".to_string(),
            operation_name: "Operasi Surveillance Urban".to_string(),
            operation_type: OperationType::SurveillanceOperation,
            status: OperationStatus::Completed,
            priority: PriorityLevel::Medium,
            start_date: "2024-10-01".to_string(),
            end_date: Some("2025-01-31".to_string()),
            target_description: "Surveillance aktivitas kriminal di area perkotaan prioritas"
                .to_string(),
            assigned_agents: vec!["Urban-Team-1".to_string(), "Urban-Team-2".to_string()],
            classification_level: ClassificationLevel::Restricted,
            progress_percentage: 100,
            collected_data_count: 956,
            reports_generated: 18,
            budget_allocated: 1200000000.0,
            budget_used: 1180000000.0,
        },
    ]
}

fn get_sample_reports() -> Vec<IntelReport> {
    vec![
        IntelReport {
            id: "RPT-2025-001".to_string(),
            operation_id: "OPS-2025-001".to_string(),
            title: "Laporan Bulanan Operasi Mata Elang - Januari 2025".to_string(),
            report_type: ReportType::OperationalUpdate,
            created_date: "2025-02-01".to_string(),
            author: "Senior Analyst-001".to_string(),
            classification: ClassificationLevel::Secret,
            summary: "Progress signifikan dalam mengidentifikasi pola korupsi di 15 instansi pemerintah. Ditemukan indikasi markup hingga 40% pada proyek infrastruktur.".to_string(),
            key_findings: vec![
                "Teridentifikasi 12 kasus markup proyek dengan total kerugian Rp 15.7 miliar".to_string(),
                "Ditemukan jaringan kolusi antara 8 perusahaan kontraktor".to_string(),
                "Pola transfer dana mencurigakan ke 23 rekening offshore".to_string(),
            ],
            recommendations: vec![
                "Lakukan penyelidikan mendalam terhadap 5 instansi prioritas".to_string(),
                "Koordinasi dengan KPK untuk tindak lanjut hukum".to_string(),
                "Perkuat monitoring sistem pengadaan elektronik".to_string(),
            ],
            attachments_count: 47,
            review_status: ReviewStatus::Approved,
        },
        IntelReport {
            id: "RPT-2025-002".to_string(),
            operation_id: "OPS-2025-002".to_string(),
            title: "Threat Assessment - Serangan Siber Q1 2025".to_string(),
            report_type: ReportType::ThreatAnalysis,
            created_date: "2025-03-31".to_string(),
            author: "Cyber Intel Team".to_string(),
            classification: ClassificationLevel::TopSecret,
            summary: "Peningkatan 300% serangan siber terhadap sistem pemerintah. Teridentifikasi 3 kelompok APT aktif dengan fokus pada data sensitif kejaksaan.".to_string(),
            key_findings: vec![
                "APT Group 'Shadow Prosecutor' melakukan 15 percobaan infiltrasi".to_string(),
                "Ditemukan 8 backdoor pada sistem pendukung".to_string(),
                "Serangan phishing meningkat 250% targeting jaksa dan staff".to_string(),
            ],
            recommendations: vec![
                "Implementasi zero-trust architecture".to_string(),
                "Pelatihan keamanan siber intensif untuk seluruh personel".to_string(),
                "Upgrade sistem keamanan endpoint".to_string(),
            ],
            attachments_count: 23,
            review_status: ReviewStatus::UnderReview,
        },
    ]
}

#[allow(dead_code)]
fn format_currency(amount: f64) -> String {
    format!("Rp {amount:.0}")
}

fn get_priority_color(priority: &PriorityLevel) -> &'static str {
    match priority {
        PriorityLevel::Critical => "text-red-600",
        PriorityLevel::High => "text-orange-600",
        PriorityLevel::Medium => "text-yellow-600",
        PriorityLevel::Low => "text-green-600",
    }
}

fn get_status_color(status: &OperationStatus) -> &'static str {
    match status {
        OperationStatus::Active => "text-green-600",
        OperationStatus::Planning => "text-blue-600",
        OperationStatus::OnHold => "text-yellow-600",
        OperationStatus::Completed => "text-gray-600",
        OperationStatus::Terminated => "text-red-600",
        OperationStatus::UnderReview => "text-purple-600",
    }
}

fn get_classification_badge(classification: &ClassificationLevel) -> &'static str {
    match classification {
        ClassificationLevel::TopSecret => "bg-red-100 text-red-800",
        ClassificationLevel::Secret => "bg-orange-100 text-orange-800",
        ClassificationLevel::Confidential => "bg-yellow-100 text-yellow-800",
        ClassificationLevel::Restricted => "bg-blue-100 text-blue-800",
        ClassificationLevel::Internal => "bg-gray-100 text-gray-800",
    }
}

#[component]
pub fn IntelligenceDashboard() -> impl IntoView {
    let operations = Memo::new(|_| get_sample_operations());
    let reports = Memo::new(|_| get_sample_reports());

    // Calculate dashboard statistics
    let total_operations = Memo::new(move |_| operations.get().len());
    let active_operations = Memo::new(move |_| {
        operations
            .get()
            .iter()
            .filter(|op| matches!(op.status, OperationStatus::Active))
            .count()
    });
    let total_data_collected = Memo::new(move |_| {
        operations
            .get()
            .iter()
            .map(|op| op.collected_data_count)
            .sum::<u32>()
    });
    let total_reports = Memo::new(move |_| {
        operations
            .get()
            .iter()
            .map(|op| op.reports_generated)
            .sum::<u32>()
    });
    view! {
        <div class="space-y-8">
            // Dashboard Statistics
            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <div class="bg-white p-6 rounded-lg shadow border-l-4 border-blue-500">
                    <h3 class="text-sm font-medium text-gray-500">"Total Operasi"</h3>
                    <p class="text-3xl font-bold text-blue-600">{move || total_operations.get()}</p>
                    <p class="text-sm text-gray-600 mt-1">"Semua kategori operasi"</p>
                </div>

                <div class="bg-white p-6 rounded-lg shadow border-l-4 border-green-500">
                    <h3 class="text-sm font-medium text-gray-500">"Operasi Aktif"</h3>
                    <p class="text-3xl font-bold text-green-600">{move || active_operations.get()}</p>
                    <p class="text-sm text-gray-600 mt-1">"Sedang berjalan"</p>
                </div>

                <div class="bg-white p-6 rounded-lg shadow border-l-4 border-purple-500">
                    <h3 class="text-sm font-medium text-gray-500">"Data Terkumpul"</h3>
                    <p class="text-3xl font-bold text-purple-600">{move || format!("{}", total_data_collected.get())}</p>
                    <p class="text-sm text-gray-600 mt-1">"Intelligence items"</p>
                </div>

                <div class="bg-white p-6 rounded-lg shadow border-l-4 border-orange-500">
                    <h3 class="text-sm font-medium text-gray-500">"Laporan"</h3>
                    <p class="text-3xl font-bold text-orange-600">{move || total_reports.get()}</p>
                    <p class="text-sm text-gray-600 mt-1">"Laporan dihasilkan"</p>
                </div>
            </div>

            // Active Operations Section
            <div class="bg-white rounded-lg shadow">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-xl font-semibold text-gray-900">"Operasi Intelligence Aktif"</h2>
                    <p class="text-sm text-gray-600">"Monitoring operasi intelijen yang sedang berjalan"</p>
                </div>
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Operasi"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Tipe"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Prioritas"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Progress"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Klasifikasi"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Data"</th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            <For
                                each=move || operations.get()
                                key=|operation| operation.id.clone()
                                children=move |operation: IntelOperation| {
                                    let priority_color = get_priority_color(&operation.priority);
                                    let status_color = get_status_color(&operation.status);
                                    let classification_badge = get_classification_badge(&operation.classification_level);

                                    view! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <div>
                                                    <div class="text-sm font-medium text-gray-900">{operation.operation_name.clone()}</div>
                                                    <div class="text-sm text-gray-500">{operation.id.clone()}</div>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <span class="text-sm text-gray-900">{format!("{:?}", operation.operation_type)}</span>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <span class={format!("text-sm font-medium {status_color}")}>
                                                    {format!("{:?}", operation.status)}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <span class={format!("text-sm font-medium {priority_color}")}>
                                                    {format!("{:?}", operation.priority)}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <div class="flex items-center">
                                                    <div class="flex-1 bg-gray-200 rounded-full h-2 mr-2">
                                                        <div class="bg-blue-600 h-2 rounded-full" style={format!("width: {}%", operation.progress_percentage)}></div>
                                                    </div>
                                                    <span class="text-sm text-gray-700">{operation.progress_percentage}"%"</span>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <span class={format!("inline-flex px-2 py-1 text-xs font-semibold rounded-full {classification_badge}")}>
                                                    {format!("{:?}", operation.classification_level)}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                <div>{format!("{}", operation.collected_data_count)}" items"</div>
                                                <div class="text-gray-500">{operation.reports_generated}" laporan"</div>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>

            // Recent Reports Section
            <div class="bg-white rounded-lg shadow">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h2 class="text-xl font-semibold text-gray-900">"Laporan Intelligence Terbaru"</h2>
                    <p class="text-sm text-gray-600">"Laporan dan analisis intelligence terkini"</p>
                </div>
                <div class="p-6">
                    <div class="space-y-4">
                        <For
                            each=move || reports.get()
                            key=|report| report.id.clone()
                            children=move |report: IntelReport| {
                                let classification_badge = get_classification_badge(&report.classification);

                                view! {
                                    <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
                                        <div class="flex justify-between items-start mb-3">
                                            <div class="flex-1">
                                                <h3 class="text-lg font-semibold text-gray-900">{report.title.clone()}</h3>
                                                <div class="flex items-center space-x-2 mt-1">
                                                    <span class="text-sm text-gray-500">
                                                        {report.created_date.clone()}" oleh "{report.author.clone()}
                                                    </span>
                                                    <span class={format!("inline-flex px-2 py-1 text-xs font-semibold rounded-full {classification_badge}")}>
                                                        {format!("{:?}", report.classification)}
                                                    </span>
                                                </div>
                                            </div>
                                            <span class="inline-flex px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded">
                                                {format!("{:?}", report.report_type)}
                                            </span>
                                        </div>

                                        <p class="text-gray-700 mb-3">{report.summary.clone()}</p>

                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div>
                                                <h4 class="text-sm font-medium text-gray-900 mb-2">"Temuan Kunci:"</h4>
                                                <ul class="text-sm text-gray-600 space-y-1">
                                                    <For
                                                        each=move || report.key_findings.clone()
                                                        key=|finding| finding.clone()
                                                        children=move |finding: String| {
                                                            view! {
                                                                <li class="flex items-start">
                                                                    <span class="text-blue-500 mr-2">"•"</span>
                                                                    <span>{finding}</span>
                                                                </li>
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            </div>

                                            <div>
                                                <h4 class="text-sm font-medium text-gray-900 mb-2">"Rekomendasi:"</h4>
                                                <ul class="text-sm text-gray-600 space-y-1">
                                                    <For
                                                        each=move || report.recommendations.clone()
                                                        key=|rec| rec.clone()
                                                        children=move |rec: String| {
                                                            view! {
                                                                <li class="flex items-start">
                                                                    <span class="text-green-500 mr-2">"→"</span>
                                                                    <span>{rec}</span>
                                                                </li>
                                                            }
                                                        }
                                                    />
                                                </ul>
                                            </div>
                                        </div>

                                        <div class="flex justify-between items-center mt-4 pt-3 border-t border-gray-100">
                                            <span class="text-sm text-gray-500">
                                                {report.attachments_count}" lampiran"
                                            </span>
                                            <span class={format!("text-sm font-medium {}",
                                                match report.review_status {
                                                    ReviewStatus::Approved => "text-green-600",
                                                    ReviewStatus::UnderReview => "text-yellow-600",
                                                    ReviewStatus::Draft => "text-gray-600",
                                                    ReviewStatus::Rejected => "text-red-600",
                                                    ReviewStatus::RequiresRevision => "text-orange-600",
                                                }
                                            )}>
                                                {format!("{:?}", report.review_status)}
                                            </span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="Intel - SIMPelv2"/>
        <Meta name="description" content="Sistem Intelligence - Kejaksaan Agung RI"/>
        <Meta name="keywords" content="intelligence, kejaksaan, monitoring, surveillance, intel"/>

        <div class="min-h-screen bg-gray-50">
            <IntelligenceDashboard />
        </div>
    }
}
