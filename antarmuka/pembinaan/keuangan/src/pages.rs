use crate::components::{footer::Footer, header::Header};
use crate::api::{fetch_metrics, fetch_budgets, fetch_transactions};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// Re-using types matching backend (and previously defined, but now we sync with API)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinancialMetrics {
    pub total_budget: f64,
    pub total_realization: f64,
    pub utilization_percentage: f64,
    pub pending_transactions: i64,
    pub approved_transactions: i64,
    pub monthly_variance: f64,
    pub cash_flow: f64,
    pub budget_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetItem {
    pub id: uuid::Uuid,
    pub category: String,
    pub subcategory: String,
    pub budget_code: String,
    pub allocated_amount: f64,
    pub realized_amount: f64,
    pub remaining_amount: f64,
    pub priority: String,
    pub status: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub responsible_unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: uuid::Uuid,
    pub transaction_code: String,
    pub description: String,
    pub amount: f64,
    pub transaction_type: String,
    pub status: String,
    pub budget_item_id: Option<uuid::Uuid>,
    pub transaction_date: DateTime<Utc>,
    pub approval_date: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
    pub supporting_documents: Option<Vec<String>>,
}

// Enums for UI Logic (Mapped from String in backend for now, or just keep string)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReportStatus {
    Draft,
    InReview,
    Approved,
    Published,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetStatus {
    Planned,
    Active,
    OnHold,
    Completed,
    Cancelled,
}

// Helper to convert string to enum for display if needed, or just use strings
impl BudgetPriority {
    fn from_str(s: &str) -> Self {
        match s {
            "Low" => Self::Low,
            "Medium" => Self::Medium,
            "High" => Self::High,
            "Critical" => Self::Critical,
            _ => Self::Medium,
        }
    }
}

// UI Components for Financial Management
#[component]
pub fn FinancialHeader(title: String, subtitle: String) -> impl IntoView {
    view! {
        <div class="bg-gradient-to-r from-green-600 to-green-800 text-white p-8 rounded-xl shadow-lg mb-8">
            <h1 class="text-3xl font-bold mb-2">{title}</h1>
            <p class="text-green-100 text-lg">{subtitle}</p>
        </div>
    }
}

#[component]
pub fn MetricsCard(
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
pub fn ReportStatusBadge(status: ReportStatus) -> impl IntoView {
    let (text, color) = match status {
        ReportStatus::Draft => ("Draft", "bg-gray-100 text-gray-800"),
        ReportStatus::InReview => ("In Review", "bg-yellow-100 text-yellow-800"),
        ReportStatus::Approved => ("Approved", "bg-green-100 text-green-800"),
        ReportStatus::Published => ("Published", "bg-blue-100 text-blue-800"),
        ReportStatus::Archived => ("Archived", "bg-gray-100 text-gray-600"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

#[component]
pub fn BudgetPriorityBadge(priority: String) -> impl IntoView {
    let (text, color) = match priority.as_str() {
        "Low" => ("Low", "bg-gray-100 text-gray-800"),
        "Medium" => ("Medium", "bg-yellow-100 text-yellow-800"),
        "High" => ("High", "bg-orange-100 text-orange-800"),
        "Critical" => ("Critical", "bg-red-100 text-red-800"),
        _ => ("Medium", "bg-yellow-100 text-yellow-800"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

#[component]
pub fn UtilizationBar(progress: f64) -> impl IntoView {
    let color = if progress >= 90.0 {
        "bg-red-600"
    } else if progress >= 75.0 {
        "bg-yellow-600"
    } else {
        "bg-green-600"
    };

    view! {
        <div class="w-full bg-gray-200 rounded-full h-2">
            <div
                class={format!("{color} h-2 rounded-full transition-all duration-300")}
                style={format!("width: {progress}%")}
            ></div>
        </div>
    }
}

#[component]
pub fn FinancialSearchInput(placeholder: String) -> impl IntoView {
    view! {
        <div class="relative">
            <input
                type="text"
                class="w-full pl-10 pr-4 py-3 rounded-lg border border-gray-300 focus:outline-none focus:ring-2 focus:ring-green-500 focus:border-transparent"
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
pub fn AmountDisplay(amount: f64) -> impl IntoView {
    let formatted = format!("Rp {:.1}M", amount / 1_000_000.0);

    view! {
        <span class="text-lg font-semibold text-gray-900">{formatted}</span>
    }
}

#[component]
pub fn ActionButton(label: String, action: String) -> impl IntoView {
    view! {
        <button
            class="bg-green-600 hover:bg-green-700 text-white px-6 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl transition-all duration-200"
            data-action=action
        >
            {label}
        </button>
    }
}

// Financial Dashboard Implementation
#[component]
pub fn KeuanganDashboard() -> impl IntoView {
    // Fetch resources
    let metrics_resource = Resource::new(|| (), |_| fetch_metrics());
    let budget_resource = Resource::new(|| (), |_| fetch_budgets());
    let transactions_resource = Resource::new(|| (), |_| fetch_transactions());

    view! {
        <div class="min-h-screen bg-gray-50">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <FinancialHeader
                    title="Dashboard Keuangan".to_string()
                    subtitle="Sistem Manajemen Keuangan - Kejaksaan Republik Indonesia".to_string()
                />

                // Key Financial Metrics
                <Suspense fallback=move || view! { <div class="p-8 text-center">"Loading Metrics..."</div> }>
                    {move || {
                        metrics_resource.get().map(|result| match result {
                            Ok(metrics) => view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
                                    <MetricsCard
                                        title="Total Anggaran".to_string()
                                        value=format!("Rp {:.1}M", metrics.total_budget / 1_000_000.0)
                                        change="+0.0%".to_string()
                                        icon="💰".to_string()
                                        positive=true
                                    />
                                    <MetricsCard
                                        title="Total Realisasi".to_string()
                                        value=format!("Rp {:.1}M", metrics.total_realization / 1_000_000.0)
                                        change="+0.0%".to_string()
                                        icon="📊".to_string()
                                        positive=true
                                    />
                                    <MetricsCard
                                        title="Utilisasi".to_string()
                                        value=format!("{:.1}%", metrics.utilization_percentage)
                                        change=format!("{:.1}%", metrics.monthly_variance)
                                        icon="📈".to_string()
                                        positive=metrics.monthly_variance > 0.0
                                    />
                                    <MetricsCard
                                        title="Saldo Kas".to_string()
                                        value=format!("Rp {:.1}M", metrics.cash_flow / 1_000_000.0)
                                        change="+0.0%".to_string()
                                        icon="🏦".to_string()
                                        positive=true
                                    />
                                </div>
                            }.into_any(),
                            Err(e) => view! { <div class="text-red-500">{format!("Error loading metrics: {}", e)}</div> }.into_any(),
                        })
                    }}
                </Suspense>

                // Budget Management Section
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
                    <div class="bg-white rounded-xl shadow-lg p-6">
                        <div class="flex items-center justify-between mb-6">
                            <h2 class="text-xl font-bold text-gray-900">"Budget Overview"</h2>
                            <ActionButton
                                label="Kelola Anggaran".to_string()
                                action="manage-budget".to_string()
                            />
                        </div>

                        <div class="space-y-4">
                            <Suspense fallback=move || view! { <div>"Loading Budgets..."</div> }>
                                {move || {
                                    budget_resource.get().map(|result| match result {
                                        Ok(items) => items.into_iter().map(|item| view! {
                                            <div class="border border-gray-200 rounded-lg p-4">
                                                <div class="flex items-center justify-between mb-2">
                                                    <h3 class="font-semibold text-gray-900">{item.category}</h3>
                                                    <BudgetPriorityBadge priority=item.priority />
                                                </div>
                                                <div class="flex items-center justify-between mb-2">
                                                    <span class="text-sm text-gray-600">{item.subcategory}</span>
                                                    <AmountDisplay amount=item.allocated_amount />
                                                </div>
                                                <UtilizationBar progress={if item.allocated_amount > 0.0 { item.realized_amount / item.allocated_amount * 100.0 } else { 0.0 }} />
                                                <div class="flex justify-between text-xs text-gray-500 mt-1">
                                                    <span>"Realisasi: "
                                                        <AmountDisplay amount=item.realized_amount />
                                                    </span>
                                                    <span>{format!("{:.1}%", if item.allocated_amount > 0.0 { item.realized_amount / item.allocated_amount * 100.0 } else { 0.0 })}</span>
                                                </div>
                                            </div>
                                        }).collect_view().into_any(),
                                        Err(e) => view! { <div class="text-red-500">{format!("Error: {}", e)}</div> }.into_any()
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>

                    <div class="bg-white rounded-xl shadow-lg p-6">
                        <div class="flex items-center justify-between mb-6">
                            <h2 class="text-xl font-bold text-gray-900">"Transaksi Terbaru"</h2>
                            <ActionButton
                                label="Input Transaksi".to_string()
                                action="new-transaction".to_string()
                            />
                        </div>

                        <div class="space-y-4">
                             <Suspense fallback=move || view! { <div>"Loading Transactions..."</div> }>
                                {move || {
                                    transactions_resource.get().map(|result| match result {
                                        Ok(txs) => txs.into_iter().map(|tx| view! {
                                            <div class="border border-gray-200 rounded-lg p-4">
                                                <div class="flex items-center justify-between mb-2">
                                                    <h3 class="font-semibold text-gray-900">{tx.description}</h3>
                                                    <span class="text-sm bg-gray-100 text-gray-800 px-2 py-1 rounded">{tx.status}</span>
                                                </div>
                                                <div class="flex items-center justify-between">
                                                    <span class="text-sm text-gray-600">{tx.transaction_date.format("%d %b %Y").to_string()}</span>
                                                    <span class={format!("text-lg font-semibold {}", if tx.transaction_type == "Income" { "text-green-600" } else { "text-red-600" })}>
                                                        {format!("{}Rp {:.1}M", if tx.transaction_type == "Income" { "+" } else { "-" }, tx.amount / 1_000_000.0)}
                                                    </span>
                                                </div>
                                            </div>
                                        }).collect_view().into_any(),
                                        Err(e) => view! { <div class="text-red-500">{format!("Error: {}", e)}</div> }.into_any()
                                    })
                                }}
                            </Suspense>
                        </div>
                    </div>
                </div>

                // Financial Reports Section (Static for now)
                <div class="bg-white rounded-xl shadow-lg p-6">
                    <div class="flex items-center justify-between mb-6">
                        <h2 class="text-xl font-bold text-gray-900">"Laporan Keuangan"</h2>
                        <ActionButton
                            label="Generate Laporan".to_string()
                            action="generate-report".to_string()
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                        <div class="bg-gray-50 rounded-lg p-4">
                            <h3 class="font-semibold text-gray-900 mb-2">"Laporan Bulanan"</h3>
                            <p class="text-sm text-gray-600 mb-4">"Januari 2024"</p>
                            <ReportStatusBadge status=ReportStatus::Published />
                        </div>

                        <div class="bg-gray-50 rounded-lg p-4">
                            <h3 class="font-semibold text-gray-900 mb-2">"Laporan Triwulan"</h3>
                            <p class="text-sm text-gray-600 mb-4">"Q4 2023"</p>
                            <ReportStatusBadge status=ReportStatus::Approved />
                        </div>

                        <div class="bg-gray-50 rounded-lg p-4">
                            <h3 class="font-semibold text-gray-900 mb-2">"Laporan Tahunan"</h3>
                            <p class="text-sm text-gray-600 mb-4">"2023"</p>
                            <ReportStatusBadge status=ReportStatus::InReview />
                        </div>
                    </div>
                </div>
            </main>

            <Footer />
        </div>
    }
}

// Budget Management Page
#[component]
pub fn KeuanganAnggaran() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <FinancialHeader
                    title="Manajemen Anggaran".to_string()
                    subtitle="Kelola dan Monitor Anggaran Keuangan".to_string()
                />

                <div class="flex items-center justify-between mb-6">
                    <FinancialSearchInput placeholder="Cari anggaran...".to_string() />
                    <ActionButton
                        label="Tambah Anggaran".to_string()
                        action="add-budget".to_string()
                    />
                </div>

                <div class="bg-white rounded-xl shadow-lg p-6">
                    <h2 class="text-xl font-bold text-gray-900 mb-6">"Daftar Anggaran"</h2>
                    <p class="text-gray-600">"Halaman ini dapat dikembangkan lebih lanjut untuk menampilkan tabel lengkap daftar anggaran dengan fitur edit/delete."</p>
                </div>
            </main>

            <Footer />
        </div>
    }
}

// Budget Realization Page
#[component]
pub fn KeuanganRealisasi() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <FinancialHeader
                    title="Realisasi Anggaran".to_string()
                    subtitle="Monitor Realisasi dan Penggunaan Anggaran".to_string()
                />

                <div class="flex items-center justify-between mb-6">
                    <FinancialSearchInput placeholder="Cari realisasi...".to_string() />
                    <ActionButton
                        label="Input Realisasi".to_string()
                        action="add-realization".to_string()
                    />
                </div>

                <div class="bg-white rounded-xl shadow-lg p-6">
                    <h2 class="text-xl font-bold text-gray-900 mb-6">"Realisasi Anggaran"</h2>
                    <p class="text-gray-600">"Halaman ini dapat dikembangkan lebih lanjut."</p>
                </div>
            </main>

            <Footer />
        </div>
    }
}

// Financial Reports Page
#[component]
pub fn KeuanganLaporan() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <FinancialHeader
                    title="Laporan Keuangan".to_string()
                    subtitle="Generate dan Kelola Laporan Keuangan".to_string()
                />

                <div class="flex items-center justify-between mb-6">
                    <FinancialSearchInput placeholder="Cari laporan...".to_string() />
                    <ActionButton
                        label="Generate Laporan".to_string()
                        action="generate-report".to_string()
                    />
                </div>

                <div class="bg-white rounded-xl shadow-lg p-6">
                    <h2 class="text-xl font-bold text-gray-900 mb-6">"Laporan Keuangan"</h2>
                    <p class="text-gray-600">"Halaman ini dapat dikembangkan lebih lanjut."</p>
                </div>
            </main>

            <Footer />
        </div>
    }
}

// Financial Audit Page
#[component]
pub fn KeuanganAudit() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <Header />

            <main class="container mx-auto px-4 py-8">
                <FinancialHeader
                    title="Audit Keuangan".to_string()
                    subtitle="Audit dan Verifikasi Transaksi Keuangan".to_string()
                />

                <div class="flex items-center justify-between mb-6">
                    <FinancialSearchInput placeholder="Cari audit...".to_string() />
                    <ActionButton
                        label="Audit Baru".to_string()
                        action="new-audit".to_string()
                    />
                </div>

                <div class="bg-white rounded-xl shadow-lg p-6">
                    <h2 class="text-xl font-bold text-gray-900 mb-6">"Audit Keuangan"</h2>
                    <p class="text-gray-600">"Halaman ini dapat dikembangkan lebih lanjut."</p>
                </div>
            </main>

            <Footer />
        </div>
    }
}
