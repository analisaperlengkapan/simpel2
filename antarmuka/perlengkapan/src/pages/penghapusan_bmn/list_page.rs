//! Penghapusan BMN list — workflow-aware deletion browser with SK flag.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{
    ARROW_COUNTER_CLOCKWISE, CARET_LEFT, CARET_RIGHT, EYE, FILE_ARROW_UP, MAGNIFYING_GLASS, PLUS,
    SIGNATURE,
};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent};

use crate::api::common::{PenghapusanBmnFilters, PenghapusanBmnWorkflow};
use crate::api::error::AppError;
use crate::api::penghapusan_bmn;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::components::status_badge::status_tone_classes;
use crate::routes::path;

const PER_PAGE: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StageFilter {
    Semua,
    Draft,
    Wilayah,
    Pusat,
    KonsepSK,
    Tertandatangani,
    Selesai,
    Ditolak,
}

impl StageFilter {
    fn value(self) -> &'static str {
        match self {
            Self::Semua => "",
            Self::Draft => "DRAFT",
            Self::Wilayah => "SUBMIT_WILAYAH",
            Self::Pusat => "SUBMIT_PUSAT",
            Self::KonsepSK => "KONSEP_SK_GENERATED",
            Self::Tertandatangani => "SK_SIGNED",
            Self::Selesai => "COMPLETED",
            Self::Ditolak => "REJECTED",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Semua => "Semua",
            Self::Draft => "Draft",
            Self::Wilayah => "Review Wilayah",
            Self::Pusat => "Review Pusat",
            Self::KonsepSK => "Konsep SK",
            Self::Tertandatangani => "SK Ditandatangani",
            Self::Selesai => "Selesai",
            Self::Ditolak => "Ditolak",
        }
    }
    fn all() -> &'static [StageFilter] {
        &[
            Self::Semua,
            Self::Draft,
            Self::Wilayah,
            Self::Pusat,
            Self::KonsepSK,
            Self::Tertandatangani,
            Self::Selesai,
            Self::Ditolak,
        ]
    }
}

#[component]
pub fn PenghapusanBmnListPage() -> impl IntoView {
    let (items, set_items) = signal::<Vec<PenghapusanBmnWorkflow>>(Vec::new());
    let (total, set_total) = signal::<i64>(0);
    let (page, set_page) = signal(1_i64);
    let (total_pages, set_total_pages) = signal(1_i64);
    let (stage, set_stage) = signal(StageFilter::Semua);
    let (metode, set_metode) = signal::<String>(String::new());
    let (search, set_search) = signal::<String>(String::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let pg = page.get() as i32;
        let stg = stage.get();
        let metode_val = metode.get();
        let search_val = search.get();
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            let status_filter = match stg {
                StageFilter::Semua => None,
                other => Some(other.value().to_string()),
            };
            let metode_opt = non_empty(&metode_val);
            let search_opt = non_empty(&search_val);

            let filters = PenghapusanBmnFilters {
                satker_id: None,
                status: status_filter,
                status_kode: None,
                metode_penghapusan: metode_opt,
                tahun: None,
            };

            match penghapusan_bmn::fetch_penghapusan_bmn_list(pg, PER_PAGE, filters).await {
                Ok(resp) => {
                    let filtered = if let Some(q) = search_opt {
                        let needle = q.to_lowercase();
                        resp.data
                            .into_iter()
                            .filter(|p| {
                                p.kode_barang.to_lowercase().contains(&needle)
                                    || p.nama_barang.to_lowercase().contains(&needle)
                                    || p.nup.to_lowercase().contains(&needle)
                                    || p.alasan.to_lowercase().contains(&needle)
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

    let on_stage = move |val: StageFilter| {
        set_stage.set(val);
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

    let on_metode = move |ev: Event| {
        if let Some(target) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        {
            set_metode.set(target.value());
            set_page.set(1);
            set_reload_tick.update(|t| *t += 1);
        }
    };

    let reset_filters = move |_| {
        set_stage.set(StageFilter::Semua);
        set_metode.set(String::new());
        set_search.set(String::new());
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
        PageBreadcrumb::leaf("Penghapusan BMN"),
    ];

    view! {
        <PageLayout
            title="Penghapusan BMN"
            description="Kelola usulan penghapusan BMN: review, konsep SK, dan unggah SK tertandatangan."
            icon="fas fa-trash-can"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || {
                view! {
                    <A
                        href=path::PENGELOLAAN_PENGHAPUSAN_BUAT
                        attr:class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                    >
                        <span class="text-[0.7rem]">
                            <AppIcon icon=PLUS />
                        </span>
                        "Usulan Baru"
                    </A>
                }
                    .into_any()
            })
        >
            <SectionCard
                title="Filter Fase"
                description="Pilih fase workflow untuk mempersempit daftar."
                icon="fas fa-filter"
            >
                <div class="flex flex-wrap gap-2">
                    {StageFilter::all()
                        .iter()
                        .copied()
                        .map(|s| {
                            let is_active = move || stage.get() == s;
                            view! {
                                <button
                                    type="button"
                                    on:click=move |_| on_stage(s)
                                    class=move || {
                                        let base = "focus-ring inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-xs font-semibold transition";
                                        if is_active() {
                                            format!("{base} bg-gold-gradient text-navy-950 shadow-sm")
                                        } else {
                                            format!(
                                                "{base} border border-white/10 bg-white/[0.04] text-slate-200 hover:bg-white/[0.08]",
                                            )
                                        }
                                    }
                                >
                                    {s.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <form
                    class="mt-4 grid gap-3 md:grid-cols-[1fr_1fr_auto]"
                    on:submit=on_search_submit
                >
                    <input
                        type="text"
                        placeholder="Cari kode barang, nama BMN, NUP, alasan..."
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        on:input=on_search_input
                        prop:value=move || search.get()
                    />
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_metode
                        prop:value=move || metode.get()
                    >
                        <option value="">"Semua Metode"</option>
                        <option value="LELANG">"Lelang"</option>
                        <option value="HIBAH">"Hibah"</option>
                        <option value="PEMUSNAHAN">"Pemusnahan"</option>
                        <option value="PENGHAPUSAN_LAIN">"Penghapusan Lain"</option>
                    </select>
                    <div class="flex gap-2">
                        <button
                            type="submit"
                            class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                        >
                            <span class="text-xs">
                                <AppIcon icon=MAGNIFYING_GLASS />
                            </span>
                            "Cari"
                        </button>
                        <button
                            type="button"
                            on:click=reset_filters
                            class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200 transition hover:bg-white/[0.08]"
                        >
                            <span class="text-xs">
                                <AppIcon icon=ARROW_COUNTER_CLOCKWISE />
                            </span>
                            "Reset"
                        </button>
                    </div>
                </form>
            </SectionCard>

            <SectionCard
                title="Daftar Usulan"
                description="Baris ditandai jika perlu tindakan: unggah SK tertandatangan."
                icon="fas fa-list"
            >
                <p class="mb-3 text-xs text-slate-400">
                    "Total: "
                    <span class="font-semibold text-slate-200">
                        {move || total.get().to_string()}
                    </span> " usulan"
                </p>
                {move || {
                    if loading.get() && items.get().is_empty() {
                        view! { <LoadingState message="Memuat daftar usulan penghapusan..." /> }
                            .into_any()
                    } else if let Some(err) = error.get() {
                        view! {
                            <ErrorState
                                error=err
                                on_retry=Box::new(move || {
                                    set_reload_tick.update(|t| *t += 1);
                                })
                            />
                        }
                            .into_any()
                    } else if items.get().is_empty() {
                        view! {
                            <EmptyState
                                title="Tidak ada usulan yang cocok"
                                description="Ubah filter fase atau kata kunci untuk menemukan usulan."
                                icon="fas fa-clipboard"
                            />
                        }
                            .into_any()
                    } else {
                        render_table(items.get()).into_any()
                    }
                }}

                <div class="mt-4 flex items-center justify-between border-t border-white/[0.06] pt-3 text-xs text-slate-400">
                    <span>
                        "Halaman "
                        <span class="font-semibold text-slate-200">
                            {move || page.get().to_string()}
                        </span> " dari "
                        <span class="font-semibold text-slate-200">
                            {move || total_pages.get().max(1).to_string()}
                        </span>
                    </span>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            on:click=prev_page
                            disabled=move || page.get() <= 1
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            <span class="text-[0.6rem]">
                                <AppIcon icon=CARET_LEFT />
                            </span>
                            "Sebelumnya"
                        </button>
                        <button
                            type="button"
                            on:click=next_page
                            disabled=move || { page.get() >= total_pages.get() }
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            "Berikutnya"
                            <span class="text-[0.6rem]">
                                <AppIcon icon=CARET_RIGHT />
                            </span>
                        </button>
                    </div>
                </div>
            </SectionCard>
        </PageLayout>
    }
}

fn render_table(items: Vec<PenghapusanBmnWorkflow>) -> impl IntoView + use<> {
    let rows: Vec<_> = items
        .into_iter()
        .map(|p| {
            let id_href = crate::routes::url::penghapusan_detail(&p.id);
            let kode = p.kode_barang.clone();
            let nama = p.nama_barang.clone();
            let nup = p.nup.clone();
            let tgl = p.tanggal_penghapusan.clone();
            let metode = p.metode_penghapusan.clone();
            let status_label = p.status_label.clone();
            let status_tone = status_tone_classes(&p.status_tone);
            let needs_upload = p.status == "KONSEP_SK_GENERATED" && p.signed_sk_pdf_url.is_none();
            let sk_generated = p.konsep_sk_url.is_some();
            view! {
                <tr class="border-b border-white/[0.04] last:border-0">
                    <td class="py-3 pr-3">
                        <p class="text-sm font-semibold text-slate-100">{kode}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{format!("NUP: {}", nup)}</p>
                    </td>
                    <td class="py-3 pr-3">
                        <p class="text-sm font-medium text-slate-100">{nama}</p>
                        <p class="mt-0.5 text-[0.7rem] text-slate-500">{format!("Tgl: {}", tgl)}</p>
                    </td>
                    <td class="py-3 pr-3">
                        <span class="inline-flex rounded-full bg-white/[0.04] px-2.5 py-0.5 text-[0.65rem] font-medium text-slate-300 ring-1 ring-white/10">
                            {metode}
                        </span>
                    </td>
                    <td class="py-3 pr-3">
                        <span class=format!(
                            "inline-flex rounded-full px-2.5 py-0.5 text-[0.65rem] font-semibold ring-1 {}",
                            status_tone,
                        )>{status_label}</span>
                        {if needs_upload {
                            Some(
                                view! {
                                    <p class="mt-1 text-[0.65rem] font-semibold text-warning-300">
                                        <span class="mr-1">
                                            <AppIcon icon=FILE_ARROW_UP />
                                        </span>
                                        "Perlu unggah SK"
                                    </p>
                                },
                            )
                        } else if sk_generated && p.signed_sk_pdf_url.is_some() {
                            Some(
                                view! {
                                    <p class="mt-1 text-[0.65rem] font-medium text-success-300">
                                        <span class="mr-1">
                                            <AppIcon icon=SIGNATURE />
                                        </span>
                                        "SK tersedia"
                                    </p>
                                },
                            )
                        } else {
                            None
                        }}
                    </td>
                    <td class="py-3 text-right">
                        <A
                            href=id_href
                            attr:class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-2.5 py-1 text-xs text-slate-100 transition hover:bg-white/[0.08]"
                        >
                            <span class="text-[0.6rem]">
                                <AppIcon icon=EYE />
                            </span>
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
                        <th class="py-2 pr-3 text-left font-medium">"Kode Barang"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Nama BMN"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Metode"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Status"</th>
                        <th class="py-2 text-right font-medium">"Aksi"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}

fn non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}
