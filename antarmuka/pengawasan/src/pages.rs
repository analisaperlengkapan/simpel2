use crate::components::*;
use crate::types::*;
use leptos::prelude::*;

/// PENGAWASAN Dashboard - Main supervision oversight page
#[component]
pub fn PengawasanDashboard() -> impl IntoView {
    // Sample supervision statistics
    let supervision_stats = SupervisionStats {
        active_audits: 12,
        total_findings: 45,
        open_findings: 23,
        overdue_actions: 7,
        compliance_rate: 87.5,
        monitoring_items: 156,
        critical_risks: 4,
        pending_reports: 3,
    };

    // Sample recent audits
    let recent_audits = vec![
        Audit {
            id: "audit-001".to_string(),
            nomor_audit: "AUD/2024/001".to_string(),
            judul_audit: "Audit Kepatuhan Keuangan Q1 2024".to_string(),
            jenis_audit: AuditType::Financial,
            status: AuditStatus::InProgress,
            tanggal_mulai: "2024-01-15".to_string(),
            tanggal_selesai: None,
            auditor_utama: "Dr. Sari Kusumawati".to_string(),
            tim_audit: vec!["Ahmad Surya".to_string(), "Nina Pratiwi".to_string()],
            scope_audit: "Pelaporan keuangan, pengelolaan anggaran, dan compliance terhadap peraturan keuangan".to_string(),
            objektif: "Memastikan kepatuhan terhadap standar akuntansi dan regulasi keuangan".to_string(),
            progress: 65,
            temuan_count: 8,
            prioritas: Priority::High,
            created_at: "2024-01-10".to_string(),
            updated_at: "2024-01-20".to_string(),
        },
        Audit {
            id: "audit-002".to_string(),
            nomor_audit: "AUD/2024/002".to_string(),
            judul_audit: "Review Sistem Teknologi Informasi".to_string(),
            jenis_audit: AuditType::IT,
            status: AuditStatus::Planning,
            tanggal_mulai: "2024-02-01".to_string(),
            tanggal_selesai: None,
            auditor_utama: "Ir. Bambang Sutrisno".to_string(),
            tim_audit: vec!["Andi Wijaya".to_string(), "Lisa Sari".to_string()],
            scope_audit: "Infrastruktur IT, keamanan sistem, dan disaster recovery".to_string(),
            objektif: "Evaluasi efektivitas sistem IT dan keamanan data".to_string(),
            progress: 15,
            temuan_count: 0,
            prioritas: Priority::Medium,
            created_at: "2024-01-25".to_string(),
            updated_at: "2024-01-25".to_string(),
        },
    ];

    view! {
        <div class="space-y-8">
            // Dashboard Header
            <SupervisionHeader
                title="Dashboard Pengawasan".to_string()
                subtitle="Sistem Pengawasan dan Audit Internal Kejaksaan RI".to_string()
                actions=vec![
                    ("Audit Baru".to_string(), "create-audit".to_string()),
                    ("Laporan Eksekutif".to_string(), "executive-report".to_string()),
                ]
            />

            // Key Performance Indicators
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <StatsCard
                    title="Audit Aktif".to_string()
                    value=supervision_stats.active_audits.to_string()
                    icon="fas fa-clipboard-check".to_string()
                    color="blue".to_string()
                    description="Audit yang sedang berlangsung".to_string()
                />
                <StatsCard
                    title="Temuan Terbuka".to_string()
                    value=supervision_stats.open_findings.to_string()
                    icon="fas fa-exclamation-triangle".to_string()
                    color="yellow".to_string()
                    description="Memerlukan tindak lanjut".to_string()
                />
                <StatsCard
                    title="Compliance Rate".to_string()
                    value=format!("{:.1}%", supervision_stats.compliance_rate)
                    icon="fas fa-shield-alt".to_string()
                    color="green".to_string()
                    description="Tingkat kepatuhan sistem".to_string()
                />
                <StatsCard
                    title="Risiko Kritis".to_string()
                    value=supervision_stats.critical_risks.to_string()
                    icon="fas fa-exclamation-circle".to_string()
                    color="red".to_string()
                    description="Memerlukan perhatian segera".to_string()
                />
            </div>

            // Quick Actions and Overview
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
                // Recent Audits
                <div class="lg:col-span-2">
                    <div class="bg-white rounded-lg shadow-sm border border-gray-200">
                        <div class="p-6 border-b border-gray-200">
                            <div class="flex items-center justify-between">
                                <h3 class="text-lg font-semibold text-gray-900">"Audit Terbaru"</h3>
                                <ActionButton
                                    label="Lihat Semua".to_string()
                                    action="view-all-audits".to_string()
                                    variant=ButtonVariant::Secondary
                                />
                            </div>
                        </div>
                        <div class="p-6 space-y-4">
                            {recent_audits.into_iter().map(|audit| view! {
                                <div class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition-colors">
                                    <div class="flex items-start justify-between">
                                        <div class="flex-1">
                                            <h4 class="font-medium text-gray-900">{audit.nomor_audit.clone()}</h4>
                                            <p class="text-sm text-gray-600 mt-1">{audit.judul_audit.clone()}</p>
                                            <div class="flex items-center space-x-4 mt-3">
                                                <span class="text-xs text-gray-500">
                                                    "Auditor: " {audit.auditor_utama.clone()}
                                                </span>
                                                <StatusBadge
                                                    status=match audit.status {
                                                        AuditStatus::Planning => "Perencanaan".to_string(),
                                                        AuditStatus::InProgress => "Berlangsung".to_string(),
                                                        AuditStatus::Review => "Review".to_string(),
                                                        AuditStatus::Completed => "Selesai".to_string(),
                                                        AuditStatus::Cancelled => "Dibatalkan".to_string(),
                                                        AuditStatus::OnHold => "Ditunda".to_string(),
                                                    }
                                                    variant=match audit.status {
                                                        AuditStatus::Planning => BadgeVariant::Info,
                                                        AuditStatus::InProgress => BadgeVariant::Warning,
                                                        AuditStatus::Review => BadgeVariant::Info,
                                                        AuditStatus::Completed => BadgeVariant::Success,
                                                        AuditStatus::Cancelled => BadgeVariant::Danger,
                                                        AuditStatus::OnHold => BadgeVariant::Default,
                                                    }
                                                />
                                                <PriorityBadge priority=audit.prioritas.clone() />
                                            </div>
                                        </div>
                                        <div class="text-right ml-4">
                                            <div class="text-sm font-medium text-gray-900">
                                                {audit.progress}"%"
                                            </div>
                                            <div class="w-full bg-gray-200 rounded-full h-2 mt-1">
                                                <div
                                                    class=format!("h-2 rounded-full transition-all duration-300 bg-{}",
                                                        if audit.progress >= 75 { "green-500" } else if audit.progress >= 50 { "yellow-500" } else { "red-500" })
                                                    style=format!("width: {}%", audit.progress)
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>

                // Supervision Quick Actions
                <div class="space-y-6">
                    // Critical Alerts
                    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                        <h3 class="text-lg font-semibold text-gray-900 mb-4">"Alert Kritis"</h3>
                        <div class="space-y-3">
                            <Alert
                                message="7 tindak lanjut melewati deadline".to_string()
                                variant=AlertVariant::Warning
                            />
                            <Alert
                                message="4 risiko prioritas tinggi teridentifikasi".to_string()
                                variant=AlertVariant::Error
                            />
                            <Alert
                                message="Monitoring rutin dijadwalkan hari ini".to_string()
                                variant=AlertVariant::Info
                            />
                        </div>
                    </div>

                    // Quick Actions
                    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                        <h3 class="text-lg font-semibold text-gray-900 mb-4">"Aksi Cepat"</h3>
                        <div class="space-y-3">
                            <ActionButton
                                label="Buat Audit Baru".to_string()
                                action="create-audit".to_string()
                                variant=ButtonVariant::Primary
                                class="w-full".to_string()
                            />
                            <ActionButton
                                label="Review Temuan".to_string()
                                action="review-findings".to_string()
                                variant=ButtonVariant::Secondary
                                class="w-full".to_string()
                            />
                            <ActionButton
                                label="Monitor Compliance".to_string()
                                action="monitor-compliance".to_string()
                                variant=ButtonVariant::Success
                                class="w-full".to_string()
                            />
                            <ActionButton
                                label="Laporan Eksekutif".to_string()
                                action="executive-report".to_string()
                                variant=ButtonVariant::Warning
                                class="w-full".to_string()
                            />
                        </div>
                    </div>

                    // Compliance Overview
                    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
                        <h3 class="text-lg font-semibold text-gray-900 mb-4">"Ringkasan Kepatuhan"</h3>
                        <div class="space-y-4">
                            <div>
                                <div class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span>"Sistem Keuangan"</span>
                                    <span>"92%"</span>
                                </div>
                                <div class="w-full bg-gray-200 rounded-full h-2">
                                    <div class="h-2 rounded-full transition-all duration-300 bg-green-500" style="width: 92%"></div>
                                </div>
                            </div>
                            <div>
                                <div class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span>"Sistem Operasional"</span>
                                    <span>"85%"</span>
                                </div>
                                <div class="w-full bg-gray-200 rounded-full h-2">
                                    <div class="h-2 rounded-full transition-all duration-300 bg-yellow-500" style="width: 85%"></div>
                                </div>
                            </div>
                            <div>
                                <div class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                                    <span>"Keamanan IT"</span>
                                    <span>"78%"</span>
                                </div>
                                <div class="w-full bg-gray-200 rounded-full h-2">
                                    <div class="h-2 rounded-full transition-all duration-300 bg-red-500" style="width: 78%"></div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// PENGAWASAN Audit Management Page
#[component]
pub fn PengawasanAudit() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SupervisionHeader
                title="Manajemen Audit".to_string()
                subtitle="Kelola proses audit internal dan eksternal".to_string()
                actions=vec![
                    ("Audit Baru".to_string(), "create-audit".to_string()),
                    ("Import Data".to_string(), "import-audit".to_string()),
                ]
            />

            <div class="flex items-center space-x-4">
                <SearchInput placeholder="Cari audit berdasarkan nomor, judul, atau auditor...".to_string() />
                <ActionButton
                    label="Filter".to_string()
                    action="filter-audits".to_string()
                    variant=ButtonVariant::Secondary
                />
                <ActionButton
                    label="Export".to_string()
                    action="export-audits".to_string()
                    variant=ButtonVariant::Secondary
                />
            </div>

            <EmptyState
                title="Belum Ada Data Audit".to_string()
                description="Mulai membuat audit baru untuk memantau kepatuhan dan kinerja sistem".to_string()
                icon="fas fa-clipboard-check".to_string()
                action=("Buat Audit Pertama".to_string(), "create-first-audit".to_string())
            />
        </div>
    }
}

/// PENGAWASAN Monitoring & Compliance Page
#[component]
pub fn PengawasanMonitoring() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SupervisionHeader
                title="Monitoring & Kepatuhan".to_string()
                subtitle="Monitor kepatuhan sistem dan proses organisasi".to_string()
                actions=vec![
                    ("Monitor Baru".to_string(), "create-monitor".to_string()),
                    ("Scan Kepatuhan".to_string(), "compliance-scan".to_string()),
                ]
            />

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                <StatsCard
                    title="Sistem Dimonitor".to_string()
                    value="156".to_string()
                    icon="fas fa-desktop".to_string()
                    color="blue".to_string()
                />
                <StatsCard
                    title="Status Normal".to_string()
                    value="142".to_string()
                    icon="fas fa-check-circle".to_string()
                    color="green".to_string()
                />
                <StatsCard
                    title="Perlu Perhatian".to_string()
                    value="14".to_string()
                    icon="fas fa-exclamation-triangle".to_string()
                    color="yellow".to_string()
                />
            </div>

            <div class="flex items-center space-x-4">
                <SearchInput placeholder="Cari sistem atau proses monitoring...".to_string() />
                <ActionButton
                    label="Real-time View".to_string()
                    action="realtime-monitor".to_string()
                    variant=ButtonVariant::Success
                />
            </div>

            <EmptyState
                title="Monitoring Dashboard Sedang Dimuat".to_string()
                description="Sistem sedang mengumpulkan data real-time dari berbagai sumber monitoring".to_string()
                icon="fas fa-chart-line".to_string()
            />
        </div>
    }
}

/// PENGAWASAN Reporting Page
#[component]
pub fn PengawasanPelaporan() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SupervisionHeader
                title="Pelaporan Pengawasan".to_string()
                subtitle="Generate dan kelola laporan hasil audit dan monitoring".to_string()
                actions=vec![
                    ("Laporan Baru".to_string(), "create-report".to_string()),
                    ("Template Laporan".to_string(), "report-templates".to_string()),
                ]
            />

            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <StatsCard
                    title="Laporan Aktif".to_string()
                    value="23".to_string()
                    icon="fas fa-file-alt".to_string()
                    color="blue".to_string()
                />
                <StatsCard
                    title="Menunggu Approval".to_string()
                    value="7".to_string()
                    icon="fas fa-hourglass-half".to_string()
                    color="yellow".to_string()
                />
                <StatsCard
                    title="Diterbitkan".to_string()
                    value="156".to_string()
                    icon="fas fa-check-circle".to_string()
                    color="green".to_string()
                />
                <StatsCard
                    title="Overdue".to_string()
                    value="3".to_string()
                    icon="fas fa-exclamation-circle".to_string()
                    color="red".to_string()
                />
            </div>

            <div class="flex items-center space-x-4">
                <SearchInput placeholder="Cari laporan berdasarkan judul, periode, atau jenis...".to_string() />
                <ActionButton
                    label="Filter Periode".to_string()
                    action="filter-period".to_string()
                    variant=ButtonVariant::Secondary
                />
                <ActionButton
                    label="Download Batch".to_string()
                    action="batch-download".to_string()
                    variant=ButtonVariant::Success
                />
            </div>

            <EmptyState
                title="Belum Ada Laporan".to_string()
                description="Buat laporan pengawasan untuk mendokumentasikan hasil audit dan monitoring".to_string()
                icon="fas fa-chart-bar".to_string()
                action=("Generate Laporan".to_string(), "generate-first-report".to_string())
            />
        </div>
    }
}

/// PENGAWASAN Follow-up Actions Page
#[component]
pub fn PengawasanTindakLanjut() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <SupervisionHeader
                title="Tindak Lanjut Temuan".to_string()
                subtitle="Kelola dan monitor tindak lanjut temuan audit".to_string()
                actions=vec![
                    ("Action Plan Baru".to_string(), "create-action-plan".to_string()),
                    ("Bulk Update".to_string(), "bulk-update".to_string()),
                ]
            />

            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                <StatsCard
                    title="Total Tindak Lanjut".to_string()
                    value="89".to_string()
                    icon="fas fa-tasks".to_string()
                    color="blue".to_string()
                />
                <StatsCard
                    title="Dalam Proses".to_string()
                    value="34".to_string()
                    icon="fas fa-spinner".to_string()
                    color="yellow".to_string()
                />
                <StatsCard
                    title="Selesai".to_string()
                    value="48".to_string()
                    icon="fas fa-check-circle".to_string()
                    color="green".to_string()
                />
                <StatsCard
                    title="Overdue".to_string()
                    value="7".to_string()
                    icon="fas fa-clock".to_string()
                    color="red".to_string()
                />
            </div>

            <div class="flex items-center space-x-4">
                <SearchInput placeholder="Cari tindak lanjut berdasarkan temuan atau PIC...".to_string() />
                <ActionButton
                    label="Filter Status".to_string()
                    action="filter-status".to_string()
                    variant=ButtonVariant::Secondary
                />
                <ActionButton
                    label="Reminder".to_string()
                    action="send-reminder".to_string()
                    variant=ButtonVariant::Warning
                />
            </div>

            <Alert
                message="7 tindak lanjut melewati deadline dan memerlukan eskalasi".to_string()
                variant=AlertVariant::Warning
                dismissible=true
            />

            <EmptyState
                title="Tidak Ada Tindak Lanjut Pending".to_string()
                description="Semua tindak lanjut temuan telah diselesaikan atau tidak ada temuan yang memerlukan tindak lanjut".to_string()
                icon="fas fa-check-double".to_string()
            />
        </div>
    }
}
