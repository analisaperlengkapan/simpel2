use crate::api::{CreateCasePayload, create_case, fetch_cases};
use crate::types::*;
use chrono::Utc;
use leptos::prelude::*;

// ... [Keep existing dumb components like MilitaryHeader, CaseStatusBadge, etc.]
// I will just paste them back or import them if I could split the file, but for now I will paste the whole file content
// and replace the data fetching part.
// Actually, `pages.rs` was huge. I should try to preserve the UI components and only change `PidmilDashboard`, `PidmilKasus`, etc.

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
pub fn ProgressBar(
    #[prop()] percentage: u8,
    #[prop(optional)] color: Option<String>,
) -> impl IntoView {
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

    // Use LocalResource for fetching recent cases
    let cases_resource =
        LocalResource::new(move || async move { fetch_cases().await.unwrap_or_default() });

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Dashboard PIDMIL".to_string()
                subtitle="Penyidikan Pidana Militer - Kejaksaan Agung RI".to_string()
                icon_class="fa-shield-alt".to_string()
            />

            // Quick Statistics (Mock for now, would be another API endpoint)
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

            // Recent Cases Section
            <div class="bg-white rounded-lg shadow-lg p-6">
                <div class="flex items-center justify-between mb-6">
                    <h2 class="text-xl font-bold text-gray-900">"Kasus Terbaru"</h2>
                    <MilitarySearchInput
                        placeholder="Cari kasus...".to_string()
                    />
                </div>

                <Suspense fallback=move || view! { <p class="text-center py-4">"Loading cases..."</p> }>
                    <div class="space-y-4">
                        {move || {
                            cases_resource.get().map(|cases| {
                                cases.into_iter().take(5).map(|case| view! {
                                    <div class="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow">
                                        <div class="flex items-start justify-between">
                                            <div class="flex-1">
                                                <div class="flex items-center gap-3 mb-2">
                                                    <h3 class="font-semibold text-lg text-gray-900">{case.case_number}</h3>
                                                    <CaseStatusBadge status=case.status.clone() />
                                                    <PriorityBadge priority=case.priority.clone() />
                                                </div>
                                                <p class="text-gray-700 font-medium mb-1">{case.title}</p>
                                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-gray-500">
                                                    <div>
                                                        <span class="font-medium">Penyidik:</span><br/>
                                                        {case.assigned_investigator}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }).collect::<Vec<_>>()
                            })
                        }}
                    </div>
                </Suspense>
            </div>
        </div>
    }
}

/// PIDMIL Cases Management Page
#[component]
pub fn PidmilKasus() -> impl IntoView {
    // Resource for cases
    let cases_resource =
        LocalResource::new(move || async move { fetch_cases().await.unwrap_or_default() });

    // Action for creating a case
    let create_case_action = Action::new_local(|input: &CreateCasePayload| {
        let payload = input.clone();
        async move { create_case(payload).await }
    });

    view! {
        <div class="space-y-6">
            <MilitaryHeader
                title="Manajemen Kasus".to_string()
                subtitle="Kelola kasus penyidikan pidana militer".to_string()
                icon_class="fa-folder".to_string()
            />

            <div class="bg-white rounded-lg shadow-lg overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h3 class="text-lg font-medium text-gray-900">"Daftar Kasus PIDMIL"</h3>
                </div>
                <div class="overflow-x-auto">
                    <Suspense fallback=move || view! { <p class="p-4">"Loading cases..."</p> }>
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"No. Kasus"</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Judul"</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Status"</th>
                                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">"Prioritas"</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                {move || cases_resource.get().map(|cases| {
                                    cases.into_iter().map(|case| view! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                                                {case.case_number}
                                            </td>
                                            <td class="px-6 py-4 text-sm text-gray-900">
                                                <div class="max-w-xs truncate">{case.title}</div>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <CaseStatusBadge status=case.status.clone() />
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <PriorityBadge priority=case.priority.clone() />
                                            </td>
                                        </tr>
                                    }).collect::<Vec<_>>()
                                })}
                            </tbody>
                        </table>
                    </Suspense>
                </div>
            </div>

             // Simple form for creating case (Demo)
            <div class="bg-white rounded-lg shadow p-6">
                <h3 class="text-lg font-medium mb-4">"Tambah Kasus Baru"</h3>
                <button
                    class="bg-red-600 text-white px-4 py-2 rounded"
                    on:click=move |_| {
                        create_case_action.dispatch(CreateCasePayload {
                            title: "Kasus Baru Demo".to_string(),
                            case_type: "Corruption".to_string(),
                            description: "Kasus dibuat dari frontend".to_string(),
                            priority: "High".to_string(),
                            assigned_investigator: "Penyidik A".to_string(),
                            unit_involved: "Unit X".to_string(),
                            location: "Jakarta".to_string(),
                        });
                    }
                >
                    "Buat Kasus Demo"
                </button>
            </div>
        </div>
    }
}

/// PIDMIL Suspects Management Page
#[component]
pub fn PidmilTersangka() -> impl IntoView {
    // Restoring mock data for demonstration if API endpoint for suspects isn't fully ready in this view
    // Ideally this would fetch from API similar to cases
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
                        // ... details
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

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                 <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200">
                    <h3 class="font-semibold text-lg text-gray-900">"Laporan Kasus Bulanan"</h3>
                    <p class="text-sm text-gray-600 mb-4">"Ringkasan kasus dan penyidikan bulan ini"</p>
                     <MilitaryActionButton
                        label="Download".to_string()
                        action="download-monthly".to_string()
                        variant="secondary".to_string()
                        icon="fa-download".to_string()
                    />
                 </div>
                 // ... more reports
            </div>
        </div>
    }
}

/// PIDMIL Investigation Management Page
#[component]
pub fn PidmilPenyidikan() -> impl IntoView {
    // Mock data
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

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                 {move || investigations.get().into_iter().map(|inv| view! {
                    <div class="bg-white rounded-lg shadow-lg p-6 border border-gray-200">
                        <h3 class="font-semibold text-lg text-gray-900">{inv.case_id.clone()}</h3>
                        <p class="text-gray-600 mb-2">{inv.investigator_name.clone()}</p>
                        <ProgressBar percentage=inv.progress_percentage />
                    </div>
                 }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
