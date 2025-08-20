use leptos::prelude::*;

// Strategic Planning Domain Models
#[derive(Debug, Clone, PartialEq)]
pub struct StrategicPlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub timeline: PlanTimeline,
    pub budget: BudgetPlan,
    pub priority: PlanPriority,
    pub status: PlanStatus,
    pub progress: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTimeline {
    pub start_date: String,
    pub end_date: String,
    pub duration_months: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetPlan {
    pub total_budget: f64,
    pub allocated_budget: f64,
    pub utilized_budget: f64,
    pub remaining_budget: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanStatus {
    Draft,
    InProgress,
    Completed,
    OnHold,
    Cancelled,
}

// UI Components for Planning
#[component]
pub fn PlanningHeader(title: String, subtitle: String) -> impl IntoView {
    view! {
        <div class="bg-gradient-to-r from-blue-600 to-blue-800 text-white p-8 rounded-xl shadow-lg mb-8">
            <h1 class="text-3xl font-bold mb-2">{title}</h1>
            <p class="text-blue-100 text-lg">{subtitle}</p>
        </div>
    }
}

#[component]
pub fn StatsCard(
    title: String,
    value: String,
    change: String,
    icon: String,
    positive: bool,
) -> impl IntoView {
    let change_color = if positive {
        "text-green-600"
    } else {
        "text-red-600"
    };

    view! {
        <div class="bg-white rounded-xl shadow-lg p-6 hover:shadow-xl transition-all duration-200">
            <div class="flex items-center justify-between mb-4">
                <div class="text-3xl">{icon}</div>
                <div class={format!("text-sm font-medium {change_color}")}>
                    {change}
                </div>
            </div>
            <h3 class="text-2xl font-bold text-gray-900 mb-1">{value}</h3>
            <p class="text-gray-600 text-sm">{title}</p>
        </div>
    }
}

#[component]
pub fn StatusBadge(status: PlanStatus) -> impl IntoView {
    let (text, color) = match status {
        PlanStatus::Draft => ("Draft", "bg-gray-100 text-gray-800"),
        PlanStatus::InProgress => ("In Progress", "bg-blue-100 text-blue-800"),
        PlanStatus::Completed => ("Completed", "bg-green-100 text-green-800"),
        PlanStatus::OnHold => ("On Hold", "bg-yellow-100 text-yellow-800"),
        PlanStatus::Cancelled => ("Cancelled", "bg-red-100 text-red-800"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

#[component]
pub fn PriorityBadge(priority: PlanPriority) -> impl IntoView {
    let (text, color) = match priority {
        PlanPriority::Low => ("Low", "bg-gray-100 text-gray-800"),
        PlanPriority::Medium => ("Medium", "bg-yellow-100 text-yellow-800"),
        PlanPriority::High => ("High", "bg-orange-100 text-orange-800"),
        PlanPriority::Critical => ("Critical", "bg-red-100 text-red-800"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

#[component]
pub fn ProgressBar(progress: f64) -> impl IntoView {
    view! {
        <div class="w-full bg-gray-200 rounded-full h-2">
            <div
                class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={format!("width: {progress}%")}
            ></div>
        </div>
    }
}

#[component]
pub fn SearchInput(placeholder: String) -> impl IntoView {
    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full pl-10 pr-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                placeholder=placeholder
            />
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn PerencanaanDashboard() -> impl IntoView {
    // Mock data for strategic plans
    let strategic_plans = vec![
        StrategicPlan {
            id: "sp-001".to_string(),
            name: "Modernisasi Teknologi Informasi".to_string(),
            description: "Peningkatan infrastruktur dan sistem informasi Kejaksaan".to_string(),
            timeline: PlanTimeline {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-12-31".to_string(),
                duration_months: 12,
            },
            budget: BudgetPlan {
                total_budget: 5_000_000_000.0,
                allocated_budget: 4_800_000_000.0,
                utilized_budget: 3_200_000_000.0,
                remaining_budget: 1_600_000_000.0,
            },
            priority: PlanPriority::High,
            status: PlanStatus::InProgress,
            progress: 67.5,
            created_at: "2024-01-01".to_string(),
            updated_at: "2024-12-20".to_string(),
        },
        StrategicPlan {
            id: "sp-002".to_string(),
            name: "Pengembangan SDM Kejaksaan".to_string(),
            description: "Program pelatihan dan pengembangan kapasitas pegawai".to_string(),
            timeline: PlanTimeline {
                start_date: "2024-03-01".to_string(),
                end_date: "2025-02-28".to_string(),
                duration_months: 12,
            },
            budget: BudgetPlan {
                total_budget: 2_500_000_000.0,
                allocated_budget: 2_400_000_000.0,
                utilized_budget: 1_800_000_000.0,
                remaining_budget: 600_000_000.0,
            },
            priority: PlanPriority::Critical,
            status: PlanStatus::InProgress,
            progress: 75.0,
            created_at: "2024-02-15".to_string(),
            updated_at: "2024-12-20".to_string(),
        },
    ];

    view! {
        <div class="space-y-8">
            // Header Section
            <PlanningHeader
                title="Dashboard Perencanaan Strategis".to_string()
                subtitle="Monitor dan kelola perencanaan program strategis Kejaksaan RI".to_string()
            />

            // Key Metrics
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatsCard
                    title="Total Rencana Strategis".to_string()
                    value="24".to_string()
                    change="+3 bulan ini".to_string()
                    icon="📋".to_string()
                    positive=true
                />
                <StatsCard
                    title="Rencana Aktif".to_string()
                    value="18".to_string()
                    change="+2 minggu ini".to_string()
                    icon="🎯".to_string()
                    positive=true
                />
                <StatsCard
                    title="Total Anggaran".to_string()
                    value="Rp 12.5M".to_string()
                    change="+15% vs tahun lalu".to_string()
                    icon="💰".to_string()
                    positive=true
                />
                <StatsCard
                    title="Progress Rata-rata".to_string()
                    value="71%".to_string()
                    change="+5% bulan ini".to_string()
                    icon="📈".to_string()
                    positive=true
                />
            </div>

            // Search and Filter Section
            <div class="bg-white rounded-xl shadow-lg p-6">
                <div class="flex flex-col md:flex-row gap-4 mb-6">
                    <div class="flex-1">
                        <SearchInput placeholder="Cari rencana strategis...".to_string() />
                    </div>
                    <div class="md:w-64">
                        <select class="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500">
                            <option value="all">"Semua Rencana"</option>
                            <option value="strategic">"Rencana Strategis"</option>
                            <option value="operational">"Rencana Operasional"</option>
                            <option value="budget">"Rencana Anggaran"</option>
                        </select>
                    </div>
                </div>

                // Strategic Plans List
                <div class="space-y-4">
                    <h3 class="text-xl font-semibold text-gray-900 mb-4">"Rencana Strategis Aktif"</h3>
                    {strategic_plans.into_iter().map(|plan| {
                        view! {
                            <div class="border border-gray-200 rounded-lg p-6 hover:shadow-md transition-all duration-200">
                                <div class="flex justify-between items-start mb-4">
                                    <div class="flex-1">
                                        <h4 class="text-lg font-semibold text-gray-900 mb-2">{plan.name}</h4>
                                        <p class="text-gray-600 mb-3">{plan.description}</p>
                                        <div class="flex items-center space-x-4 text-sm text-gray-600">
                                            <span>"📅 " {plan.timeline.start_date} " - " {plan.timeline.end_date}</span>
                                            <span>"💰 Rp " {format!("{:.1}M", plan.budget.total_budget / 1_000_000.0)}</span>
                                        </div>
                                    </div>
                                    <div class="flex flex-col items-end space-y-2">
                                        <StatusBadge status=plan.status />
                                        <PriorityBadge priority=plan.priority />
                                    </div>
                                </div>

                                <div class="flex items-center justify-between">
                                    <div class="flex-1 mr-4">
                                        <div class="flex items-center justify-between text-sm text-gray-600 mb-1">
                                            <span>"Progress"</span>
                                            <span>{format!("{:.1}%", plan.progress)}</span>
                                        </div>
                                        <ProgressBar progress=plan.progress />
                                    </div>
                                    <div class="flex space-x-2">
                                        <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors">
                                            "Detail"
                                        </button>
                                        <button class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors">
                                            "Edit"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Recent Activities
            <div class="bg-white rounded-xl shadow-lg p-6">
                <h3 class="text-xl font-semibold text-gray-900 mb-4">"Aktivitas Terkini"</h3>
                <div class="space-y-3">
                    <div class="flex items-start space-x-3 p-3 border border-gray-200 rounded-lg">
                        <div class="text-blue-600 text-xl">"📝"</div>
                        <div class="flex-1">
                            <p class="text-gray-900 font-medium">"Rencana strategis 'Modernisasi TI' diperbarui"</p>
                            <p class="text-gray-600 text-sm">"Progress dinaikkan menjadi 67.5% - 2 jam yang lalu"</p>
                        </div>
                    </div>
                    <div class="flex items-start space-x-3 p-3 border border-gray-200 rounded-lg">
                        <div class="text-green-600 text-xl">"✅"</div>
                        <div class="flex-1">
                            <p class="text-gray-900 font-medium">"Milestone 'Implementasi Sistem Baru' tercapai"</p>
                            <p class="text-gray-600 text-sm">"Dari rencana 'Pengembangan SDM' - 1 hari yang lalu"</p>
                        </div>
                    </div>
                    <div class="flex items-start space-x-3 p-3 border border-gray-200 rounded-lg">
                        <div class="text-orange-600 text-xl">"⚠️"</div>
                        <div class="flex-1">
                            <p class="text-gray-900 font-medium">"Perhatian: Anggaran rencana 'Infrastruktur' mendekati batas"</p>
                            <p class="text-gray-600 text-sm">"Utilisasi sudah mencapai 85% - 3 hari yang lalu"</p>
                        </div>
                    </div>
                </div>
            </div>

            // Quick Actions
            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div class="bg-gradient-to-r from-blue-500 to-blue-600 text-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 cursor-pointer">
                    <div class="text-3xl mb-4">"➕"</div>
                    <h3 class="text-xl font-semibold mb-2">"Buat Rencana Baru"</h3>
                    <p class="text-blue-100">"Tambahkan rencana strategis atau operasional baru"</p>
                </div>

                <div class="bg-gradient-to-r from-green-500 to-green-600 text-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 cursor-pointer">
                    <div class="text-3xl mb-4">"📊"</div>
                    <h3 class="text-xl font-semibold mb-2">"Laporan Progress"</h3>
                    <p class="text-green-100">"Lihat laporan kemajuan semua rencana"</p>
                </div>

                <div class="bg-gradient-to-r from-purple-500 to-purple-600 text-white p-6 rounded-xl shadow-lg hover:shadow-xl transition-all duration-200 cursor-pointer">
                    <div class="text-3xl mb-4">"⚙️"</div>
                    <h3 class="text-xl font-semibold mb-2">"Pengaturan"</h3>
                    <p class="text-purple-100">"Konfigurasi template dan workflow"</p>
                </div>
            </div>
        </div>
    }
}
