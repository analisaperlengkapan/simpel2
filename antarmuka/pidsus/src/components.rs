use leptos::*;
use crate::types::*;

/// Timeline komponen untuk pelacakan kasus PIDSUS
#[component]
pub fn SpecialTimeline(case_id: String) -> impl IntoView {
    // Sample timeline data
    let timeline_events = vec![
        ("2024-01-15", "Laporan Diterima", "Laporan dugaan tindak pidana khusus diterima dari pelapor", "fa-flag"),
        ("2024-01-18", "Penyidikan Dimulai", "Tim PIDSUS memulai penyidikan awal", "fa-search"),
        ("2024-01-25", "Pengumpulan Bukti", "Proses pengumpulan dan analisis bukti fisik", "fa-box"),
        ("2024-02-01", "Analisis Forensik", "Analisis forensik digital dan keuangan", "fa-microscope"),
        ("2024-02-10", "Koordinasi Lintas Instansi", "Koordinasi dengan instansi terkait", "fa-handshake"),
        ("2024-02-15", "Penetapan Tersangka", "Penetapan tersangka berdasarkan bukti", "fa-user-secret"),
    ];

    view! {
        <div class="relative">
            <div class="absolute left-4 top-0 bottom-0 w-0.5 bg-gray-300"></div>
            <div class="space-y-6">
                {timeline_events.into_iter().enumerate().map(|(index, (date, title, description, icon))| {
                    let is_completed = index < 4; // Assume first 4 are completed
                    let is_current = index == 4; // 5th item is current
                    
                    view! {
                        <div class="relative flex items-start space-x-4">
                            <div class={format!("flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center {}", 
                                if is_completed {
                                    "bg-green-500 text-white"
                                } else if is_current {
                                    "bg-blue-500 text-white"
                                } else {
                                    "bg-gray-300 text-gray-600"
                                }
                            )}>
                                <i class={format!("fas {} text-sm", icon)}></i>
                            </div>
                            <div class="flex-1 min-w-0">
                                <div class="flex items-center space-x-2">
                                    <h3 class={format!("text-sm font-medium {}", 
                                        if is_completed || is_current { "text-gray-900" } else { "text-gray-500" }
                                    )}>
                                        {title}
                                    </h3>
                                    <span class="text-xs text-gray-500">{date}</span>
                                </div>
                                <p class={format!("mt-1 text-sm {}", 
                                    if is_completed || is_current { "text-gray-600" } else { "text-gray-400" }
                                )}>
                                    {description}
                                </p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

/// Komponen untuk menampilkan statistik operasi khusus
#[component]
pub fn SpecialOperationStats() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <i class="fas fa-bullseye text-3xl text-blue-500"></i>
                    </div>
                    <div class="ml-4">
                        <p class="text-sm font-medium text-gray-500">"Operasi Aktif"</p>
                        <p class="text-2xl font-semibold text-gray-900">"12"</p>
                        <p class="text-sm text-green-600">"↑ 2 dari bulan lalu"</p>
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <i class="fas fa-users text-3xl text-green-500"></i>
                    </div>
                    <div class="ml-4">
                        <p class="text-sm font-medium text-gray-500">"Tim Terlibat"</p>
                        <p class="text-2xl font-semibold text-gray-900">"45"</p>
                        <p class="text-sm text-blue-600">"Multi-instansi"</p>
                    </div>
                </div>
            </div>

            <div class="bg-white rounded-lg shadow p-6">
                <div class="flex items-center">
                    <div class="flex-shrink-0">
                        <i class="fas fa-globe text-3xl text-purple-500"></i>
                    </div>
                    <div class="ml-4">
                        <p class="text-sm font-medium text-gray-500">"Kerjasama Internasional"</p>
                        <p class="text-2xl font-semibold text-gray-900">"8"</p>
                        <p class="text-sm text-purple-600">"Lintas negara"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Komponen untuk analisis forensik
#[component]
pub fn ForensicAnalysisCard(analysis_id: String) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold flex items-center">
                    <i class="fas fa-microscope text-purple-500 mr-2"></i>
                    "Analisis Forensik"
                </h3>
                <span class="px-3 py-1 bg-purple-100 text-purple-800 text-sm rounded-full">
                    "ID: " {analysis_id}
                </span>
            </div>

            <div class="space-y-4">
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <span class="text-sm font-medium text-gray-500">"Forensik Digital"</span>
                        <div class="mt-1">
                            <div class="bg-gray-200 rounded-full h-2">
                                <div class="bg-blue-600 h-2 rounded-full" style="width: 85%"></div>
                            </div>
                            <span class="text-xs text-gray-600">"85% Selesai"</span>
                        </div>
                    </div>
                    <div>
                        <span class="text-sm font-medium text-gray-500">"Forensik Keuangan"</span>
                        <div class="mt-1">
                            <div class="bg-gray-200 rounded-full h-2">
                                <div class="bg-green-600 h-2 rounded-full" style="width: 92%"></div>
                            </div>
                            <span class="text-xs text-gray-600">"92% Selesai"</span>
                        </div>
                    </div>
                </div>

                <div class="border-t pt-4">
                    <div class="flex justify-between items-center">
                        <span class="text-sm font-medium">"Temuan Kunci"</span>
                        <span class="text-sm text-blue-600">"23 item"</span>
                    </div>
                    <div class="flex justify-between items-center mt-2">
                        <span class="text-sm font-medium">"Bukti Digital"</span>
                        <span class="text-sm text-green-600">"156 file"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Komponen untuk kerjasama internasional
#[component]
pub fn InternationalCooperationCard() -> impl IntoView {
    let cooperations = vec![
        ("Singapura", "MLA Request", "Aktif", "green"),
        ("Malaysia", "Asset Recovery", "Pending", "yellow"),
        ("Australia", "Information Sharing", "Completed", "blue"),
        ("Swiss", "Bank Records", "In Progress", "orange"),
    ];

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <h3 class="text-lg font-semibold mb-4 flex items-center">
                <i class="fas fa-globe text-blue-500 mr-2"></i>
                "Kerjasama Internasional"
            </h3>

            <div class="space-y-3">
                {cooperations.into_iter().map(|(country, type_req, status, color)| {
                    let status_class = match color {
                        "green" => "bg-green-100 text-green-800",
                        "yellow" => "bg-yellow-100 text-yellow-800",
                        "blue" => "bg-blue-100 text-blue-800",
                        "orange" => "bg-orange-100 text-orange-800",
                        _ => "bg-gray-100 text-gray-800",
                    };

                    view! {
                        <div class="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                            <div class="flex items-center space-x-3">
                                <div class="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
                                    <i class="fas fa-flag text-blue-600 text-sm"></i>
                                </div>
                                <div>
                                    <p class="text-sm font-medium text-gray-900">{country}</p>
                                    <p class="text-xs text-gray-500">{type_req}</p>
                                </div>
                            </div>
                            <span class={format!("px-2 py-1 rounded-full text-xs font-medium {}", status_class)}>
                                {status}
                            </span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-4 pt-4 border-t">
                <button class="w-full text-left text-sm text-blue-600 hover:text-blue-800 font-medium">
                    "Lihat semua kerjasama →"
                </button>
            </div>
        </div>
    }
}

/// Komponen untuk monitoring kasus prioritas tinggi
#[component]
pub fn HighPriorityCasesMonitor() -> impl IntoView {
    let priority_cases = vec![
        ("PIDSUS-2024-001", "Korupsi BUMN", "Urgent", "red"),
        ("PIDSUS-2024-003", "Money Laundering", "High", "orange"),
        ("PIDSUS-2024-007", "Cyber Crime", "High", "orange"),
        ("PIDSUS-2024-009", "Human Trafficking", "Medium", "yellow"),
    ];

    view! {
        <div class="bg-white rounded-lg shadow-lg p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold flex items-center">
                    <i class="fas fa-exclamation-triangle text-red-500 mr-2"></i>
                    "Monitor Kasus Prioritas"
                </h3>
                <span class="text-sm text-gray-500">
                    {priority_cases.len()} " kasus"
                </span>
            </div>

            <div class="space-y-3">
                {priority_cases.into_iter().map(|(case_id, title, priority, color)| {
                    let priority_class = match color {
                        "red" => "bg-red-100 text-red-800 border-red-200",
                        "orange" => "bg-orange-100 text-orange-800 border-orange-200",
                        "yellow" => "bg-yellow-100 text-yellow-800 border-yellow-200",
                        _ => "bg-gray-100 text-gray-800 border-gray-200",
                    };

                    view! {
                        <div class={format!("p-3 border rounded-lg {}", priority_class)}>
                            <div class="flex items-center justify-between">
                                <div>
                                    <p class="font-medium text-sm">{case_id}</p>
                                    <p class="text-xs opacity-75">{title}</p>
                                </div>
                                <div class="text-right">
                                    <span class="text-xs font-bold">{priority}</span>
                                    <div class="flex items-center mt-1">
                                        {(0..3).map(|i| {
                                            let is_filled = match (priority, i) {
                                                ("Urgent", _) => true,
                                                ("High", 0..=1) => true,
                                                ("Medium", 0) => true,
                                                _ => false,
                                            };
                                            
                                            view! {
                                                <div class={format!("w-2 h-2 rounded-full mr-1 {}", 
                                                    if is_filled { 
                                                        match color {
                                                            "red" => "bg-red-600",
                                                            "orange" => "bg-orange-600", 
                                                            "yellow" => "bg-yellow-600",
                                                            _ => "bg-gray-400"
                                                        }
                                                    } else { 
                                                        "bg-gray-300" 
                                                    }
                                                )}></div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="mt-4 pt-4 border-t">
                <div class="flex justify-between text-sm">
                    <span class="text-gray-600">"Update terakhir:"</span>
                    <span class="text-gray-900">"5 menit lalu"</span>
                </div>
            </div>
        </div>
    }
}

/// Komponen alert untuk kasus mendesak
#[component]
pub fn UrgentCaseAlert() -> impl IntoView {
    view! {
        <div class="bg-red-50 border-l-4 border-red-400 p-4 mb-6">
            <div class="flex">
                <div class="flex-shrink-0">
                    <i class="fas fa-exclamation-triangle text-red-400"></i>
                </div>
                <div class="ml-3">
                    <p class="text-sm text-red-700">
                        <span class="font-medium">"Perhatian: "</span>
                        "Terdapat 2 kasus dengan prioritas URGENT yang memerlukan tindakan segera."
                    </p>
                    <div class="mt-2">
                        <div class="flex space-x-2">
                            <button class="bg-red-100 text-red-800 px-3 py-1 rounded text-xs font-medium hover:bg-red-200">
                                "Lihat Detail"
                            </button>
                            <button class="bg-white text-red-600 px-3 py-1 rounded text-xs font-medium border border-red-300 hover:bg-red-50">
                                "Tandai Sudah Dibaca"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Komponen untuk menampilkan statistik real-time
#[component]
pub fn RealTimeStats() -> impl IntoView {
    view! {
        <div class="bg-gradient-to-r from-blue-600 to-purple-600 text-white p-6 rounded-lg shadow-lg">
            <h3 class="text-lg font-semibold mb-4 flex items-center">
                <i class="fas fa-chart-line mr-2"></i>
                "Statistik Real-Time"
            </h3>
            
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div class="text-center">
                    <div class="text-2xl font-bold">"23"</div>
                    <div class="text-sm opacity-75">"Kasus Aktif"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold">"156"</div>
                    <div class="text-sm opacity-75">"Total Bukti"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold">"12"</div>
                    <div class="text-sm opacity-75">"Tim Operasi"</div>
                </div>
                <div class="text-center">
                    <div class="text-2xl font-bold">"89%"</div>
                    <div class="text-sm opacity-75">"Success Rate"</div>
                </div>
            </div>

            <div class="mt-4 pt-4 border-t border-white/20">
                <div class="flex items-center justify-between text-sm">
                    <span class="opacity-75">"Update terakhir:"</span>
                    <span class="font-medium">"2 detik lalu"</span>
                </div>
            </div>
        </div>
    }
} 