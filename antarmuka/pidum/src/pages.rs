use crate::types::*;
use chrono::Utc;
use leptos::prelude::*;

/// Header khusus PIDUM dengan tema biru
#[component]
fn GeneralHeader() -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-blue-600 to-blue-800 text-white shadow-lg">
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-4">
                        <div class="bg-white bg-opacity-20 p-2 rounded-lg">
                            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 20 20">
                                <path d="M3 3a1 1 0 011-1h12a1 1 0 011 1v3a1 1 0 01-.293.707L12 11.414V15a1 1 0 01-.293.707l-2 2A1 1 0 018 17v-5.586L3.293 6.707A1 1 0 013 6V3z"/>
                            </svg>
                        </div>
                        <div>
                            <h1 class="text-2xl font-bold">PIDUM</h1>
                            <p class="text-blue-100">Penyidikan Pidana Umum</p>
                        </div>
                    </div>
                    <div class="text-right">
                        <p class="text-sm text-blue-100">Kejaksaan Republik Indonesia</p>
                        <p class="text-xs text-blue-200">Sistem Terintegrasi Penyidikan</p>
                    </div>
                </div>
            </div>
        </header>
    }
}

/// Badge untuk status
#[component]
fn StatusBadge(status: GeneralCaseStatus) -> impl IntoView {
    let (color, text) = match status {
        GeneralCaseStatus::Investigation => ("bg-yellow-100 text-yellow-800", "Penyelidikan"),
        GeneralCaseStatus::Prosecution => ("bg-blue-100 text-blue-800", "Penuntutan"),
        GeneralCaseStatus::Trial => ("bg-purple-100 text-purple-800", "Persidangan"),
        GeneralCaseStatus::Verdict => ("bg-indigo-100 text-indigo-800", "Putusan"),
        GeneralCaseStatus::Appeal => ("bg-orange-100 text-orange-800", "Banding"),
        GeneralCaseStatus::Cassation => ("bg-red-100 text-red-800", "Kasasi"),
        GeneralCaseStatus::Execution => ("bg-green-100 text-green-800", "Eksekusi"),
        GeneralCaseStatus::Closed => ("bg-gray-200 text-gray-600", "Ditutup"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

/// Badge untuk prioritas
#[component]
fn PriorityBadge(priority: CasePriority) -> impl IntoView {
    let (color, text) = match priority {
        CasePriority::Urgent => ("bg-red-100 text-red-800", "Darurat"),
        CasePriority::High => ("bg-orange-100 text-orange-800", "Tinggi"),
        CasePriority::Medium => ("bg-yellow-100 text-yellow-800", "Sedang"),
        CasePriority::Low => ("bg-green-100 text-green-800", "Rendah"),
    };

    view! {
        <span class={format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {color}")}>
            {text}
        </span>
    }
}

/// Card statistik untuk PIDUM
#[component]
fn GeneralStatCard(
    title: String,
    value: String,
    subtitle: String,
    icon_color: String,
) -> impl IntoView {
    view! {
        <div class="bg-white p-6 rounded-lg shadow-md border-l-4 border-blue-500">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-600">{title}</p>
                    <p class="text-3xl font-bold text-blue-600">{value}</p>
                    <p class="text-xs text-gray-500 mt-1">{subtitle}</p>
                </div>
                <div class={format!("p-3 rounded-full {icon_color}")}>
                    <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z"/>
                    </svg>
                </div>
            </div>
        </div>
    }
}

/// Input pencarian untuk PIDUM
#[component]
fn GeneralSearchInput() -> impl IntoView {
    let (search_query, set_search_query) = signal(String::new());

    view! {
        <div class="relative">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="h-5 w-5 text-gray-400" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd"/>
                </svg>
            </div>
            <input
                type="text"
                class="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="Cari perkara, tersangka, atau nomor perkara..."
                on:input=move |ev| {
                    set_search_query.set(event_target_value(&ev));
                }
                prop:value=search_query
            />
        </div>
    }
}

/// Progress bar untuk tracking
#[component]
fn GeneralProgressBar(label: String, percentage: f64, color: String) -> impl IntoView {
    view! {
        <div class="mb-4">
            <div class="flex justify-between text-sm font-medium text-gray-700 mb-1">
                <span>{label}</span>
                <span>{format!("{percentage:.1}%")}</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
                <div
                    class={format!("h-2 rounded-full {color}")}
                    style={format!("width: {percentage:.1}%")}
                ></div>
            </div>
        </div>
    }
}

/// PIDUM Dashboard Page
#[component]
pub fn PidumDashboard() -> impl IntoView {
    // Mock data untuk statistik
    let stats = signal(PidumStatistics {
        total_cases: 1247,
        investigation_cases: 346,
        prosecution_cases: 189,
        trial_cases: 92,
        completed_cases: 620,
        monthly_new_cases: 73,
        case_resolution_rate: 76.4,
        average_processing_time: 142,
    });

    // Mock data kasus aktif
    let active_cases = signal(vec![
        GeneralCase {
            case_number: "REG-2024-001".to_string(),
            crime_type: GeneralCrimeType::Fraud,
            status: GeneralCaseStatus::Investigation,
            priority: CasePriority::High,
            defendant_count: 2,
            witness_count: 5,
            evidence_count: 12,
            prosecutor: "Jaksa Agung".to_string(),
            investigator: "Polda Metro".to_string(),
            registered_date: Utc::now(),
            estimated_trial_date: None,
            court: "PN Jakarta Pusat".to_string(),
            case_summary: "Penipuan berkedok investasi dengan kerugian Rp 2.5 miliar".to_string(),
            total_loss: Some(2_500_000_000.0),
        },
        GeneralCase {
            case_number: "REG-2024-002".to_string(),
            crime_type: GeneralCrimeType::Theft,
            status: GeneralCaseStatus::Prosecution,
            priority: CasePriority::Medium,
            defendant_count: 1,
            witness_count: 3,
            evidence_count: 8,
            prosecutor: "Jaksa Adi".to_string(),
            investigator: "Polsek Menteng".to_string(),
            registered_date: Utc::now(),
            estimated_trial_date: None,
            court: "PN Jakarta Pusat".to_string(),
            case_summary: "Pencurian kendaraan bermotor".to_string(),
            total_loss: Some(150_000_000.0),
        },
        GeneralCase {
            case_number: "REG-2024-003".to_string(),
            crime_type: GeneralCrimeType::Domestic,
            status: GeneralCaseStatus::Trial,
            priority: CasePriority::High,
            defendant_count: 1,
            witness_count: 2,
            evidence_count: 6,
            prosecutor: "Jaksa Budi".to_string(),
            investigator: "Polsek Kelapa Gading".to_string(),
            registered_date: Utc::now(),
            estimated_trial_date: None,
            court: "PN Jakarta Utara".to_string(),
            case_summary: "Kekerasan dalam rumah tangga".to_string(),
            total_loss: None,
        },
    ]);

    view! {
        <div class="space-y-8">
            <GeneralHeader />

            // Statistics Overview
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <GeneralStatCard
                    title="Total Perkara".to_string()
                    value=stats.0.with(|s| s.total_cases.to_string())
                    subtitle="Semua perkara aktif".to_string()
                    icon_color="bg-blue-500".to_string()
                />
                <GeneralStatCard
                    title="Dalam Penyidikan".to_string()
                    value=stats.0.with(|s| s.investigation_cases.to_string())
                    subtitle="Sedang diselidiki".to_string()
                    icon_color="bg-orange-500".to_string()
                />
                <GeneralStatCard
                    title="Dalam Penuntutan".to_string()
                    value=stats.0.with(|s| s.prosecution_cases.to_string())
                    subtitle="Tahap penuntutan".to_string()
                    icon_color="bg-purple-500".to_string()
                />
                <GeneralStatCard
                    title="Rata-rata Proses".to_string()
                    value=stats.0.with(|s| format!("{} hari", s.average_processing_time))
                    subtitle="Waktu penyelesaian".to_string()
                    icon_color="bg-green-500".to_string()
                />
            </div>

            // Search and Actions
            <div class="bg-white p-6 rounded-lg shadow-md">
                <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between space-y-4 sm:space-y-0">
                    <div class="flex-1 max-w-lg">
                        <GeneralSearchInput />
                    </div>
                    <div class="flex space-x-3">
                        <button class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Perkara Baru
                        </button>
                        <button class="bg-gray-600 hover:bg-gray-700 text-white px-4 py-2 rounded-md text-sm font-medium">
                            Laporan
                        </button>
                    </div>
                </div>
            </div>

            // Progress Overview
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="bg-white p-6 rounded-lg shadow-md">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">Progress Perkara</h3>
                    <GeneralProgressBar
                        label="Tingkat Penyelesaian".to_string()
                        percentage=stats.0.with(|s| s.case_resolution_rate)
                        color="bg-green-500".to_string()
                    />
                    <GeneralProgressBar
                        label="Penyidikan Selesai".to_string()
                        percentage=72.3
                        color="bg-blue-500".to_string()
                    />
                    <GeneralProgressBar
                        label="Penuntutan Selesai".to_string()
                        percentage=68.7
                        color="bg-purple-500".to_string()
                    />
                </div>

                <div class="bg-white p-6 rounded-lg shadow-md">
                    <h3 class="text-lg font-semibold text-gray-900 mb-4">Distribusi Jenis Perkara</h3>
                    <div class="space-y-3">
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-600">Penipuan</span>
                            <span class="font-medium text-gray-900">342 kasus (27%)</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-600">Pencurian</span>
                            <span class="font-medium text-gray-900">298 kasus (24%)</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-600">Lalu Lintas</span>
                            <span class="font-medium text-gray-900">187 kasus (15%)</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-600">KDRT</span>
                            <span class="font-medium text-gray-900">156 kasus (13%)</span>
                        </div>
                        <div class="flex justify-between text-sm">
                            <span class="text-gray-600">Lainnya</span>
                            <span class="font-medium text-gray-900">264 kasus (21%)</span>
                        </div>
                    </div>
                </div>
            </div>

            // Active Cases Table
            <div class="bg-white shadow-md rounded-lg overflow-hidden">
                <div class="px-6 py-4 border-b border-gray-200">
                    <h3 class="text-lg font-semibold text-gray-900">Perkara Prioritas Tinggi</h3>
                </div>
                <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Nomor Perkara
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Jenis Pidana
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Status
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Prioritas
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Jaksa/Penyidik
                                </th>
                                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Kerugian
                                </th>
                                <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                    Aksi
                                </th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                                                        {move || {
                                let cases = active_cases.0.get();
                                cases.iter().cloned().map(|case| {
                                    let loss_str = case.total_loss
                                        .map(|loss| {
                                            if loss >= 1_000_000_000.0 {
                                                format!("Rp {:.1} M", loss / 1_000_000_000.0)
                                            } else if loss >= 1_000_000.0 {
                                                format!("Rp {:.1} jt", loss / 1_000_000.0)
                                            } else {
                                                format!("Rp {:.0} rb", loss / 1_000.0)
                                            }
                                        })
                                        .unwrap_or_else(|| "Tidak diketahui".to_string());

                                    let crime_type_text = match case.crime_type {
                                        GeneralCrimeType::Fraud => "Penipuan",
                                        GeneralCrimeType::Theft => "Pencurian",
                                        GeneralCrimeType::Embezzlement => "Penggelapan",
                                        GeneralCrimeType::Assault => "Penganiayaan",
                                        GeneralCrimeType::DrugPossession => "Narkotika",
                                        GeneralCrimeType::Traffic => "Lalu Lintas",
                                        GeneralCrimeType::Domestic => "KDRT",
                                        GeneralCrimeType::Property => "Properti",
                                        GeneralCrimeType::Commercial => "Komersial",
                                        GeneralCrimeType::Other => "Lainnya",
                                    };

                                    view! {
                                        <tr class="hover:bg-gray-50">
                                            <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{case.case_number.clone()}</td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{crime_type_text}</td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <StatusBadge status={case.status} />
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap">
                                                <PriorityBadge priority={case.priority} />
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                                                <div class="flex flex-col">
                                                    <div>{case.defendant_count.to_string()}</div>
                                                    <div class="text-xs text-gray-500">{loss_str}</div>
                                                </div>
                                            </td>
                                            <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{case.prosecutor.clone()}</td>
                                        </tr>
                                    }
                                }).collect::<Vec<_>>()
                            }}
                        </tbody>
                    </table>
                </div>
            </div>

            // Quick Actions Footer
            <div class="bg-white p-6 rounded-lg shadow-md">
                <h3 class="text-lg font-semibold text-gray-900 mb-4">Aksi Cepat</h3>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <button class="flex flex-col items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
                        <div class="bg-blue-100 p-2 rounded-full mb-2">
                            <svg class="w-6 h-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6"/>
                            </svg>
                        </div>
                        <span class="text-sm font-medium">Buat Perkara</span>
                    </button>
                    <button class="flex flex-col items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
                        <div class="bg-green-100 p-2 rounded-full mb-2">
                            <svg class="w-6 h-6 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4M7.835 4.697a3.42 3.42 0 001.946-.806 3.42 3.42 0 014.438 0 3.42 3.42 0 001.946.806 3.42 3.42 0 013.138 3.138 3.42 3.42 0 00.806 1.946 3.42 3.42 0 010 4.438 3.42 3.42 0 00-.806 1.946 3.42 3.42 0 01-3.138 3.138 3.42 3.42 0 00-1.946.806 3.42 3.42 0 01-4.438 0 3.42 3.42 0 00-1.946-.806 3.42 3.42 0 01-3.138-3.138 3.42 3.42 0 00-.806-1.946 3.42 3.42 0 010-4.438 3.42 3.42 0 00.806-1.946 3.42 3.42 0 013.138-3.138z"/>
                            </svg>
                        </div>
                        <span class="text-sm font-medium">Validasi BAP</span>
                    </button>
                    <button class="flex flex-col items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
                        <div class="bg-purple-100 p-2 rounded-full mb-2">
                            <svg class="w-6 h-6 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                            </svg>
                        </div>
                        <span class="text-sm font-medium">Buat Dakwaan</span>
                    </button>
                    <button class="flex flex-col items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50">
                        <div class="bg-orange-100 p-2 rounded-full mb-2">
                            <svg class="w-6 h-6 text-orange-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                            </svg>
                        </div>
                        <span class="text-sm font-medium">Laporan</span>
                    </button>
                </div>
            </div>
        </div>
    }
}
