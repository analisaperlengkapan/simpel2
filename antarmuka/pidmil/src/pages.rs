use crate::types::*;
use chrono::Utc;
use leptos::prelude::*;

/// Military Case Header Component
#[component]
pub fn MilitaryHeader(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    #[prop(optional)] icon_class: Option<String>,
) -> impl IntoView {
    let icon = icon_class.unwrap_or_else(|| "fa-shield-alt".to_string());

    view! {
        <div class="bg-gradient-to-r from-red-800 to-red-900 text-white p-6 rounded-lg shadow-lg mb-6">
            <div class="flex items-center">
                <i class={format!("fas {icon} text-3xl mr-4")}></i>
                <div>
                    <h1 class="text-2xl font-bold">{title}</h1>
                    <p class="text-red-100">{subtitle}</p>
                </div>
            </div>
        </div>
    }
}

/// Case Status Badge Component
#[component]
pub fn CaseStatusBadge(status: CaseStatus) -> impl IntoView {
    let (class, text) = match status {
        CaseStatus::Reported => ("bg-blue-100 text-blue-800", "Dilaporkan"),
        CaseStatus::UnderInvestigation => ("bg-yellow-100 text-yellow-800", "Dalam Penyidikan"),
        CaseStatus::EvidenceCollection => ("bg-orange-100 text-orange-800", "Pengumpulan Bukti"),
        CaseStatus::SuspectIdentified => {
            ("bg-purple-100 text-purple-800", "Tersangka Teridentifikasi")
        }
        CaseStatus::AwaitingTrial => ("bg-indigo-100 text-indigo-800", "Menunggu Sidang"),
        CaseStatus::InTrial => ("bg-pink-100 text-pink-800", "Dalam Sidang"),
        CaseStatus::Concluded => ("bg-green-100 text-green-800", "Selesai"),
        CaseStatus::Dismissed => ("bg-gray-100 text-gray-800", "Dibatalkan"),
        CaseStatus::Appealed => ("bg-red-100 text-red-800", "Banding"),
    };

    view! {
        <span class={format!("px-2 py-1 rounded-full text-xs font-medium {class}")}>
            {text}
        </span>
    }
}

/// Priority Badge Component
#[component]
pub fn PriorityBadge(priority: CasePriority) -> impl IntoView {
    let (class, text) = match priority {
        CasePriority::Low => ("bg-gray-100 text-gray-800", "Rendah"),
        CasePriority::Medium => ("bg-blue-100 text-blue-800", "Sedang"),
        CasePriority::High => ("bg-orange-100 text-orange-800", "Tinggi"),
        CasePriority::Critical => ("bg-red-100 text-red-800", "Kritis"),
        CasePriority::TopSecret => ("bg-black text-white", "Rahasia"),
    };

    view! {
        <span class={format!("px-2 py-1 rounded-full text-xs font-medium {class}")}>
            {text}
        </span>
    }
}

/// Statistics Card Component for Military Dashboard
#[component]
pub fn MilitaryStatCard(
    #[prop(into)] title: String,
    #[prop(into)] value: String,
    #[prop(into)] icon: String,
    #[prop(optional)] trend: Option<String>,
    #[prop(optional)] color: Option<String>,
) -> impl IntoView {
    let color_class = color.unwrap_or_else(|| "red".to_string());
    let bg_class = format!("bg-{color_class}-50");
    let text_class = format!("text-{color_class}-600");
    let icon_class = format!("text-{color_class}-500");

    view! {
        <div class={format!("p-6 rounded-lg shadow-lg border border-gray-200 {bg_class}")}>
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-600">{title}</p>
                    <p class={format!("text-2xl font-bold {text_class}")}>{value}</p>
                    {trend.map(|t| view! { <p class="text-xs text-gray-500 mt-1">{t}</p> })}
                </div>
                <div class={"p-3 rounded-full bg-white shadow".to_string()}>
                    <i class={format!("fas {icon} text-xl {icon_class}")}></i>
                </div>
            </div>
        </div>
    }
}

/// Military Search Input Component
#[component]
pub fn MilitarySearchInput(
    #[prop(into)] placeholder: String,
    #[prop(optional)] on_search: Option<leptos::callback::Callback<String>>,
) -> impl IntoView {
    let (search_value, set_search_value) = signal("".to_string());

    view! {
        <div class="relative max-w-md">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <i class="fas fa-search text-gray-400"></i>
            </div>
            <input
                type="text"
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-red-500 focus:border-red-500"
                placeholder={placeholder}
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    set_search_value.set(val.clone());
                    if let Some(callback) = on_search {
                        callback.run(val);
                    }
                }
                value=move || search_value.get()
            />
        </div>
    }
}

/// Action Button Component
#[component]
pub fn MilitaryActionButton(
    #[prop(into)] label: String,
    #[prop(into)] action: String,
    #[prop(optional)] variant: Option<String>,
    #[prop(optional)] icon: Option<String>,
) -> impl IntoView {
    let variant_class = match variant.as_deref() {
        Some("secondary") => "bg-gray-600 hover:bg-gray-700 text-white",
        Some("danger") => "bg-red-600 hover:bg-red-700 text-white",
        Some("success") => "bg-green-600 hover:bg-green-700 text-white",
        _ => "bg-red-600 hover:bg-red-700 text-white",
    };

    view! {
        <button
            class={format!("inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 {variant_class}")}
            on:click=move |_| {
                web_sys::console::log_1(&format!("Military Action: {action}").into());
            }
        >
            {icon.map(|i| view! { <i class={format!("fas {i} mr-2")}></i> })}
            {label}
        </button>
    }
}

/// Progress Bar Component
#[component]
pub fn ProgressBar(#[prop()] percentage: u8, #[prop(optional)] color: Option<String>) -> impl IntoView {
    let color_class = color.unwrap_or_else(|| "red".to_string());
    let progress_class = format!("bg-{color_class}-600");
    let bg_class = format!("bg-{color_class}-200");

    view! {
        <div class={format!("w-full rounded-full h-2 {bg_class}")}>
            <div
                class={format!("h-2 rounded-full transition-all duration-300 {progress_class}")}
                style={format!("width: {percentage}%")}
            ></div>
        </div>
    }
}

/// PIDMIL Dashboard Page
#[component]
pub fn PidmilDashboard() -> impl IntoView {
    // Sample statistics data
    let (stats, _set_stats) = signal(MilitaryStatistics {
        total_cases: 127,
        active_cases: 15,
        completed_cases: 112,
        total_suspects: 89,
        suspects_in_custody: 12,
        conviction_rate: 94.2,
        average_investigation_time: 45,
    });

    // Sample recent cases
    let (recent_cases, _set_recent_cases) = signal(vec![
        MilitaryCase {
            id: "1".to_string(),
            case_number: "PIDMIL-2024-001".to_string(),
            case_type: CaseType::Corruption,
            title: "Korupsi Pengadaan Peralatan Militer".to_string(),
            description: "Dugaan korupsi dalam pengadaan peralatan militer senilai 2.5 miliar"
                .to_string(),
            status: CaseStatus::UnderInvestigation,
            priority: CasePriority::High,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            assigned_investigator: "Mayor CPI Budi Santoso".to_string(),
            unit_involved: "Kodam Jaya".to_string(),
            location: "Jakarta".to_string(),
            suspects_count: 3,
            evidence_count: 15,
        },
        MilitaryCase {
            id: "2".to_string(),
            case_number: "PIDMIL-2024-002".to_string(),
            case_type: CaseType::Desertion,
            title: "Kasus Desersi Berulang".to_string(),
            description: "Kasus desersi yang melibatkan beberapa anggota TNI".to_string(),
            status: CaseStatus::EvidenceCollection,
            priority: CasePriority::Medium,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            assigned_investigator: "Kapten CPI Ahmad Wijaya".to_string(),
            unit_involved: "Kodam Brawijaya".to_string(),
            location: "Malang".to_string(),
            suspects_count: 2,
            evidence_count: 8,
        },
    ]);

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Dashboard PIDMIL".to_string()
                subtitle="Penyidikan Pidana Militer - Kejaksaan Agung RI".to_string()
                icon_class="fa-shield-alt".to_string()
            />

            // Quick Statistics
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <MilitaryStatCard
                    title="Total Kasus".to_string()
                    value=stats.get().total_cases.to_string()
                    icon="fa-folder".to_string()
                    color="blue".to_string()
                    trend="Semua periode".to_string()
                />
                <MilitaryStatCard
                    title="Kasus Aktif".to_string()
                    value=stats.get().active_cases.to_string()
                    icon="fa-folder-open".to_string()
                    color="yellow".to_string()
                    trend="Dalam proses".to_string()
                />
                <MilitaryStatCard
                    title="Tersangka".to_string()
                    value=stats.get().total_suspects.to_string()
                    icon="fa-user-secret".to_string()
                    color="red".to_string()
                    trend=format!("{} dalam tahanan", stats.get().suspects_in_custody)
                />
                <MilitaryStatCard
                    title="Tingkat Konviksi".to_string()
                    value=format!("{:.1}%", stats.get().conviction_rate)
                    icon="fa-gavel".to_string()
                    color="green".to_string()
                    trend="Rata-rata tahunan".to_string()
                />
            </div>

            // Action Buttons
            <div class="flex flex-wrap gap-3">
                <MilitaryActionButton
                    label="Kasus Baru".to_string()
                    action="new-case".to_string()
                    icon="fa-plus".to_string()
                />
                <MilitaryActionButton
                    label="Penyidikan Baru".to_string()
                    action="new-investigation".to_string()
                    variant="secondary".to_string()
                    icon="fa-search".to_string()
                />
                <MilitaryActionButton
                    label="Laporan Mingguan".to_string()
                    action="weekly-report".to_string()
                    variant="success".to_string()
                    icon="fa-chart-line".to_string()
                />
            </div>

            // Recent Cases Section
            <div class="bg-white rounded-lg shadow-lg p-6">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-bold text-gray-900">"Kasus Terbaru"</h2>
                    <MilitarySearchInput
                        placeholder="Cari kasus...".to_string()
                    />
                </div>

                <div class="space-y-4">
                    {move || recent_cases.get().into_iter().map(|case| view! {
                        <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
                            <div class="flex items-start justify-between">
                                <div class="flex-1">
                                    <div class="flex items-center gap-3 mb-2">
                                        <h3 class="font-semibold text-lg text-gray-900">{case.case_number.clone()}</h3>
                                        <CaseStatusBadge status=case.status.clone() />
                                        <PriorityBadge priority=case.priority.clone() />
                                    </div>
                                    <p class="text-gray-700 font-medium mb-1">{case.title.clone()}</p>
                                    <p class="text-gray-600 text-sm mb-3">{case.description.clone()}</p>
                                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-gray-500">
                                        <div>
                                            <span class="font-medium">Penyidik:</span><br/>
                                            {case.assigned_investigator.clone()}
                                        </div>
                                        <div>
                                            <span class="font-medium">Unit:</span><br/>
                                            {case.unit_involved.clone()}
                                        </div>
                                        <div>
                                            <span class="font-medium">Tersangka:</span><br/>
                                            {case.suspects_count} orang
                                        </div>
                                        <div>
                                            <span class="font-medium">Bukti:</span><br/>
                                            {case.evidence_count} item
                                        </div>
                                    </div>
                                </div>
                                <div class="ml-4">
                                    <MilitaryActionButton
                                        label="Detail".to_string()
                                        action=format!("view-case-{}", case.id)
                                        variant="secondary".to_string()
                                        icon="fa-eye".to_string()
                                    />
                                </div>
                            </div>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Investigation Progress Overview
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4">"Kasus Prioritas Tinggi"</h3>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDMIL-2024-001"</span>
                            <span class="text-xs bg-red-100 text-red-800 px-2 py-1 rounded">"Kritis"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDMIL-2024-003"</span>
                            <span class="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">"Tinggi"</span>
                        </div>
                        <div class="flex items-center justify-between">
                            <span class="text-sm text-gray-600">"PIDMIL-2024-005"</span>
                            <span class="text-xs bg-orange-100 text-orange-800 px-2 py-1 rounded">"Tinggi"</span>
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4">"Progress Penyidikan"</h3>
                    <div class="space-y-4">
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Korupsi TNI-001"</span>
                                <span class="text-sm text-gray-600">"75%"</span>
                            </div>
                            <ProgressBar percentage=75 color="green".to_string() />
                        </div>
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Desersi MIL-045"</span>
                                <span class="text-sm text-gray-600">"45%"</span>
                            </div>
                            <ProgressBar percentage=45 color="yellow".to_string() />
                        </div>
                        <div>
                            <div class="flex justify-between mb-1">
                                <span class="text-sm text-gray-600">"Penyalahgunaan"</span>
                                <span class="text-sm text-gray-600">"20%"</span>
                            </div>
                            <ProgressBar percentage=20 color="red".to_string() />
                        </div>
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6">
                    <h3 class="text-lg font-semibold mb-4">"Status Tersangka"</h3>
                    <div class="space-y-3">
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Dalam Tahanan"</span>
                            <span class="font-semibold text-red-600">12</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Bebas Bersyarat"</span>
                            <span class="font-semibold text-yellow-600">8</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Menunggu Sidang"</span>
                            <span class="font-semibold text-blue-600">15</span>
                        </div>
                        <div class="flex justify-between">
                            <span class="text-sm text-gray-600">"Dalam Proses Banding"</span>
                            <span class="font-semibold text-purple-600">3</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// PIDMIL Investigation Management Page
#[component]
pub fn PidmilPenyidikan() -> impl IntoView {
    let (investigations, _set_investigations) = signal(vec![Investigation {
        id: "1".to_string(),
        case_id: "PIDMIL-2024-001".to_string(),
        investigator_name: "Mayor CPI Budi Santoso".to_string(),
        investigation_type: InvestigationType::Formal,
        status: InvestigationStatus::InProgress,
        start_date: Utc::now(),
        target_completion: Some(Utc::now()),
        progress_percentage: 75,
        findings: vec![],
        notes: "Investigasi korupsi pengadaan peralatan militer".to_string(),
        next_actions: vec![
            "Analisis dokumen keuangan".to_string(),
            "Wawancara saksi ahli".to_string(),
        ],
    }]);

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Penyidikan Militer".to_string()
                subtitle="Kelola proses penyidikan kasus pidana militer".to_string()
                icon_class="fa-search".to_string()
            />

            <div class="flex items-center justify-between">
                <MilitarySearchInput
                    placeholder="Cari penyidikan...".to_string()
                />
                <MilitaryActionButton
                    label="Penyidikan Baru".to_string()
                    action="add-investigation".to_string()
                    icon="fa-plus".to_string()
                />
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {move || investigations.get().into_iter().map(|inv| view! {
                    <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200">
                        <div class="flex items-start justify-between mb-4">
                            <div>
                                <h3 class="font-semibold text-lg text-gray-900">{inv.case_id.clone()}</h3>
                                <p class="text-gray-600">{inv.investigator_name.clone()}</p>
                            </div>
                            <span class="bg-blue-100 text-blue-800 px-2 py-1 rounded text-xs font-medium">
                                {format!("{:?}", inv.investigation_type)}
                            </span>
                        </div>

                        <div class="mb-4">
                            <div class="flex justify-between mb-2">
                                <span class="text-sm text-gray-600">"Progress"</span>
                                <span class="text-sm font-medium">{inv.progress_percentage}%</span>
                            </div>
                            <ProgressBar percentage=inv.progress_percentage />
                        </div>

                        <div class="space-y-2 mb-4">
                            <p class="text-sm text-gray-700">{inv.notes.clone()}</p>
                        </div>

                        <div class="space-y-2">
                            <h4 class="font-medium text-sm text-gray-900">"Tindakan Selanjutnya:"</h4>
                            {inv.next_actions.into_iter().map(|action| view! {
                                <div class="flex items-center text-sm text-gray-600">
                                    <i class="fas fa-arrow-right mr-2 text-xs"></i>
                                    {action}
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>

                        <div class="mt-4 pt-4 border-t flex justify-end space-x-2">
                            <MilitaryActionButton
                                label="Detail".to_string()
                                action=format!("view-investigation-{}", inv.id)
                                variant="secondary".to_string()
                                icon="fa-eye".to_string()
                            />
                            <MilitaryActionButton
                                label="Update".to_string()
                                action=format!("update-investigation-{}", inv.id)
                                icon="fa-edit".to_string()
                            />
                        </div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// PIDMIL Cases Management Page
#[component]
pub fn PidmilKasus() -> impl IntoView {
    let (cases, _set_cases) = signal(vec![
        MilitaryCase {
            id: "1".to_string(),
            case_number: "PIDMIL-2024-001".to_string(),
            case_type: CaseType::Corruption,
            title: "Korupsi Pengadaan Peralatan Militer".to_string(),
            description: "Dugaan korupsi dalam pengadaan peralatan militer".to_string(),
            status: CaseStatus::UnderInvestigation,
            priority: CasePriority::High,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            assigned_investigator: "Mayor CPI Budi Santoso".to_string(),
            unit_involved: "Kodam Jaya".to_string(),
            location: "Jakarta".to_string(),
            suspects_count: 3,
            evidence_count: 15,
        },
        MilitaryCase {
            id: "2".to_string(),
            case_number: "PIDMIL-2024-002".to_string(),
            case_type: CaseType::Desertion,
            title: "Kasus Desersi Berulang".to_string(),
            description: "Kasus desersi yang melibatkan beberapa anggota TNI".to_string(),
            status: CaseStatus::EvidenceCollection,
            priority: CasePriority::Medium,
            created_date: Utc::now(),
            updated_date: Utc::now(),
            assigned_investigator: "Kapten CPI Ahmad Wijaya".to_string(),
            unit_involved: "Kodam Brawijaya".to_string(),
            location: "Malang".to_string(),
            suspects_count: 2,
            evidence_count: 8,
        },
    ]);

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Manajemen Kasus".to_string()
                subtitle="Kelola kasus penyidikan pidana militer".to_string()
                icon_class="fa-folder".to_string()
            />

            <div class="flex items-center justify-between">
                <MilitarySearchInput
                    placeholder="Cari kasus...".to_string()
                />
                <MilitaryActionButton
                    label="Kasus Baru".to_string()
                    action="add-case".to_string()
                    icon="fa-plus".to_string()
                />
            </div>

            <div class="bg-white rounded-lg shadow-lg overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h3 class="text-lg font-medium text-gray-900">"Daftar Kasus PIDMIL"</h3>
                </div>
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"No. Kasus"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Judul"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Jenis"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Prioritas"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Penyidik"</th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Aksi"</th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            {move || cases.get().into_iter().map(|case| view! {
                                <tr class="hover:bg-gray-50">
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                        {case.case_number.clone()}
                                    </td>
                                    <td class="px-6 py-4 text-sm text-gray-900">
                                        <div class="max-w-xs truncate">{case.title.clone()}</div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                        {format!("{:?}", case.case_type)}
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <CaseStatusBadge status=case.status.clone() />
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <PriorityBadge priority=case.priority.clone() />
                                    </td>
                                    <td class="px-6 py-4 text-sm text-gray-900">
                                        <div class="max-w-xs truncate">{case.assigned_investigator.clone()}</div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm font-medium">
                                        <div class="flex space-x-2">
                                            <MilitaryActionButton
                                                label="Detail".to_string()
                                                action=format!("view-case-{}", case.id)
                                                variant="secondary".to_string()
                                                icon="fa-eye".to_string()
                                            />
                                        </div>
                                    </td>
                                </tr>
                            }).collect::<Vec<_>>()}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}

/// PIDMIL Suspects Management Page
#[component]
pub fn PidmilTersangka() -> impl IntoView {
    let (suspects, _set_suspects) = signal(vec![MilitarySuspect {
        id: "1".to_string(),
        nrp: "31850012345678".to_string(),
        name: "Mayor XYZ".to_string(),
        rank: MilitaryRank::Mayor,
        unit: "Kodam Jaya".to_string(),
        position: "Kepala Logistik".to_string(),
        case_id: "PIDMIL-2024-001".to_string(),
        status: SuspectStatus::Accused,
        arrest_date: Some(Utc::now()),
        detention_status: DetentionStatus::Detained,
        charges: vec!["Korupsi".to_string(), "Penyalahgunaan Wewenang".to_string()],
        contact_info: ContactInfo {
            phone: Some("081234567890".to_string()),
            emergency_contact: Some("081234567891".to_string()),
            address: Some("Jakarta Selatan".to_string()),
            next_of_kin: Some("Istri - Ny. ABC".to_string()),
        },
    }]);

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Manajemen Tersangka".to_string()
                subtitle="Kelola data tersangka dalam kasus pidana militer".to_string()
                icon_class="fa-user-secret".to_string()
            />

            <div class="flex items-center justify-between">
                <MilitarySearchInput
                    placeholder="Cari tersangka...".to_string()
                />
                <MilitaryActionButton
                    label="Tambah Tersangka".to_string()
                    action="add-suspect".to_string()
                    icon="fa-plus".to_string()
                />
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {move || suspects.get().into_iter().map(|suspect| view! {
                    <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200">
                        <div class="flex items-start justify-between mb-4">
                            <div>
                                <h3 class="font-semibold text-lg text-gray-900">{suspect.name.clone()}</h3>
                                <p class="text-gray-600">{format!("{:?} - {}", suspect.rank, suspect.nrp)}</p>
                            </div>
                            <div class="text-right">
                                <span class={format!("px-2 py-1 rounded text-xs font-medium {}",
                                    match suspect.detention_status {
                                        DetentionStatus::Detained => "bg-red-100 text-red-800",
                                        DetentionStatus::OnBail => "bg-yellow-100 text-yellow-800",
                                        DetentionStatus::Free => "bg-green-100 text-green-800",
                                        _ => "bg-gray-100 text-gray-800",
                                    }
                                )}>
                                    {format!("{:?}", suspect.detention_status)}
                                </span>
                            </div>
                        </div>

                        <div class="space-y-3 mb-4">
                            <div class="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <span class="font-medium text-gray-700">"Unit:"</span>
                                    <p class="text-gray-600">{suspect.unit.clone()}</p>
                                </div>
                                <div>
                                    <span class="font-medium text-gray-700">"Posisi:"</span>
                                    <p class="text-gray-600">{suspect.position.clone()}</p>
                                </div>
                            </div>
                            <div>
                                <span class="font-medium text-gray-700">"Tuduhan:"</span>
                                <div class="flex flex-wrap gap-1 mt-1">
                                    {suspect.charges.into_iter().map(|charge| view! {
                                        <span class="bg-red-100 text-red-800 px-2 py-1 rounded text-xs">
                                            {charge}
                                        </span>
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>

                        <div class="mt-4 pt-4 border-t flex justify-end space-x-2">
                            <MilitaryActionButton
                                label="Profile".to_string()
                                action=format!("view-suspect-{}", suspect.id)
                                variant="secondary".to_string()
                                icon="fa-user".to_string()
                            />
                            <MilitaryActionButton
                                label="Update".to_string()
                                action=format!("update-suspect-{}", suspect.id)
                                icon="fa-edit".to_string()
                            />
                        </div>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// PIDMIL Reports Page
#[component]
pub fn PidmilLaporan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Laporan PIDMIL".to_string()
                subtitle="Laporan penyidikan dan statistik kasus pidana militer".to_string()
                icon_class="fa-chart-bar".to_string()
            />

            <div class="flex items-center justify-between">
                <MilitarySearchInput
                    placeholder="Cari laporan...".to_string()
                />
                <MilitaryActionButton
                    label="Generate Laporan".to_string()
                    action="generate-report".to_string()
                    icon="fa-plus".to_string()
                />
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-blue-100 mr-4">
                            <i class="fas fa-calendar-alt text-blue-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Laporan Kasus Bulanan"</h3>
                            <p class="text-sm text-gray-600">"Ringkasan kasus dan penyidikan bulan ini"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Periode: Januari 2024"</p>
                        <p>"Total kasus: 15"</p>
                        <p>"Updated: Hari ini"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-monthly-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-red-100 mr-4">
                            <i class="fas fa-user-secret text-red-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Statistik Tersangka"</h3>
                            <p class="text-sm text-gray-600">"Analisis data tersangka dan status hukum"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Tersangka aktif: 89"</p>
                        <p>"Dalam tahanan: 12"</p>
                        <p>"Updated: Kemarin"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-suspects-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-green-100 mr-4">
                            <i class="fas fa-chart-line text-green-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Laporan Kinerja"</h3>
                            <p class="text-sm text-gray-600">"Evaluasi kinerja tim penyidik militer"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Tingkat penyelesaian: 94.2%"</p>
                        <p>"Rata-rata waktu: 45 hari"</p>
                        <p>"Updated: 3 hari lalu"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-performance-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-purple-100 mr-4">
                            <i class="fas fa-search text-purple-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Laporan Penyidikan"</h3>
                            <p class="text-sm text-gray-600">"Detail progress penyidikan per kasus"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Penyidikan aktif: 15"</p>
                        <p>"Menunggu sidang: 8"</p>
                        <p>"Updated: Hari ini"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-investigation-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-yellow-100 mr-4">
                            <i class="fas fa-archive text-yellow-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Laporan Barang Bukti"</h3>
                            <p class="text-sm text-gray-600">"Inventori dan status barang bukti"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Total bukti: 247"</p>
                        <p>"Dalam analisis: 23"</p>
                        <p>"Updated: 2 hari lalu"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-evidence-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>

                <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200 hover:shadow-xl transition-shadow cursor-pointer">
                    <div class="flex items-center mb-4">
                        <div class="p-3 rounded-full bg-indigo-100 mr-4">
                            <i class="fas fa-gavel text-indigo-600"></i>
                        </div>
                        <div>
                            <h3 class="font-semibold text-lg text-gray-900">"Laporan Putusan"</h3>
                            <p class="text-sm text-gray-600">"Ringkasan putusan pengadilan militer"</p>
                        </div>
                    </div>
                    <div class="text-sm text-gray-500 mb-4">
                        <p>"Putusan selesai: 112"</p>
                        <p>"Tingkat konviksi: 94.2%"</p>
                        <p>"Updated: Minggu lalu"</p>
                    </div>
                    <div class="flex justify-end">
                        <MilitaryActionButton
                            label="Download".to_string()
                            action="download-verdict-report".to_string()
                            variant="secondary".to_string()
                            icon="fa-download".to_string()
                        />
                    </div>
                </div>
            </div>
        </div>
    }
}
