//! BADIKLAT specific components

use leptos::*;
use simpelv2_shared::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PelatihanItem {
    pub id: String,
    pub judul: String,
    pub deskripsi: String,
    pub tanggal_mulai: String,
    pub tanggal_selesai: String,
    pub instruktur: String,
    pub kuota: u32,
    pub peserta_terdaftar: u32,
    pub status: String,
    pub kategori: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PesertaItem {
    pub id: String,
    pub nama: String,
    pub nip: String,
    pub unit_kerja: String,
    pub email: String,
    pub pelatihan_diikuti: Vec<String>,
    pub sertifikat: Vec<String>,
}

#[component]
pub fn PelatihanCard(
    pelatihan: PelatihanItem,
    #[prop(optional)] on_detail: Option<Callback<String>>,
) -> impl IntoView {
    let status_color = match pelatihan.status.as_str() {
        "Aktif" => "green",
        "Selesai" => "blue",
        "Dibatalkan" => "red",
        _ => "gray",
    };

    let progress = if pelatihan.kuota > 0 {
        (pelatihan.peserta_terdaftar as f64 / pelatihan.kuota as f64 * 100.0) as u32
    } else {
        0
    };

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow duration-200">
            <div class="flex items-start justify-between mb-4">
                <div class="flex-1">
                    <h3 class="text-lg font-semibold text-gray-900 mb-2">{pelatihan.judul}</h3>
                    <p class="text-sm text-gray-600 mb-3 line-clamp-2">{pelatihan.deskripsi}</p>

                    <div class="flex items-center space-x-4 text-sm text-gray-500 mb-3">
                        <div class="flex items-center">
                            <i class="fas fa-calendar mr-2"></i>
                            <span>{pelatihan.tanggal_mulai} " - " {pelatihan.tanggal_selesai}</span>
                        </div>
                        <div class="flex items-center">
                            <i class="fas fa-user-tie mr-2"></i>
                            <span>{pelatihan.instruktur}</span>
                        </div>
                    </div>
                </div>
                <div class={format!("px-3 py-1 bg-{}-100 text-{}-700 text-xs font-medium rounded-full", status_color, status_color)}>
                    {pelatihan.status}
                </div>
            </div>

            <div class="mb-4">
                <div class="flex items-center justify-between text-sm text-gray-600 mb-2">
                    <span>"Peserta Terdaftar"</span>
                    <span>{pelatihan.peserta_terdaftar} "/" {pelatihan.kuota}</span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                    <div
                        class={format!("bg-green-500 h-2 rounded-full transition-all duration-300")}
                        style={format!("width: {}%", progress)}
                    ></div>
                </div>
            </div>

            <div class="flex items-center justify-between">
                <span class="text-xs text-gray-500 bg-gray-100 px-2 py-1 rounded">
                    {pelatihan.kategori}
                </span>
                {if let Some(callback) = on_detail {
                    view! {
                        <button
                            class="btn btn-sm btn-primary"
                            on:click=move |_| callback.run(pelatihan.id.clone())
                        >
                            <i class="fas fa-eye mr-1"></i>
                            "Detail"
                        </button>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>
        </div>
    }
}

#[component]
pub fn PesertaCard(
    peserta: PesertaItem,
    #[prop(optional)] on_detail: Option<Callback<String>>,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 hover:shadow-md transition-shadow duration-200">
            <div class="flex items-start justify-between mb-4">
                <div class="flex items-center space-x-4">
                    <div class="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center">
                        <i class="fas fa-user text-green-600 text-lg"></i>
                    </div>
                    <div>
                        <h3 class="font-semibold text-gray-900">{peserta.nama}</h3>
                        <p class="text-sm text-gray-600">"NIP: " {peserta.nip}</p>
                        <p class="text-sm text-gray-500">{peserta.unit_kerja}</p>
                    </div>
                </div>
            </div>

            <div class="space-y-2 mb-4">
                <div class="flex items-center text-sm text-gray-600">
                    <i class="fas fa-envelope mr-2 w-4"></i>
                    <span>{peserta.email}</span>
                </div>
                <div class="flex items-center text-sm text-gray-600">
                    <i class="fas fa-graduation-cap mr-2 w-4"></i>
                    <span>{peserta.pelatihan_diikuti.len()} " pelatihan diikuti"</span>
                </div>
                <div class="flex items-center text-sm text-gray-600">
                    <i class="fas fa-certificate mr-2 w-4"></i>
                    <span>{peserta.sertifikat.len()} " sertifikat"</span>
                </div>
            </div>

            {if let Some(callback) = on_detail {
                view! {
                    <button
                        class="w-full btn btn-sm btn-outline"
                        on:click=move |_| callback.run(peserta.id.clone())
                    >
                        <i class="fas fa-user mr-2"></i>
                        "Lihat Detail"
                    </button>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

#[component]
pub fn StatistikPelatihan() -> impl IntoView {
    // Mock data - in real app, this would come from API
    let stats = vec![
        ("fa-chalkboard-teacher", "Pelatihan Aktif", "12", Some("green"), None),
        ("fa-users", "Peserta Aktif", "245", Some("blue"), Some(Trend {
            percentage: 15.3,
            direction: TrendDirection::Up,
            period: "vs bulan lalu".to_string(),
        })),
        ("fa-certificate", "Sertifikat Diterbitkan", "186", Some("purple"), Some(Trend {
            percentage: 8.7,
            direction: TrendDirection::Up,
            period: "bulan ini".to_string(),
        })),
        ("fa-user-tie", "Instruktur", "24", Some("orange"), None),
    ];

    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
            <For
                each=move || stats.clone()
                key=|(icon, title, _, _, _)| format!("{}-{}", icon, title)
                children=move |(icon, title, value, color, trend)| {
                    view! {
                        <StatCard
                            icon=icon.to_string()
                            title=title.to_string()
                            value=value.to_string()
                            color=color.map(|s| s.to_string())
                            trend=trend
                            description=Some("Data terkini".to_string())
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
pub fn PelatihanFilter(
    on_filter_change: Callback<(String, String)>, // (kategori, status)
) -> impl IntoView {
    let (selected_kategori, set_kategori) = create_signal("semua".to_string());
    let (selected_status, set_status) = create_signal("semua".to_string());

    let kategoris = vec![
        ("semua", "Semua Kategori"),
        ("teknis", "Teknis"),
        ("kepemimpinan", "Kepemimpinan"),
        ("administrasi", "Administrasi"),
        ("keuangan", "Keuangan"),
    ];

    let status_list = vec![
        ("semua", "Semua Status"),
        ("aktif", "Aktif"),
        ("selesai", "Selesai"),
        ("dibatalkan", "Dibatalkan"),
    ];

    let handle_filter_change = move |_| {
    on_filter_change.run((selected_kategori.get(), selected_status.get()));
    };

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-200 p-6 mb-6">
            <h3 class="font-semibold text-gray-900 mb-4">
                <i class="fas fa-filter mr-2"></i>
                "Filter Pelatihan"
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">"Kategori"</label>
                    <select
                        class="simpelv2-form-input"
                        on:change=move |e| {
                            set_kategori.set(event_target_value(&e));
                            handle_filter_change(());
                        }
                    >
                        <For
                            each=move || kategoris.clone()
                            key=|(value, _)| value.to_string()
                            children=move |(value, label)| {
                                view! {
                                    <option value=value selected=move || selected_kategori.get() == value>
                                        {label}
                                    </option>
                                }
                            }
                        />
                    </select>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">"Status"</label>
                    <select
                        class="simpelv2-form-input"
                        on:change=move |e| {
                            set_status.set(event_target_value(&e));
                            handle_filter_change(());
                        }
                    >
                        <For
                            each=move || status_list.clone()
                            key=|(value, _)| value.to_string()
                            children=move |(value, label)| {
                                view! {
                                    <option value=value selected=move || selected_status.get() == value>
                                        {label}
                                    </option>
                                }
                            }
                        />
                    </select>
                </div>

                <div class="flex items-end">
                    <button
                        class="btn btn-primary w-full"
                        on:click=move |_| {
                            set_kategori.set("semua".to_string());
                            set_status.set("semua".to_string());
                            handle_filter_change(());
                        }
                    >
                        <i class="fas fa-undo mr-2"></i>
                        "Reset Filter"
                    </button>
                </div>
            </div>
        </div>
    }
}
