//! Bank Aset unified list — filter + search + pagination over SIMAN data.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{
    ARROW_COUNTER_CLOCKWISE, CARET_LEFT, CARET_RIGHT, CHART_LINE, EYE, MAGNIFYING_GLASS, QR_CODE,
};
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent};

use super::dashboard_page::{format_rupiah, format_thousands};
use crate::api::bank_aset::{self, BankAsetItem, ListFilter};
use crate::api::common::PaginatedResponse;
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::{path, url};

const PER_PAGE: i32 = 25;

#[component]
pub fn BankAsetListPage() -> impl IntoView {
    let (items, set_items) = signal::<Vec<BankAsetItem>>(Vec::new());
    let (total, set_total) = signal::<i64>(0);
    let (total_pages, set_total_pages) = signal::<i32>(1);
    let (page, set_page) = signal::<i32>(1);
    let (jenis, set_jenis) = signal::<String>(String::new());
    let (kategori, set_kategori) = signal::<String>(String::new());
    let (kondisi, set_kondisi) = signal::<String>(String::new());
    let (satker, set_satker) = signal::<String>(String::new());
    let (search, set_search) = signal::<String>(String::new());
    let (sort, set_sort) = signal::<String>(String::new());

    // Filter dropdown values loaded dynamically from real SIMAN data.
    let (jenis_opts, set_jenis_opts) = signal::<Vec<bank_aset::FilterOption>>(Vec::new());
    let (kategori_opts, set_kategori_opts) = signal::<Vec<bank_aset::FilterOption>>(Vec::new());
    let (kondisi_opts, set_kondisi_opts) = signal::<Vec<bank_aset::FilterOption>>(Vec::new());
    let (satker_opts, set_satker_opts) = signal::<Vec<bank_aset::FilterOption>>(Vec::new());

    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);

    // Load filter options once on mount (no reactive deps → runs a single time).
    Effect::new(move |_| {
        spawn_local(async move {
            if let Ok(opts) = bank_aset::fetch_filter_options().await {
                set_jenis_opts.set(opts.jenis);
                set_kategori_opts.set(opts.kategori);
                set_kondisi_opts.set(opts.kondisi);
                set_satker_opts.set(opts.satker);
            }
        });
    });

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let filter = ListFilter {
            page: page.get(),
            per_page: PER_PAGE,
            jenis: some_if_non_empty(&jenis.get()),
            kategori: some_if_non_empty(&kategori.get()),
            kondisi: some_if_non_empty(&kondisi.get()),
            satker: some_if_non_empty(&satker.get()),
            search: some_if_non_empty(&search.get()),
            sort: some_if_non_empty(&sort.get()),
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match bank_aset::fetch_list(&filter).await {
                Ok(PaginatedResponse {
                    data,
                    total,
                    total_pages,
                    ..
                }) => {
                    set_items.set(data);
                    set_total.set(total);
                    set_total_pages.set(total_pages.max(1));
                }
                Err(e) => {
                    set_error.set(Some(e));
                }
            }
            set_loading.set(false);
        });
    });

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Bank Aset", path::BANK_ASET_DASHBOARD),
        PageBreadcrumb::leaf("Daftar"),
    ];

    let on_search_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };
    let on_jenis = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
        if let Some(el) = target {
            set_jenis.set(el.value());
            set_page.set(1);
        }
    };
    let on_kategori = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
        if let Some(el) = target {
            set_kategori.set(el.value());
            set_page.set(1);
        }
    };
    let on_kondisi = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
        if let Some(el) = target {
            set_kondisi.set(el.value());
            set_page.set(1);
        }
    };
    let on_sort = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
        if let Some(el) = target {
            set_sort.set(el.value());
            set_page.set(1);
        }
    };
    let on_search_input = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(el) = target {
            set_search.set(el.value());
        }
    };
    let on_satker = move |ev: Event| {
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok());
        if let Some(el) = target {
            set_satker.set(el.value());
            set_page.set(1);
        }
    };
    let reset_filters = move |_| {
        set_jenis.set(String::new());
        set_kategori.set(String::new());
        set_kondisi.set(String::new());
        set_satker.set(String::new());
        set_search.set(String::new());
        set_sort.set(String::new());
        set_page.set(1);
        set_reload_tick.update(|t| *t += 1);
    };

    let go_prev = move |_| {
        let cur = page.get();
        if cur > 1 {
            set_page.set(cur - 1);
        }
    };
    let go_next = move |_| {
        let cur = page.get();
        if cur < total_pages.get() {
            set_page.set(cur + 1);
        }
    };

    view! {
        <PageLayout
            title="Daftar Aset"
            description="Jelajahi, cari, dan filter seluruh BMN dari SIMAN."
            icon="fas fa-warehouse"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || view! {
                <A
                    href=path::BANK_ASET_DASHBOARD
                    attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                >
                    <span class="text-[0.7rem]"><AppIcon icon=CHART_LINE /></span>
                    "Dashboard"
                </A>
                <A
                    href=path::BANK_ASET_QRCODE
                    attr:class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                >
                    <span class="text-[0.7rem]"><AppIcon icon=QR_CODE /></span>
                    "QR Code"
                </A>
            }.into_any())
        >
            <SectionCard
                title="Filter"
                description="Batasi daftar berdasarkan kategori, kondisi, atau kata kunci."
                icon="fas fa-filter"
            >
                <form class="grid gap-3 md:grid-cols-[1fr_1fr_1fr_1fr_2fr_auto]" on:submit=on_search_submit>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_jenis
                        prop:value=move || jenis.get()
                    >
                        <option value="">"Semua Jenis BMN"</option>
                        {move || {
                            jenis_opts
                                .get()
                                .into_iter()
                                .map(|o| {
                                    let label = format!("{} ({})", o.value, o.count);
                                    view! { <option value=o.value.clone()>{label}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_kategori
                        prop:value=move || kategori.get()
                    >
                        <option value="">"Semua Kategori"</option>
                        {move || {
                            kategori_opts
                                .get()
                                .into_iter()
                                .map(|o| {
                                    let label = format!("{} ({})", o.value, o.count);
                                    view! { <option value=o.value.clone()>{label}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_kondisi
                        prop:value=move || kondisi.get()
                    >
                        <option value="">"Semua Kondisi"</option>
                        {move || {
                            kondisi_opts
                                .get()
                                .into_iter()
                                .map(|o| {
                                    let label = format!("{} ({})", o.value, o.count);
                                    view! { <option value=o.value.clone()>{label}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_sort
                        prop:value=move || sort.get()
                    >
                        <option value="">"Terbaru diperbarui"</option>
                        <option value="updated_at_asc">"Terlama diperbarui"</option>
                        <option value="nama_asc">"Nama A→Z"</option>
                        <option value="nama_desc">"Nama Z→A"</option>
                        <option value="nilai_desc">"Nilai tertinggi"</option>
                        <option value="nilai_asc">"Nilai terendah"</option>
                    </select>
                    <input
                        type="text"
                        placeholder="Cari nama/kode/NUP/merk..."
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        on:input=on_search_input
                        prop:value=move || search.get()
                    />
                    <div class="flex gap-2">
                        <button
                            type="submit"
                            class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                        >
                            <span class="text-xs"><AppIcon icon=MAGNIFYING_GLASS /></span>
                            "Terapkan"
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
                <select
                    class="focus-ring mt-3 w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                    on:change=on_satker
                    prop:value=move || satker.get()
                >
                    <option value="">"Semua Satker"</option>
                    {move || {
                        satker_opts
                            .get()
                            .into_iter()
                            .map(|o| {
                                let label = format!("{} ({})", o.value, o.count);
                                view! { <option value=o.value.clone()>{label}</option> }
                            })
                            .collect_view()
                    }}
                </select>
            </SectionCard>

            <SectionCard
                title="Daftar Aset"
                description="Data bersumber dari SIMAN, disinkronkan secara berkala."
                icon="fas fa-list"
            >
                <p class="mb-3 text-xs text-slate-400">
                    "Total: " <span class="font-semibold text-slate-200">{move || format_thousands(total.get())}</span> " aset"
                </p>
                {move || {
                    if loading.get() && items.get().is_empty() {
                        view! { <LoadingState message="Memuat daftar aset..." /> }.into_any()
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
                                title="Tidak ada aset"
                                description="Coba ubah filter atau kata kunci pencarian."
                                icon="fas fa-box-open"
                            />
                        }.into_any()
                    } else {
                        render_table(items.get()).into_any()
                    }
                }}
                <div class="mt-4 flex flex-col items-center justify-between gap-3 border-t border-white/[0.05] pt-4 sm:flex-row">
                    <p class="text-xs text-slate-400">
                        "Halaman " <span class="font-semibold text-slate-200">{move || page.get()}</span>
                        " dari " <span class="font-semibold text-slate-200">{move || total_pages.get()}</span>
                    </p>
                    <div class="flex items-center gap-2">
                        <button
                            type="button"
                            on:click=go_prev
                            disabled=move || page.get() <= 1 || loading.get()
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            <span class="text-[0.6rem]"><AppIcon icon=CARET_LEFT /></span>
                            "Sebelumnya"
                        </button>
                        <button
                            type="button"
                            on:click=go_next
                            disabled=move || page.get() >= total_pages.get() || loading.get()
                            class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08] disabled:opacity-40"
                        >
                            "Selanjutnya"
                            <span class="text-[0.6rem]"><AppIcon icon=CARET_RIGHT /></span>
                        </button>
                    </div>
                </div>
            </SectionCard>
        </PageLayout>
    }
}

fn render_table(items: Vec<BankAsetItem>) -> impl IntoView {
    let rows: Vec<_> = items.into_iter().map(|item| {
        let id = item.id.clone();
        let detail_href = url::bank_aset_detail(&id);
        let nama = item.nama_aset.clone().unwrap_or_else(|| "-".to_string());
        let kategori = item.kategori_aset.clone();
        let kode = item.kode_barang.clone().unwrap_or_else(|| "-".to_string());
        let nup = item.nup.clone().unwrap_or_else(|| "-".to_string());
        let kondisi = item.kondisi.clone().unwrap_or_else(|| "-".to_string());
        let satker = item.satker.clone().unwrap_or_else(|| "-".to_string());
        let nilai = item.nilai_perolehan.map(format_rupiah).unwrap_or_else(|| "-".to_string());
        let tone = kondisi_tone(&kondisi);

        view! {
            <tr class="border-b border-white/[0.04] transition hover:bg-white/[0.02]">
                <td class="py-3 pr-3">
                    <A href=detail_href attr:class="text-sm font-semibold text-slate-100 transition hover:text-gold-300">
                        {nama}
                    </A>
                    <p class="mt-0.5 text-[0.7rem] uppercase tracking-wide text-slate-500">{kategori}</p>
                </td>
                <td class="py-3 pr-3 text-xs text-slate-300">{kode}</td>
                <td class="py-3 pr-3 text-xs text-slate-300">{nup}</td>
                <td class="py-3 pr-3">
                    <span class=format!("inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[0.65rem] font-semibold ring-1 {}", tone)>
                        {kondisi}
                    </span>
                </td>
                <td class="py-3 pr-3 text-xs text-slate-300">{satker}</td>
                <td class="py-3 pr-3 text-right text-xs text-slate-300">{nilai}</td>
                <td class="py-3 text-right">
                    <A href=url::bank_aset_detail(&id)
                       attr:class="focus-ring inline-flex items-center gap-1 rounded-md border border-white/10 bg-white/[0.04] px-2 py-1 text-[0.7rem] text-slate-200 transition hover:bg-white/[0.08]">
                        <span class="text-[0.6rem]"><AppIcon icon=EYE /></span>
                        "Detail"
                    </A>
                </td>
            </tr>
        }
    }).collect();

    view! {
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="border-b border-white/[0.08]">
                    <tr class="text-[0.65rem] uppercase tracking-wide text-slate-500">
                        <th class="py-2 pr-3 text-left font-medium">"Aset"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Kode"</th>
                        <th class="py-2 pr-3 text-left font-medium">"NUP"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Kondisi"</th>
                        <th class="py-2 pr-3 text-left font-medium">"Satker"</th>
                        <th class="py-2 pr-3 text-right font-medium">"Nilai"</th>
                        <th class="py-2 text-right font-medium"></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}

fn kondisi_tone(kondisi: &str) -> &'static str {
    let u = kondisi.to_uppercase();
    if u.contains("BAIK") {
        "bg-success-500/10 text-success-300 ring-success-500/25"
    } else if u.contains("RUSAK RINGAN") {
        "bg-warning-500/10 text-warning-300 ring-warning-500/25"
    } else if u.contains("RUSAK") {
        "bg-danger-500/10 text-danger-300 ring-danger-500/25"
    } else {
        "bg-white/[0.05] text-slate-300 ring-white/10"
    }
}

fn some_if_non_empty(s: &str) -> Option<String> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

use wasm_bindgen::JsCast;
