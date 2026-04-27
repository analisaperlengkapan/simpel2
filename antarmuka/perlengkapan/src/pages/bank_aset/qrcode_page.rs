//! Bank Aset QR code generator — batch-ready label printer.

use leptos::prelude::*;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{ARROW_LEFT, CHECKS, ERASER, MAGNIFYING_GLASS, PRINTER, X};
use leptos::task::spawn_local;
use leptos_router::components::A;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement, SubmitEvent};

use qrcode::QrCode;
use qrcode::render::svg as qrsvg;
use serde::Serialize;

use crate::api::bank_aset::{self, BankAsetItem, ListFilter};
use crate::api::common::PaginatedResponse;
use crate::api::error::AppError;
use crate::components::layout::{
    EmptyState, ErrorState, LoadingState, PageBreadcrumb, PageLayout, SectionCard,
};
use crate::routes::path;

const PER_PAGE: i32 = 50;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LabelSize {
    Small,  // 30 x 60 mm (legacy compat)
    Medium, // 40 x 80 mm
    Large,  // 50 x 100 mm
}

impl LabelSize {
    fn value(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
    fn from_str(s: &str) -> Self {
        match s {
            "medium" => Self::Medium,
            "large" => Self::Large,
            _ => Self::Small,
        }
    }
    fn dimensions(self) -> (&'static str, &'static str) {
        match self {
            Self::Small => ("30mm", "60mm"),
            Self::Medium => ("40mm", "80mm"),
            Self::Large => ("50mm", "100mm"),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Small => "30×60 mm (kompatibel legacy)",
            Self::Medium => "40×80 mm",
            Self::Large => "50×100 mm",
        }
    }
}

#[derive(Serialize)]
struct QrPayload<'a> {
    s: &'a str,
    k: &'a str,
    n: &'a str,
    t: &'a str,
    i: &'a str,
}

#[component]
pub fn BankAsetQrCodePage() -> impl IntoView {
    let (items, set_items) = signal::<Vec<BankAsetItem>>(Vec::new());
    let (kategori, set_kategori) = signal::<String>(String::new());
    let (search, set_search) = signal::<String>(String::new());
    let (label_size, set_label_size) = signal(LabelSize::Small);
    let (selected, set_selected) = signal::<HashSet<String>>(HashSet::new());
    let (loading, set_loading) = signal(true);
    let (error, set_error) = signal::<Option<AppError>>(None);
    let (reload_tick, set_reload_tick) = signal(0_u32);
    let (show_preview, set_show_preview) = signal(false);

    Effect::new(move |_| {
        let _ = reload_tick.get();
        let filter = ListFilter {
            page: 1,
            per_page: PER_PAGE,
            kategori: non_empty(&kategori.get()),
            kondisi: None,
            satker: None,
            search: non_empty(&search.get()),
            sort: None,
        };
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match bank_aset::fetch_list(&filter).await {
                Ok(PaginatedResponse { data, .. }) => set_items.set(data),
                Err(e) => set_error.set(Some(e)),
            }
            set_loading.set(false);
        });
    });

    let on_kategori = move |ev: Event| {
        if let Some(el) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        {
            set_kategori.set(el.value());
        }
    };
    let on_size = move |ev: Event| {
        if let Some(el) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        {
            set_label_size.set(LabelSize::from_str(&el.value()));
        }
    };
    let on_search_input = move |ev: Event| {
        if let Some(el) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            set_search.set(el.value());
        }
    };
    let on_search_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_reload_tick.update(|t| *t += 1);
    };

    let toggle_id = move |id: String| {
        set_selected.update(|s| {
            if !s.remove(&id) {
                s.insert(id);
            }
        });
    };
    let select_all = move |_| {
        let ids: HashSet<String> = items.get().iter().map(|it| it.id.clone()).collect();
        set_selected.set(ids);
    };
    let clear_all = move |_| set_selected.set(HashSet::new());

    let selected_items = move || {
        let sel = selected.get();
        items
            .get()
            .into_iter()
            .filter(|it| sel.contains(&it.id))
            .collect::<Vec<_>>()
    };

    let open_preview = move |_| set_show_preview.set(true);
    let close_preview = move |_| set_show_preview.set(false);
    let print_labels = move |_| {
        if let Some(w) = web_sys::window() {
            let _ = w.print();
        }
    };

    let breadcrumbs = vec![
        PageBreadcrumb::new("Dashboard", path::DASHBOARD),
        PageBreadcrumb::new("Bank Aset", path::BANK_ASET_DASHBOARD),
        PageBreadcrumb::leaf("QR Code"),
    ];

    view! {
        <PageLayout
            title="Generator QR Code Aset"
            description="Pilih aset, tentukan ukuran label, lalu cetak batch untuk penandaan fisik BMN."
            icon="fas fa-qrcode"
            breadcrumbs=breadcrumbs
            actions=Box::new(move || view! {
                <A
                    href=path::BANK_ASET_DAFTAR
                    attr:class="focus-ring inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]"
                >
                    <span class="text-[0.7rem]"><AppIcon icon=ARROW_LEFT /></span>
                    "Kembali"
                </A>
            }.into_any())
        >
            <SectionCard
                title="Konfigurasi"
                description="Filter aset yang ingin dicetak dan pilih ukuran label."
                icon="fas fa-sliders"
            >
                <form class="grid gap-3 md:grid-cols-[1fr_1fr_2fr_auto]" on:submit=on_search_submit>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_kategori
                        prop:value=move || kategori.get()
                    >
                        <option value="">"Semua Kategori"</option>
                        <option value="TANAH">"Tanah"</option>
                        <option value="GEDUNG_BANGUNAN">"Gedung & Bangunan"</option>
                        <option value="PERALATAN_MESIN">"Peralatan & Mesin"</option>
                        <option value="JALAN_JEMBATAN">"Jalan & Jembatan"</option>
                        <option value="ASET_LAINNYA">"Aset Lainnya"</option>
                    </select>
                    <select
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100"
                        on:change=on_size
                        prop:value=move || label_size.get().value().to_string()
                    >
                        <option value="small">{LabelSize::Small.label()}</option>
                        <option value="medium">{LabelSize::Medium.label()}</option>
                        <option value="large">{LabelSize::Large.label()}</option>
                    </select>
                    <input
                        type="text"
                        placeholder="Cari nama/kode/NUP..."
                        class="focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-100 placeholder-slate-500"
                        on:input=on_search_input
                        prop:value=move || search.get()
                    />
                    <button
                        type="submit"
                        class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                    >
                        <span class="text-xs"><AppIcon icon=MAGNIFYING_GLASS /></span>
                        "Cari"
                    </button>
                </form>
            </SectionCard>

            <SectionCard
                title="Pilih Aset"
                description="Centang aset yang ingin dicetak. Gunakan tombol bulk untuk mempercepat."
                icon="fas fa-list-check"
                actions=Box::new(move || view! {
                    <span class="text-xs font-semibold text-slate-300">
                        {move || format!("{} terpilih", selected.get().len())}
                    </span>
                    <button
                        type="button"
                        on:click=select_all
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.6rem]"><AppIcon icon=CHECKS /></span>
                        "Pilih semua"
                    </button>
                    <button
                        type="button"
                        on:click=clear_all
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08]"
                    >
                        <span class="text-[0.6rem]"><AppIcon icon=ERASER /></span>
                        "Bersihkan"
                    </button>
                    <button
                        type="button"
                        on:click=open_preview
                        disabled=move || selected.get().is_empty()
                        class="focus-ring inline-flex items-center gap-1.5 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-40"
                    >
                        <span class="text-[0.6rem]"><AppIcon icon=PRINTER /></span>
                        "Pratinjau & Cetak"
                    </button>
                }.into_any())
            >
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
                                title="Tidak ada aset yang cocok"
                                description="Ubah kata kunci atau kategori untuk menemukan aset."
                                icon="fas fa-box-open"
                            />
                        }.into_any()
                    } else {
                        render_selection_list(items.get(), selected, toggle_id).into_any()
                    }
                }}
            </SectionCard>

            <Show when=move || show_preview.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm print:static print:bg-transparent print:backdrop-blur-none">
                    <div class="flex h-[90vh] w-[95vw] max-w-5xl flex-col rounded-2xl border border-white/[0.06] bg-surface-panel shadow-2xl print:h-auto print:w-full print:max-w-none print:rounded-none print:border-0 print:shadow-none">
                        <header class="flex items-center justify-between border-b border-white/[0.06] px-5 py-3 print:hidden">
                            <div>
                                <h2 class="text-sm font-semibold text-white">"Pratinjau Label QR Code"</h2>
                                <p class="text-xs text-slate-400">
                                    {move || format!("{} label · ukuran {}", selected.get().len(), label_size.get().label())}
                                </p>
                            </div>
                            <div class="flex items-center gap-2">
                                <button
                                    type="button"
                                    on:click=print_labels
                                    class="focus-ring inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-3 py-1.5 text-xs font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                                >
                                    <span class="text-[0.7rem]"><AppIcon icon=PRINTER /></span>
                                    "Cetak"
                                </button>
                                <button
                                    type="button"
                                    on:click=close_preview
                                    class="focus-ring inline-flex items-center gap-1 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-200 transition hover:bg-white/[0.08]"
                                >
                                    <span class="text-[0.7rem]"><AppIcon icon=X /></span>
                                    "Tutup"
                                </button>
                            </div>
                        </header>
                        <div class="flex-1 overflow-y-auto bg-white px-5 py-4 print:overflow-visible print:px-0 print:py-0">
                            {move || render_label_sheet(selected_items(), label_size.get())}
                        </div>
                    </div>
                </div>
            </Show>
        </PageLayout>
    }
}

fn render_selection_list(
    items: Vec<BankAsetItem>,
    selected: ReadSignal<HashSet<String>>,
    toggle: impl Fn(String) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let rows: Vec<_> = items.into_iter().map(|item| {
        let id = item.id.clone();
        let id_for_toggle = id.clone();
        let id_for_check = id.clone();
        let nama = item.nama_aset.clone().unwrap_or_else(|| "-".to_string());
        let kode = item.kode_barang.clone().unwrap_or_else(|| "-".to_string());
        let nup = item.nup.clone().unwrap_or_else(|| "-".to_string());
        let satker = item.satker.clone().unwrap_or_else(|| "-".to_string());
        let checked = move || selected.get().contains(&id_for_check);

        view! {
            <li class="flex items-center gap-3 rounded-lg border border-white/[0.05] bg-white/[0.02] px-3 py-2 transition hover:bg-white/[0.04]">
                <input
                    type="checkbox"
                    class="h-4 w-4 cursor-pointer rounded border-white/20 bg-white/[0.08] accent-gold-400"
                    prop:checked=checked
                    on:change=move |_| toggle(id_for_toggle.clone())
                />
                <div class="min-w-0 flex-1">
                    <p class="truncate text-sm font-semibold text-slate-100">{nama}</p>
                    <p class="mt-0.5 truncate text-[0.7rem] text-slate-500">
                        {"Kode: "} {kode} {" · NUP: "} {nup} {" · "} {satker}
                    </p>
                </div>
            </li>
        }
    }).collect();
    view! { <ul class="flex flex-col gap-2">{rows}</ul> }.into_any()
}

fn render_label_sheet(items: Vec<BankAsetItem>, size: LabelSize) -> impl IntoView {
    if items.is_empty() {
        return view! {
            <p class="py-10 text-center text-sm text-slate-500">"Pilih minimal satu aset untuk dicetak."</p>
        }.into_any();
    }
    let (h, w) = size.dimensions();
    let container_style = format!(
        "display: grid; grid-template-columns: repeat(auto-fill, minmax({w}, 1fr)); gap: 6px; color: #0f172a;"
    );
    let labels: Vec<_> = items
        .into_iter()
        .map(|item| render_single_label(item, h, w))
        .collect();
    view! { <div style=container_style>{labels}</div> }.into_any()
}

fn render_single_label(item: BankAsetItem, height: &str, width: &str) -> impl IntoView + use<> {
    let kode = item.kode_barang.clone().unwrap_or_default();
    let nup = item.nup.clone().unwrap_or_default();
    let kode_satker = item.kode_satker.clone().unwrap_or_default();
    let tipe = item.kategori_aset.clone();
    let id_str = item.id.clone();

    let payload = QrPayload {
        s: &kode_satker,
        k: &kode,
        n: &nup,
        t: &tipe,
        i: &id_str,
    };
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();

    let svg = match QrCode::new(payload_str.as_bytes()) {
        Ok(code) => code
            .render::<qrsvg::Color>()
            .min_dimensions(80, 80)
            .quiet_zone(false)
            .dark_color(qrsvg::Color("#0f172a"))
            .light_color(qrsvg::Color("#ffffff"))
            .build(),
        Err(_) => {
            "<svg xmlns='http://www.w3.org/2000/svg' width='80' height='80'></svg>".to_string()
        }
    };

    let nama = item.nama_aset.clone().unwrap_or_else(|| "-".to_string());
    let style = format!(
        "width: {width}; height: {height}; border: 1px solid #94a3b8; padding: 2mm; display: flex; flex-direction: column; gap: 1mm; break-inside: avoid; font-family: sans-serif; font-size: 6pt; background: #ffffff;"
    );

    view! {
        <div style=style>
            <div style="display: flex; align-items: center; justify-content: space-between; gap: 2mm;">
                <div style="flex: 1; min-width: 0;">
                    <p style="font-weight: 700; font-size: 6pt; margin: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{nama}</p>
                    <p style="margin: 0.5mm 0 0 0; font-size: 5pt; color: #475569;">{format!("Kode: {kode}")}</p>
                    <p style="margin: 0; font-size: 5pt; color: #475569;">{format!("NUP: {nup}")}</p>
                    <p style="margin: 0; font-size: 5pt; color: #475569;">{format!("Satker: {kode_satker}")}</p>
                </div>
                <div
                    style="width: 18mm; height: 18mm; flex-shrink: 0;"
                    inner_html=svg
                ></div>
            </div>
            <p style="margin: 0; text-align: center; font-size: 5pt; color: #64748b; letter-spacing: 0.5px;">
                "SIMPel Kejaksaan RI"
            </p>
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
