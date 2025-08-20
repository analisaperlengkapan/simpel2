use crate::components::*;
use leptos::prelude::*;

#[component]
pub fn PerencanaanDashboard() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());
    let (selected_plan_type, set_selected_plan_type) = signal("all".to_string());
    
    // Mock data for strategic plans
    let (strategic_plans, _set_strategic_plans) = signal(vec![
        StrategicPlan {
            id: "sp-001".to_string(),
            name: "Modernisasi Teknologi Informasi".to_string(),
            description: "Peningkatan infrastruktur dan sistem informasi Kejaksaan".to_string(),
            objectives: vec![],
            timeline: PlanTimeline {
                start_date: "2024-01-01".to_string(),
                end_date: "2024-12-31".to_string(),
                duration_months: 12,
                milestones: vec![],
            },
            budget: BudgetPlan {
                total_budget: 5_000_000_000.0,
                allocated_budget: 4_800_000_000.0,
                utilized_budget: 3_200_000_000.0,
                remaining_budget: 1_600_000_000.0,
                categories: vec![],
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
            objectives: vec![],
            timeline: PlanTimeline {
                start_date: "2024-03-01".to_string(),
                end_date: "2025-02-28".to_string(),
                duration_months: 12,
                milestones: vec![],
            },
            budget: BudgetPlan {
                total_budget: 2_500_000_000.0,
                allocated_budget: 2_400_000_000.0,
                utilized_budget: 1_800_000_000.0,
                remaining_budget: 600_000_000.0,
                categories: vec![],
            },
            priority: PlanPriority::Critical,
            status: PlanStatus::InProgress,
            progress: 75.0,
            created_at: "2024-02-15".to_string(),
            updated_at: "2024-12-20".to_string(),
        },
    ]);

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
                        <SearchInput 
                            placeholder="Cari rencana strategis...".to_string()
                            value=search_query.into()
                        />
                    </div>
                    <div class="md:w-64">
                        <select 
                            class="w-full px-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            on:change=move |ev| {
                                set_selected_plan_type.set(event_target_value(&ev));
                            }
                        >
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
                    {move || {
                        let plans = strategic_plans.get();
                        if plans.is_empty() {
                            view! {
                                <EmptyState message="Belum ada rencana strategis yang dibuat".to_string() />
                            }.into_view()
                        } else {
                            plans.into_iter().map(|plan| {
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
                            }).collect::<Vec<_>>().into_view()
                        }
                    }}
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
