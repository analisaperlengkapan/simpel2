//! Pemakaian BMN list — lifecycle-aware permit browser.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_COUNTER_CLOCKWISE, CARET_LEFT, CARET_RIGHT, EYE, MAGNIFYING_GLASS, PLUS};
use leptos::task::spawn_local;
use leptos_router::components::A;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent};

use crate::api::error::AppError;
use crate::api::pemakaian_bmn::{self, IzinPemakaianBmn};
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::path;

const PER_PAGE: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LifecycleFilter {
    Semua,
    Draft,
    Diajukan,
    Aktif,
    AkanExpire,
    Kadaluarsa,
    Dicabut,
}

impl LifecycleFilter {
    fn value(self) -> &'static str {
        match self {
            Self::Semua => "",
            Self::Draft => "DRAFT",
            Self::Diajukan => "SUBMITTED",
            Self::Aktif => "ACTIVE",
            Self::AkanExpire => "EXPIRING_SOON",
            Self::Kadaluarsa => "EXPIRED",
            Self::Dicabut => "REVOKED",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Semua => "Semua",
            Self::Draft => "Draft",
            Self::Diajukan => "Diajukan",
            Self::Aktif => "Aktif",
            Self::AkanExpire => "Akan Expire ≤7 hari",
            Self::Kadaluarsa => "Kadaluarsa",
            Self::Dicabut => "Dicabut",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "DRAFT" => Self::Draft,
            "SUBMITTED" => Self::Diajukan,
            "ACTIVE" => Self::Aktif,
            "EXPIRING_SOON" => Self::AkanExpire,
            "EXPIRED" => Self::Kadaluarsa,
            "REVOKED" => Self::Dicabut,
            _ => Self::Semua,
        }
    }
    fn all() -> &'static [LifecycleFilter] {
        &[
            Self::Semua,
            Self::Draft,
            Self::Diajukan,
            Self::Aktif,
            Self::AkanExpire,
            Self::Kadaluarsa,
            Self::Dicabut,
        ]
    }
}

#[component]
pub fn PemakaianBmnListPage() -> impl IntoView {
    let (items, set_items) = signal::<Vec<IzinPemakaianBmn>>(Vec::new());
    let (total, set_total) = signal::<i64>(0);
    let (page, set_page) = signal(1_i64);
    let (total_pages, set_total_pages) = signal(1_i64);
    let (lifecycle, set_lifecycle) = signal(LifecycleFilter::Semua);
    let (search, set_search) = signal::<String>(String::new());
    let (jenis, set_jenis) = signal::<String>(String::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let pg = page.get() as i32;
        let life = lifecycle.get();
        let search_val = search.get();
        let jenis_val = jenis.get();
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let (status_filter, expiring_only) = match life {
                LifecycleFilter::Semua => (None, false),
                LifecycleFilter::AkanExpire => (Some("ACTIVE".to_string()), true),
                other => (Some(other.value().to_string()), false),
            };
            let search_opt = non_empty(&search_val);
            let jenis_opt = non_empty(&jenis_val);

            match pemakaian_bmn::fetch_pemakaian_bmn_list(
                pg,
                PER_PAGE,
                status_filter,
                jenis_opt,
                None,
                None,
                search_opt,
            )
            .await
            {
                Ok(resp) => {
                    let filtered = if expiring_only {
                        resp.data
                            .into_iter()
                            .filter(|item| {
                                days_until(&item.tanggal_selesai)
                                    .is_some_and(|d| (0..=7).contains(&d))
                            })
                            .collect::<Vec<_>>()
                    } else {
                        resp.data
                    };
                    set_items.set(filtered);
                    set_total.set(resp.total);
                    set_total_pages.set(resp.total_pages.into());
                }
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let on_lifecycle = move |val: LifecycleFilter| {
        set_lifecycle.set(val);
        set_page.set(1);
    };

    let on_search_input = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_search.set(target.value());
        }
    };

    let on_search_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let on_jenis = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        {
            set_jenis.set(target.value());
            set_page.set(1);
            set_reload_tick.update(|t| *t += 1);
        }
    };

    let reset_filters = move |_| {
        set_lifecycle.set(LifecycleFilter::Semua);
        set_search.set(String::new());
        set_jenis.set(String::new());
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let prev_page = move |_| {
        let cur = page.get();
        if cur > 1 {
            set_page.set(cur - 1);
        }
    };
    let next_page = move |_| {
        let cur = page.get();
        if cur < total_pages.get() {
            set_page.set(cur + 1);
        }
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::leaf("Pemakaian BMN"),
    ];

    view! {
        <PageLayout
            title="Izin Pemakaian BMN"
            description="Kelola permohonan dan siklus hidup izin pemakaian BMN."
            icon="fas fa-handshake"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || view! {
                <A
                    href=path::PENGELOLAAN_PEMAKAIAN_BUAT
                    attr:class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                >
                    <span class="text-[0.7rem]"><AppIcon icon=PLUS /></span>
                    "Ajukan Izin Baru"
                </A>
            }.into_any())
        >
            <SectionCard
                title="Filter Lifecycle"
                description="Pilih fase izin untuk mempersempit daftar."
                icon="fas fa-filter"
            >
                <div class="flex flex-wrap gap-2">
                    {LifecycleFilter::all().iter().copied().map(|f| {
                        let is_active = move || lifecycle.get() == f;
                        view! {
                            <button
                                type="button"
                                on:click=move |_| on_lifecycle(f)
                                class=move || {
                                    let base = "focus-ring inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-semibold transition";
                                    if is_active() {
                                        format!("{base} bg-gold-gradient text-navy-950 shadow-sm")
                                    } else {
                                        format!("{base} border border-white/10 bg-white/[0.04] text-slate-200 hover:bg-white/[0.08]")
                                    }
                                }
                            >
                                {f.label()}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <form class="mt-4 grid gap-3 md:grid-cols-[1fr_1fr_auto]" on:submit=on_search_submit>
                    <input
                        type="text"
                        placeholder="Cari nomor izin, pemohon, NUP, BMN..."
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        on:input=on_search_input
                        prop:value=move || search.get()
                    />
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_jenis
                        prop:value=move || jenis.get()
                    >
                        <option value="">"Semua Jenis BMN"</option>
                        <option value="KENDARAAN_BERMOTOR">"Kendaraan Bermotor"</option>
                        <option value="RUMAH_NEGARA">"Rumah Negara"</option>
                        <option value="LAPTOP">"Laptop / Komputer"</option>
                        <option value="LAINNYA">"Lainnya"</option>
                    </select>
                    <div class="flex gap-2">
                        <button
                            type="submit"
                            class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                        >
                            <span class="text-xs"><AppIcon icon=MAGNIFYING_GLASS /></span>
                            "Cari"
                        </button>
                        <button
                            type="button"
                            on:click=reset_filters
                            class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200 transition hover:bg-white/[0.08]"
                        >
                            <span class="text-xs"><AppIcon icon=ARROW_COUNTER_CLOCKWISE /></span>
                            "Reset"
                        </button>
                    </div>
                </form>
            </SectionCard>

            <SectionCard
                title="Daftar Izin"
                description="Urutkan berdasarkan status untuk mengawasi lifecycle izin."
                icon="fas fa-list"
            >
                <p class="mb-3 text-xs text-slate-400">
                    "Total: " <span class="font-semibold text-slate-200">{move || total.get().to_string()}</span> " izin"
                </p>
                {move || {
                    if loading.get() && items.get().is_empty() {
                        view! { <LoadingState message="Memuat daftar izin..." /> }.into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <ErrorState
                                error=err
                                on_retry=Box::new(move || { set_reload_tick.update(|t| *t += 1); })
                            />
                        }.into_any()
                    } else if items.get().is_empty() {
                        view! {
                            <EmptyState
                                title="Tidak ada izin yang cocok"
                                description="Ubah filter lifecycle atau kata kunci untuk menemukan izin."
                                icon="fas fa-clipboard"
                            />
                        }.into_any()
                    } else {
                        render_table(items.get()).into_any()
                    }
                }}

                <div class="mt-4 flex items-center justify-between border-t border-white/[0.06] pt-3 text-xs text-slate-400">
                    <span>
                        "Halaman " <span class="font-semibold text-slate-200">{move || page.get().to_string()}</span>
                        " dari " <span class="font-semibold text-slate-200">{move || total_pages.get().max(1).to_string()}</span>
                    </span>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            on:click=prev_page
                            disabled=move || page.get() <= 1
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            <span class="text-[0.6rem]"><AppIcon icon=CARET_LEFT /></span>
                            "Sebelumnya"
                        </button>
                        <button
                            type="button"
                            on:click=next_page
                            disabled=move || page.get() >= total_pages.get()
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            "Berikutnya"
                            <span class="text-[0.6rem]"><AppIcon icon=CARET_RIGHT /></span>
                        </button>
                    </div>
                </div>
            </SectionCard>
        </PageLayout>
    }
}

fn render_table(items: Vec<IzinPemakaianBmn>) -> impl IntoView + use<> {
    let rows: Vec<_> = items
        .into_iter()
        .map(|p| {
            let id_href = format!("/perlengkapan/pemakaian-bmn/{}", p.id);
            let no = p.nomor_izin.clone().unwrap_or_else(|| "-".to_string());
            let nama = p.pegawai_nama.clone();
            let nip = p.pegawai_nip.clone();
            let bmn = p.bmn_nama_barang.clone();
            let nup = p.bmn_nup.clone();
            let periode = format!("{} → {}", p.tanggal_mulai, p.tanggal_selesai);
            let countdown = countdown_label(&p.tanggal_selesai, &p.status);
            let (status_label, status_tone) = status_descriptor(&p.status);
            view! {
                <tr class="border-b border-white/[0.04] last:border-0">
                    <td class="py-3 pr-3">
                        <p class="text-sm font-semibold text-slate-100">{no}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{periode}</p>
                    </td>
                    <td class="py-3 pr-3">
                        <p class="text-sm font-medium text-slate-100">{nama}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{nip}</p>
                    </td>
                    <td class="py-3 pr-3">
                        <p class="text-sm text-slate-100">{bmn}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{format!("NUP: {}", nup)}</p>
                    </td>
                    <td class="py-3 pr-3">
                        <span class=format!(
                            "inline-flex rounded-full px-2.5 py-0.5 text-[0.65rem] font-semibold ring-1 {}",
                            status_tone
                        )>
                            {status_label}
                        </span>
                        {countdown.map(|(text, tone)| view! {
                            <p class=format!("mt-1 text-[0.65rem] font-medium {}", tone)>{text}</p>
                        })}
                    </td>
                    <td class="py-3 text-right">
                        <A
                            href=id_href
                            attr:class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-2.5 py-1 text-xs text-slate-100 transition hover:bg-white/[0.08]"
                        >
                            <span class="text-[0.6rem]"><AppIcon icon=EYE /></span>
                            "Detail"
                        </A>
                    </td>
                </tr>
            }
        })
        .collect();

    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="border-b border-white/[0.08]">
                    <tr class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                        <th class="py-2 pr-3 text-left font-medium">"Nomor Izin"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Pemohon"</th>
                        <th class="py-2 pr-3 text-left font-medium">"BMN"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Status"</th>
                        <th class="py-2 text-right font-medium">"Aksi"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}

fn status_descriptor(status: &str) -> (&'static str, &'static str) {
    match status {
        "DRAFT" => ("Draft", "bg-slate-500/10 text-slate-300 ring-slate-500/20"),
        "SUBMITTED" => ("Diajukan", "bg-info-500/10 text-info-300 ring-info-500/20"),
        "APPROVED" => (
            "Disetujui",
            "bg-success-500/10 text-success-300 ring-success-500/20",
        ),
        "REJECTED" => (
            "Ditolak",
            "bg-danger-500/10 text-danger-300 ring-danger-500/20",
        ),
        "ACTIVE" => (
            "Aktif",
            "bg-success-500/10 text-success-300 ring-success-500/20",
        ),
        "EXPIRED" => (
            "Kadaluarsa",
            "bg-warning-500/10 text-warning-300 ring-warning-500/20",
        ),
        "REVOKED" => (
            "Dicabut",
            "bg-danger-500/10 text-danger-300 ring-danger-500/20",
        ),
        _ => (
            "Lainnya",
            "bg-slate-500/10 text-slate-300 ring-slate-500/20",
        ),
    }
}

fn countdown_label(tanggal_selesai: &str, status: &str) -> Option<(String, &'static str)> {
    if !matches!(status, "ACTIVE" | "APPROVED") {
        return None;
    }
    let days = days_until(tanggal_selesai)?;
    let (text, tone) = if days < 0 {
        (format!("Lewat {} hari", -days), "text-danger-300")
    } else if days == 0 {
        ("Berakhir hari ini".to_string(), "text-warning-300")
    } else if days <= 7 {
        (format!("{} hari lagi", days), "text-warning-300")
    } else {
        (format!("{} hari lagi", days), "text-slate-400")
    };
    Some((text, tone))
}

fn days_until(date_iso: &str) -> Option<i64> {
    let today = js_sys::Date::new_0();
    let target = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(date_iso));
    if target.get_time().is_nan() {
        return None;
    }
    let diff_ms = target.get_time() - today.get_time();
    Some((diff_ms / (1000.0 * 60.0 * 60.0 * 24.0)).ceil() as i64)
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
