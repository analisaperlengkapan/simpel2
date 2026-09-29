//! Pakaian Dinas Laporan (Reports) Component
//!
//! Dashboard for viewing various reports on uniform data.
//! Format follows simpel_web-main: cetakRekapTemplateV & cetakDaftarTemplateV

use crate::api::{
    AppError, JenisPakaianDinas, LaporanDaftarPegawai, LaporanQuery, LaporanRekapUkuran,
    PaginatedResponse, PengajuanPakaianDinas, PengajuanSatker, export_laporan_pakaian_dinas,
    fetch_jenis_pakaian_dinas, fetch_laporan_daftar_pegawai, fetch_laporan_rekap_ukuran,
    fetch_pengajuan_pakaian_dinas, fetch_pengajuan_satker,
};
use crate::components::layout::{EmptyState, ErrorState, LoadingState, PageLayout, SectionCard};
use leptos::prelude::*;
use leptos_fetch::QueryClient;
use lib_ui::components::icon::{AppIcon, icon_from_fa_class};
use phosphor_leptos::{CHART_PIE, FILE_PDF, FILE_XLS, USERS};

/// Filter-option queries — both keyed `()` because the dropdowns
/// always show "first 100 active items" regardless of context.
async fn query_pengajuan_options(_: ()) -> Result<Vec<PengajuanPakaianDinas>, AppError> {
    fetch_pengajuan_pakaian_dinas(1, 100, None)
        .await
        .map(|r| r.data)
}

async fn query_jenis_options(_: ()) -> Result<Vec<JenisPakaianDinas>, AppError> {
    fetch_jenis_pakaian_dinas(1, 100).await.map(|r| r.data)
}

/// The satker taking part in ONE campaign, keyed by that campaign.
///
/// Unlike the two above this cannot be keyed `()`: which satker exist is a
/// property of the campaign, and a national list would offer 238 satker of
/// which a given campaign may involve three. It is also already scoped
/// server-side — an operator sees only their own — so the dropdown cannot
/// offer a satker whose report the caller may not read.
async fn query_satker_options(
    pengajuan_id: Option<String>,
) -> Result<Vec<PengajuanSatker>, AppError> {
    let Some(id) = pengajuan_id else {
        return Ok(Vec::new());
    };
    fetch_pengajuan_satker(id, 1, 500).await.map(|r| r.data)
}

/// Rekap query keyed by the entire filter struct — so toggling
/// back to a previously seen filter combination is instant.
///
/// `pengajuan_id` is OPTIONAL on this side and REQUIRED on the other: the
/// handler takes `Query<LaporanRekapQuery>` whose `pengajuan_id` is a plain
/// `Uuid`, so axum rejects a request without it before the handler runs —
///
///     400  Failed to deserialize query string: missing field `pengajuan_id`
///
/// The page mounts before anything is selected, so it fired that request on
/// every load and the user was shown an error state instead of being told to
/// pick a pengajuan. `download_export` in this same file already guards on
/// exactly this (`let Some(pengajuan_id) = … else { return }`); the guard was
/// simply never carried across to the two read paths.
async fn query_laporan_rekap(query: LaporanQuery) -> Result<Vec<LaporanRekapUkuran>, AppError> {
    if query.pengajuan_id.is_none() {
        return Ok(Vec::new());
    }
    fetch_laporan_rekap_ukuran(query).await.map(|r| r.data)
}

/// Daftar pegawai query keyed by `(filter, page)`.
/// Same required-vs-optional mismatch as `query_laporan_rekap`:
/// `get_laporan_daftar_pegawai` takes `Query<LaporanDaftarQuery>` with a plain
/// `Uuid` `pengajuan_id`, so it 400s before the handler for a request that
/// omits it — page and limit make no difference.
async fn query_laporan_daftar(
    key: (LaporanQuery, i32),
) -> Result<PaginatedResponse<LaporanDaftarPegawai>, AppError> {
    let (query, page) = key;
    if query.pengajuan_id.is_none() {
        return Ok(PaginatedResponse {
            success: true,
            data: Vec::new(),
            total: 0,
            page,
            per_page: 20,
            total_pages: 0,
            message: String::new(),
        });
    }
    fetch_laporan_daftar_pegawai(query, page, 20).await
}

/// Translate the active tab id into the `jenis_laporan` the export endpoint
/// understands.
///
/// The tabs are keyed `"rekap"` / `"pegawai"`, but `cetak_laporan` matches on
/// `"rekap"` / `"daftar"` and 400s on anything else — so passing the tab id
/// straight through meant every export from the pegawai tab failed.
fn jenis_laporan_for_tab(tab: &str) -> &'static str {
    match tab {
        "pegawai" => "daftar",
        _ => "rekap",
    }
}

/// Fetch the export with the JWT attached, then hand the bytes to the browser
/// through a blob URL and a synthetic anchor.
///
/// This replaced a `window.open(url, "_blank")`, which could never work: the
/// navigation carries no `Authorization` header and `/laporan/cetak` is
/// `Claims`-guarded, so both buttons 401'd before the query string was even
/// parsed. `laporan_kebutuhan_bmn.rs:59` had already documented exactly this
/// for its own export — the pattern is copied from there rather than invented.
#[cfg(target_arch = "wasm32")]
fn download_export(jenis_laporan: &'static str, jenis_file: &'static str, query: LaporanQuery) {
    use leptos::task::spawn_local;
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, Url};

    let Some(pengajuan_id) = query.pengajuan_id.clone() else {
        return;
    };
    spawn_local(async move {
        let bytes =
            match export_laporan_pakaian_dinas(jenis_laporan, jenis_file, &pengajuan_id, &query)
                .await
            {
                Ok(b) => b,
                Err(e) => {
                    leptos::logging::error!("export {jenis_file} gagal: {e}");
                    return;
                }
            };
        let (mime, ext) = if jenis_file == "pdf" {
            ("application/pdf", "pdf")
        } else {
            (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                "xlsx",
            )
        };
        let array = js_sys::Uint8Array::from(&bytes[..]);
        let parts = js_sys::Array::new();
        parts.push(&array);
        let opts = BlobPropertyBag::new();
        opts.set_type(mime);
        let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &opts) else {
            return;
        };
        let Ok(url) = Url::create_object_url_with_blob(&blob) else {
            return;
        };
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(a) = doc.create_element("a") {
                let a: web_sys::HtmlAnchorElement = a.unchecked_into();
                a.set_href(&url);
                let nama = if jenis_laporan == "daftar" {
                    "Laporan_Daftar"
                } else {
                    "Laporan_Rekap"
                };
                a.set_download(&format!("{}.{}", nama, ext));
                a.click();
            }
        }
        let _ = Url::revoke_object_url(&url);
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn download_export(_jenis_laporan: &'static str, _jenis_file: &'static str, _query: LaporanQuery) {}

/// Helper to render a filter select with dark-theme styling.
fn filter_select(
    label: &'static str,
    on_change: impl Fn(Option<String>) + 'static,
    children: impl IntoView,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1.5 block text-xs font-medium text-slate-400">{label}</label>
            <select
                class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200"
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    on_change(if val.is_empty() { None } else { Some(val) });
                }
            >
                {children}
            </select>
        </div>
    }
}

/// Shown in place of both report tabs while no pengajuan is selected.
///
/// The reports are per-pengajuan by definition — the backend rejects a
/// request without one — so there is no meaningful "all pengajuan" view to
/// render here.
fn pilih_pengajuan_prompt() -> impl IntoView {
    view! {
        <EmptyState
            icon="fas fa-filter"
            title="Pilih Pengajuan"
            description="Laporan rekap ukuran dan daftar pegawai dihitung per pengajuan. Pilih pengajuan pada filter di atas untuk menampilkan datanya."
        />
    }
}

#[component]
pub fn PakaianDinasLaporan() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("rekap");
    let (selected_pengajuan, set_selected_pengajuan) = signal(Option::<String>::None);
    let (selected_satker, set_selected_satker) = signal(Option::<String>::None);
    let (selected_jenis, set_selected_jenis) = signal(Option::<String>::None);
    let (selected_jenis_kelamin, set_selected_jenis_kelamin) = signal(Option::<String>::None);
    let (selected_jenis_pegawai, set_selected_jenis_pegawai) = signal(Option::<String>::None);
    let (selected_eselon, set_selected_eselon) = signal(Option::<String>::None);

    // `daftar_page` lives in the parent so each filter's `on:change` handler
    // can atomically reset it to 1 alongside the filter change. Keeping page
    // local to `DaftarPegawaiTab` would force a separate `Effect` to watch
    // the filter signals and reset page after the fact — which makes the
    // leptos-fetch keyer evaluate twice in quick succession (first with
    // `(new_query, OLD_page)`, then with `(new_query, 1)`), allocating two
    // cache slots and issuing a wasted network request for a page the user
    // never sees. Mirrors the pattern called out in
    // `kebutuhan_bmn_list.rs:91-97`.
    let (daftar_page, set_daftar_page) = signal(1i32);

    // Fetch filter options through the leptos-fetch cache so other
    // pages that reuse the same dropdowns share the result.
    let client: QueryClient = expect_context();
    let pengajuan_options = client.local_resource(query_pengajuan_options, || ());
    let jenis_options = client.local_resource(query_jenis_options, || ());
    // Re-fetches whenever the period changes, and clearing the period empties
    // it — a satker filter left over from another campaign would silently
    // narrow the next report to nothing.
    let satker_options =
        client.local_resource(query_satker_options, move || selected_pengajuan.get());

    let tab_class = move |tab: &'static str| {
        let active = active_tab.get() == tab;
        if active {
            "flex items-center gap-2 border-b-2 border-gold-400 px-4 py-2.5 text-sm font-semibold text-gold-400"
        } else {
            "flex items-center gap-2 border-b-2 border-transparent px-4 py-2.5 text-sm font-medium text-slate-400 transition hover:text-slate-200"
        }
    };

    view! {
        <PageLayout
            title="Laporan Pakaian Dinas"
            icon="fas fa-chart-pie"
            description="Rekap dan daftar pegawai berdasarkan ukuran pakaian dinas"
        >
            // Tabs
            <div class="mb-5 flex border-b border-white/[0.06]">
                <button
                    data-testid="laporan-tab-rekap"
                    class=move || tab_class("rekap")
                    on:click=move |_| set_active_tab.set("rekap")
                >
                    <span class="text-xs">
                        <AppIcon icon=CHART_PIE />
                    </span>
                    "Rekap Ukuran"
                </button>
                <button
                    data-testid="laporan-tab-pegawai"
                    class=move || tab_class("pegawai")
                    on:click=move |_| set_active_tab.set("pegawai")
                >
                    <span class="text-xs">
                        <AppIcon icon=USERS />
                    </span>
                    "Daftar Pegawai"
                </button>
            </div>

            // Filters
            <SectionCard title="Filter Laporan" dense=true>
                <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                    // Pengajuan filter
                    <div>
                        <label class="mb-1.5 block text-xs font-medium text-slate-400">
                            "Periode Pengajuan"
                        </label>
                        <Suspense fallback=move || {
                            view! {
                                <select class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-500">
                                    <option>"Memuat..."</option>
                                </select>
                            }
                        }>
                            {move || match pengajuan_options.get() {
                                Some(Ok(options)) => {
                                    view! {
                                        <select
                                            data-testid="laporan-pengajuan"
                                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_selected_pengajuan
                                                    .set(if val.is_empty() { None } else { Some(val) });
                                                set_daftar_page.set(1);
                                            }
                                        >
                                            // Not "Semua Periode": every report endpoint here
                                            // requires a `pengajuan_id`, so an all-periods view
                                            // does not exist to be offered.
                                            <option value="">"— Pilih Periode —"</option>
                                            {options
                                                .into_iter()
                                                .map(|p| {
                                                    let label = format!("{} ({})", p.nama, p.tahun);
                                                    view! { <option value=p.id>{label}</option> }
                                                })
                                                .collect_view()}
                                        </select>
                                    }
                                        .into_any()
                                }
                                // Still "Semua Periode" here until the contract
                                // drift above was found: the success arm had
                                // dropped that option, but this arm was the one
                                // that actually rendered, so the page really did
                                // offer a view no endpoint can serve.
                                _ => {
                                    view! {
                                        <select
                                            data-testid="laporan-pengajuan"
                                            disabled=true
                                            class="focus-ring w-full rounded-lg border border-danger-500/30 bg-danger-500/5 px-3 py-2 text-sm text-danger-300"
                                        >
                                            <option>"Gagal memuat periode"</option>
                                        </select>
                                    }
                                        .into_any()
                                }
                            }}
                        </Suspense>
                    </div>

                    // Jenis Pakaian filter
                    <div>
                        <label class="mb-1.5 block text-xs font-medium text-slate-400">
                            "Jenis Pakaian"
                        </label>
                        <Suspense fallback=move || {
                            view! {
                                <select class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-500">
                                    <option>"Memuat..."</option>
                                </select>
                            }
                        }>
                            {move || match jenis_options.get() {
                                Some(Ok(options)) => {
                                    view! {
                                        <select
                                            data-testid="laporan-jenis"
                                            class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200"
                                            on:change=move |ev| {
                                                let val = event_target_value(&ev);
                                                set_selected_jenis
                                                    .set(if val.is_empty() { None } else { Some(val) });
                                                set_daftar_page.set(1);
                                            }
                                        >
                                            <option value="">"Semua Jenis"</option>
                                            {options
                                                .into_iter()
                                                .map(|j| {
                                                    view! { <option value=j.id>{j.nama}</option> }
                                                })
                                                .collect_view()}
                                        </select>
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! {
                                        <select class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-500">
                                            <option>"Semua Jenis"</option>
                                        </select>
                                    }
                                        .into_any()
                                }
                            }}
                        </Suspense>
                    </div>

                    // Satker filter
                    <div>
                        <label class="mb-1.5 block text-xs font-medium text-slate-400">
                            "Satker"
                        </label>
                        // Picked from the campaign's own satker list, not typed.
                        // It used to be a free-text box asking for a `kode_satker`
                        // — a code nobody carries in their head, where one wrong
                        // digit returns an empty report that looks exactly like a
                        // satker with nothing to report.
                        <Suspense fallback=move || {
                            view! {
                                <select class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-500">
                                    <option>"Memuat..."</option>
                                </select>
                            }
                        }>
                            {move || {
                                let opsi = match satker_options.get() {
                                    Some(Ok(v)) => v,
                                    _ => Vec::new(),
                                };
                                let belum_pilih_periode = selected_pengajuan.get().is_none();
                                view! {
                                    <select
                                        data-testid="laporan-satker"
                                        class="focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2 text-sm text-slate-200 disabled:text-slate-500"
                                        prop:disabled=belum_pilih_periode
                                        on:change=move |ev| {
                                            let val = event_target_value(&ev);
                                            set_selected_satker
                                                .set(if val.is_empty() { None } else { Some(val) });
                                            set_daftar_page.set(1);
                                        }
                                    >
                                        <option value="">
                                            {if belum_pilih_periode {
                                                "Pilih periode dulu"
                                            } else {
                                                "Semua Satker"
                                            }}
                                        </option>
                                        {opsi
                                            .into_iter()
                                            .map(|s| {
                                                // `satker_id` IS the MySIMKARI `kode_satker`
                                                // the filter expects; `satker_kode` is the
                                                // same value joined back for display, and is
                                                // not always present.
                                                let nilai = s.satker_id.clone();
                                                let label = s
                                                    .satker_nama
                                                    .clone()
                                                    .unwrap_or_else(|| nilai.clone());
                                                view! { <option value=nilai>{label}</option> }
                                            })
                                            .collect_view()}
                                    </select>
                                }
                            }}
                        </Suspense>
                    </div>

                    // Jenis Kelamin
                    {filter_select(
                        "Jenis Kelamin",
                        move |v| {
                            set_selected_jenis_kelamin.set(v);
                            set_daftar_page.set(1);
                        },
                        view! {
                            <option value="">"Semua"</option>
                            <option value="L">"Laki-laki"</option>
                            <option value="P">"Perempuan"</option>
                        },
                    )}

                    // Jenis Pegawai
                    {filter_select(
                        "Jenis Pegawai",
                        move |v| {
                            set_selected_jenis_pegawai.set(v);
                            set_daftar_page.set(1);
                        },
                        view! {
                            <option value="">"Semua"</option>
                            <option value="0">"Jaksa"</option>
                            <option value="1">"Tata Usaha"</option>
                        },
                    )}

                    // Eselon
                    {filter_select(
                        "Eselon",
                        move |v| {
                            set_selected_eselon.set(v);
                            set_daftar_page.set(1);
                        },
                        view! {
                            <option value="">"Semua Eselon"</option>
                            <option value="I">"Eselon I"</option>
                            <option value="II">"Eselon II"</option>
                            <option value="III">"Eselon III"</option>
                            <option value="IV">"Eselon IV"</option>
                        },
                    )}
                </div>
            </SectionCard>

            // Export buttons
            //
            // `/laporan/cetak` takes the same required `pengajuan_id` as the
            // two tab endpoints (`CetakQuery`), so the buttons stay disabled
            // until one is picked — otherwise the only possible outcome is a
            // 400 the user never sees, since the download happens in the
            // background.
            <div class="mt-4 flex flex-wrap gap-3">
                <button
                    data-testid="laporan-cetak-excel"
                    class="inline-flex items-center gap-2 rounded-lg border border-success-500/30 bg-success-500/10 px-4 py-2.5 text-sm font-medium text-success-300 transition hover:bg-success-500/20 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-success-500/10"
                    disabled=move || selected_pengajuan.get().is_none()
                    on:click=move |_| {
                        download_export(
                            jenis_laporan_for_tab(active_tab.get()),
                            "excel",
                            LaporanQuery {
                                pengajuan_id: selected_pengajuan.get(),
                                satker_id: selected_satker.get(),
                                jenis_pakaian_id: selected_jenis.get(),
                                jenis_kelamin: selected_jenis_kelamin.get(),
                                eselon: selected_eselon.get(),
                                jenis: selected_jenis_pegawai.get(),
                            },
                        );
                    }
                >
                    <span class="text-xs">
                        <AppIcon icon=FILE_XLS />
                    </span>
                    "Cetak Excel"
                </button>
                <button
                    data-testid="laporan-cetak-pdf"
                    class="inline-flex items-center gap-2 rounded-lg border border-danger-500/30 bg-danger-500/10 px-4 py-2.5 text-sm font-medium text-danger-300 transition hover:bg-danger-500/20 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:bg-danger-500/10"
                    disabled=move || selected_pengajuan.get().is_none()
                    on:click=move |_| {
                        download_export(
                            jenis_laporan_for_tab(active_tab.get()),
                            "pdf",
                            LaporanQuery {
                                pengajuan_id: selected_pengajuan.get(),
                                satker_id: selected_satker.get(),
                                jenis_pakaian_id: selected_jenis.get(),
                                jenis_kelamin: selected_jenis_kelamin.get(),
                                eselon: selected_eselon.get(),
                                jenis: selected_jenis_pegawai.get(),
                            },
                        );
                    }
                >
                    <span class="text-xs">
                        <AppIcon icon=FILE_PDF />
                    </span>
                    "Cetak PDF"
                </button>
            </div>

            // Tab Content
            //
            // Both report endpoints take `pengajuan_id` as a REQUIRED `Uuid`
            // (`LaporanRekapQuery`/`LaporanDaftarQuery` in
            // `layanan/perlengkapan/src/pakaian_dinas/handlers.rs`), so the
            // outer `<Show>` is what keeps the tab from mounting — and
            // therefore from fetching — before one is picked. Gating here
            // rather than inside the tab matters: a `local_resource` starts
            // its fetch as soon as the component is built, so a guard placed
            // around the `<Suspense>` would still fire the request.
            <div class="mt-4">
                <Show when=move || selected_pengajuan.get().is_some() fallback=pilih_pengajuan_prompt>
                    // What the numbers below actually cover.
                    //
                    // Every laporan query counts only satker that reached
                    // `Selesai` — deliberately, since a recap is a procurement
                    // figure and half-revised sizes should not be ordered
                    // against. But the filter was invisible: mid-campaign the
                    // report just came back small, and a recap over 3 of 238
                    // satker rendered identically to a complete one.
                    //
                    // Both counts were already on the campaign this page has
                    // fetched for its own dropdown, computed with the same
                    // predicate. They were simply never shown.
                    {move || {
                        let terpilih = selected_pengajuan.get()?;
                        let daftar = match pengajuan_options.get() {
                            Some(Ok(v)) => v,
                            _ => return None,
                        };
                        let p = daftar.into_iter().find(|p| p.id == terpilih)?;
                        let selesai = p.satker_selesai.unwrap_or(0);
                        let total = p.total_satker.unwrap_or(0);
                        let lengkap = total > 0 && selesai == total;
                        // Three states, because two of them collapse into a
                        // sentence that is not true. A campaign with no
                        // participating satker at all is not "0 of 0 finished,
                        // so the numbers are not final" — there is nothing to
                        // finish and nothing to report. Staging carries exactly
                        // such a campaign, which is how this was caught.
                        let (kelas, teks) = if total == 0 {
                            (
                                "border-white/10 bg-white/[0.04] text-slate-400",
                                "Belum ada satker yang ditetapkan untuk pengajuan ini, jadi belum ada yang bisa dilaporkan."
                                    .to_string(),
                            )
                        } else if lengkap {
                            (
                                "border-success-500/20 bg-success-500/10 text-success-300",
                                format!(
                                    "Seluruh {total} satker sudah Selesai — angka di bawah mencakup pengajuan ini sepenuhnya.",
                                ),
                            )
                        } else {
                            (
                                "border-warning-500/20 bg-warning-500/10 text-warning-300",
                                format!(
                                    // `\` continuations, not a wrapped literal: joining the
                                    // lines the other way leaves the indentation INSIDE the
                                    // string, and HTML collapsing it is what hides that from
                                    // the screen while every other reader keeps the gap.
                                    "Mencakup {selesai} dari {total} satker yang sudah Selesai. \
                                     Satker yang masih diisi atau menunggu validasi belum \
                                     terhitung, jadi angka di bawah belum final.",
                                ),
                            )
                        };
                        Some(
                            view! {
                                <p
                                    data-testid="laporan-cakupan"
                                    data-lengkap=lengkap.to_string()
                                    class=format!(
                                        "mb-4 rounded-lg border px-3 py-2 text-sm {kelas}",
                                    )
                                >
                                    {teks}
                                </p>
                            },
                        )
                    }}
                    <Show when=move || active_tab.get() == "rekap">
                        <RekapUkuranTab
                            pengajuan_id=selected_pengajuan
                            satker_id=selected_satker
                            jenis_pakaian_id=selected_jenis
                            jenis_kelamin=selected_jenis_kelamin
                            jenis=selected_jenis_pegawai
                            eselon=selected_eselon
                        />
                    </Show>
                    <Show when=move || active_tab.get() == "pegawai">
                        <DaftarPegawaiTab
                            pengajuan_id=selected_pengajuan
                            satker_id=selected_satker
                            jenis_pakaian_id=selected_jenis
                            jenis_kelamin=selected_jenis_kelamin
                            jenis=selected_jenis_pegawai
                            eselon=selected_eselon
                            page=daftar_page
                            set_page=set_daftar_page
                        />
                    </Show>
                </Show>
            </div>
        </PageLayout>
    }
}

// ── Rekap Ukuran Tab ──────────────────────────────────────────────────────

/// Maps ukuran_group to accent class tokens.
fn group_accent(ukuran_group: &str) -> (&'static str, &'static str) {
    match ukuran_group {
        "BAJU" => ("info", "fas fa-tshirt"),
        "CELANA" => ("success", "fas fa-male"),
        "SEPATU" => ("gold", "fas fa-shoe-prints"),
        _ => ("slate", "fas fa-box"),
    }
}

#[component]
fn RekapUkuranTab(
    pengajuan_id: ReadSignal<Option<String>>,
    satker_id: ReadSignal<Option<String>>,
    jenis_pakaian_id: ReadSignal<Option<String>>,
    jenis_kelamin: ReadSignal<Option<String>>,
    jenis: ReadSignal<Option<String>>,
    eselon: ReadSignal<Option<String>>,
) -> impl IntoView {
    let client: QueryClient = expect_context();
    // Read filter signals reactively inside the keyer — `<Show>` keeps this
    // component mounted across filter dropdown changes, so a one-shot
    // `.get().clone()` capture would freeze the cache key at the value the
    // filters held when the tab was first activated.
    let data = client.local_resource(query_laporan_rekap, move || LaporanQuery {
        pengajuan_id: pengajuan_id.get(),
        satker_id: satker_id.get(),
        jenis_pakaian_id: jenis_pakaian_id.get(),
        jenis_kelamin: jenis_kelamin.get(),
        eselon: eselon.get(),
        jenis: jenis.get(),
    });

    view! {
        <Suspense fallback=move || {
            view! { <LoadingState message="Memuat data rekap...".to_string() /> }
        }>
            {move || match data.get() {
                None => view! { <LoadingState /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(rekap)) => {
                    if rekap.is_empty() {
                        // An empty result and an unselected pengajuan look
                        // identical here, but they are not the same thing:
                        // without a pengajuan nothing was ever asked for.
                        let unselected = pengajuan_id.get().is_none();
                        view! {
                            <EmptyState
                                icon="fas fa-chart-bar"
                                title=if unselected { "Pilih Pengajuan" } else { "Tidak Ada Data" }
                                description=if unselected {
                                    "Pilih pengajuan pakaian dinas terlebih dahulu untuk menampilkan rekap ukuran."
                                } else {
                                    "Tidak ada data untuk filter yang dipilih."
                                }
                            />
                        }
                            .into_any()
                    } else {
                        render_rekap_groups(rekap)
                    }
                }
            }}
        </Suspense>
    }
}

/// Group rekap items by pakaian_nama and render a table per group.
fn render_rekap_groups(rekap: Vec<LaporanRekapUkuran>) -> AnyView {
    let mut groups: Vec<String> = vec![];
    for r in &rekap {
        if !groups.contains(&r.pakaian_nama) {
            groups.push(r.pakaian_nama.clone());
        }
    }

    view! {
        <div class="flex flex-col gap-5">
            {groups
                .into_iter()
                .map(|group_name| {
                    let items: Vec<LaporanRekapUkuran> = rekap
                        .iter()
                        .filter(|r| r.pakaian_nama == group_name)
                        .cloned()
                        .collect();
                    render_rekap_table(group_name, items)
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

/// Render a single rekap table for one pakaian group.
fn render_rekap_table(group_name: String, items: Vec<LaporanRekapUkuran>) -> impl IntoView {
    let ukuran_group = items
        .first()
        .map(|r| r.ukuran_group.clone())
        .unwrap_or_default();
    let (accent, icon) = group_accent(&ukuran_group);
    let total_l: i64 = items.iter().map(|r| r.jumlah_laki).sum();
    let total_p: i64 = items.iter().map(|r| r.jumlah_perempuan).sum();
    let total_all: i64 = items.iter().map(|r| r.jumlah_total).sum();

    let header_bg = format!("bg-{}-500/10", accent);
    let header_text = format!("text-{}-400", accent);
    let ring = format!("ring-{}-500/25", accent);

    view! {
        <div
            data-testid="laporan-rekap-group"
            class=format!(
                "overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel ring-1 {}",
                ring,
            )
        >
            // Group header
            <div class=format!("flex items-center gap-3 px-5 py-3 {}", header_bg)>
                <span class=format!("inline-flex {}", header_text)>
                    <AppIcon icon=icon_from_fa_class(icon) size=14 />
                </span>
                <h4 class=format!(
                    "text-sm font-semibold {}",
                    header_text,
                )>{format!("{} ({})", group_name, ukuran_group)}</h4>
            </div>

            <div class="overflow-x-auto">
                <table class="min-w-full text-sm">
                    <thead>
                        <tr class="border-b border-white/[0.06] bg-white/[0.02]">
                            <th class="px-4 py-2.5 text-left text-xs font-semibold uppercase text-slate-400">
                                "Gender"
                            </th>
                            {items
                                .iter()
                                .map(|r| {
                                    let ukuran = r.ukuran.clone();
                                    view! {
                                        <th class="px-3 py-2.5 text-center text-xs font-semibold uppercase text-slate-400">
                                            {ukuran}
                                        </th>
                                    }
                                })
                                .collect_view()}
                            <th class="px-4 py-2.5 text-center text-xs font-bold uppercase text-slate-300">
                                "Total"
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        // Laki-laki row
                        <tr class="border-b border-white/[0.04]">
                            <td class="px-4 py-2.5 text-sm font-medium text-slate-200">
                                "Laki-laki (L)"
                            </td>
                            {items
                                .iter()
                                .map(|r| {
                                    let v = r.jumlah_laki.to_string();
                                    view! {
                                        <td class="px-3 py-2.5 text-center text-slate-300">{v}</td>
                                    }
                                })
                                .collect_view()}
                            <td class="px-4 py-2.5 text-center font-bold text-slate-100">
                                {total_l.to_string()}
                            </td>
                        </tr>
                        // Perempuan row
                        <tr class="border-b border-white/[0.04]">
                            <td class="px-4 py-2.5 text-sm font-medium text-slate-200">
                                "Perempuan (P)"
                            </td>
                            {items
                                .iter()
                                .map(|r| {
                                    let v = r.jumlah_perempuan.to_string();
                                    view! {
                                        <td class="px-3 py-2.5 text-center text-slate-300">{v}</td>
                                    }
                                })
                                .collect_view()}
                            <td class="px-4 py-2.5 text-center font-bold text-slate-100">
                                {total_p.to_string()}
                            </td>
                        </tr>
                        // Total row
                        <tr class="bg-white/[0.03]">
                            <td class="px-4 py-2.5 text-sm font-bold text-slate-100">"Jumlah"</td>
                            {items
                                .iter()
                                .map(|r| {
                                    let v = r.jumlah_total.to_string();
                                    view! {
                                        <td class="px-3 py-2.5 text-center font-bold text-slate-100">
                                            {v}
                                        </td>
                                    }
                                })
                                .collect_view()}
                            <td class="px-4 py-2.5 text-center font-bold text-gold-400">
                                {total_all.to_string()}
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ── Daftar Pegawai Tab ────────────────────────────────────────────────────

#[component]
fn DaftarPegawaiTab(
    pengajuan_id: ReadSignal<Option<String>>,
    satker_id: ReadSignal<Option<String>>,
    jenis_pakaian_id: ReadSignal<Option<String>>,
    jenis_kelamin: ReadSignal<Option<String>>,
    jenis: ReadSignal<Option<String>>,
    eselon: ReadSignal<Option<String>>,
    /// Pagination state lifted into the parent so each filter's
    /// `on:change` handler can reset page to 1 atomically with the
    /// filter change. Doing the reset in a local `Effect` here would
    /// cause the leptos-fetch keyer to evaluate twice on every filter
    /// change — first with `(new_query, OLD_page)`, then with
    /// `(new_query, 1)` — allocating two cache slots and issuing a
    /// wasted network request for a page the user never sees.
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
) -> impl IntoView {
    let client: QueryClient = expect_context();
    // Filter signals are read reactively inside the keyer so dropdown
    // changes while the tab is mounted re-key the leptos-fetch resource.
    // See the matching comment in `RekapUkuranTab` for the rationale.
    let data = client.local_resource(query_laporan_daftar, move || {
        let query = LaporanQuery {
            pengajuan_id: pengajuan_id.get(),
            satker_id: satker_id.get(),
            jenis_pakaian_id: jenis_pakaian_id.get(),
            jenis_kelamin: jenis_kelamin.get(),
            eselon: eselon.get(),
            jenis: jenis.get(),
        };
        (query, page.get())
    });

    view! {
        <Suspense fallback=move || {
            view! { <LoadingState message="Memuat daftar pegawai...".to_string() /> }
        }>
            {move || match data.get() {
                None => view! { <LoadingState /> }.into_any(),
                Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                Some(Ok(response)) => {
                    if response.data.is_empty() {
                        // An empty result and an unselected pengajuan look
                        // identical here, but they are not the same thing:
                        // without a pengajuan nothing was ever asked for.
                        let unselected = pengajuan_id.get().is_none();
                        view! {
                            <EmptyState
                                icon="fas fa-users"
                                title=if unselected { "Pilih Pengajuan" } else { "Tidak Ada Data" }
                                description=if unselected {
                                    "Pilih pengajuan pakaian dinas terlebih dahulu untuk menampilkan daftar pegawai."
                                } else {
                                    "Tidak ada data untuk filter yang dipilih."
                                }
                            />
                        }
                            .into_any()
                    } else {
                        render_pegawai_table(
                            response.data,
                            response.page,
                            response.total,
                            response.total_pages,
                            page,
                            set_page,
                        )
                    }
                }
            }}
        </Suspense>
    }
}

/// Renders the pegawai list table matching legacy cetakDaftarTemplateV.
fn render_pegawai_table(
    data: Vec<LaporanDaftarPegawai>,
    current_page: i32,
    total: i64,
    total_pages: i32,
    page: ReadSignal<i32>,
    set_page: WriteSignal<i32>,
) -> AnyView {
    let headers = [
        "No", "NIP", "Nama", "Jabatan", "Gol", "Status", "Gender", "Hijab", "Baju", "Celana",
        "Sepatu",
    ];

    view! {
        <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
            <div class="overflow-x-auto">
                <table class="min-w-full divide-y divide-white/[0.04]">
                    <thead class="bg-white/[0.02]">
                        <tr>
                            {headers
                                .iter()
                                .map(|h| {
                                    view! {
                                        <th class="px-3 py-3 text-xs font-semibold uppercase tracking-wide text-slate-400">
                                            {*h}
                                        </th>
                                    }
                                })
                                .collect_view()}
                        </tr>
                    </thead>
                    <tbody>
                        {data
                            .into_iter()
                            .enumerate()
                            .map(|(idx, item)| {
                                let status_label = match item.jenis.as_deref() {
                                    Some("0") => "J",
                                    Some("1") => "T",
                                    _ => "-",
                                };
                                let hijab_label = match item.with_hijab {
                                    Some(true) => "Y",
                                    Some(false) => "T",
                                    None => "-",
                                };
                                let num = ((page.get() - 1) * 20 + idx as i32 + 1).to_string();
                                let bg = if idx % 2 == 0 {
                                    "bg-transparent"
                                } else {
                                    "bg-white/[0.015]"
                                };
                                view! {
                                    <tr class=format!("border-b border-white/[0.04] {}", bg)>
                                        <td class="px-3 py-2.5 text-sm text-slate-400">{num}</td>
                                        <td class="px-3 py-2.5 font-mono text-xs text-slate-300">
                                            {item.nip}
                                        </td>
                                        <td class="px-3 py-2.5 text-sm font-medium text-slate-100">
                                            {item.nama}
                                        </td>
                                        <td class="px-3 py-2.5 text-sm text-slate-400">
                                            {item.jabatan.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-xs text-slate-400">
                                            {item.gol_kd.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-xs text-slate-300">
                                            {status_label}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-xs text-slate-300">
                                            {item.jenis_kelamin.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-xs text-slate-300">
                                            {hijab_label}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-sm font-semibold text-slate-200">
                                            {item.ukuran_baju.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-sm font-semibold text-slate-200">
                                            {item.ukuran_celana.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                        <td class="px-3 py-2.5 text-center text-sm font-semibold text-slate-200">
                                            {item.ukuran_sepatu.unwrap_or_else(|| "-".to_string())}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>

            // Pagination
            <div class="flex items-center justify-between border-t border-white/[0.04] px-5 py-3">
                <p class="text-xs text-slate-400">
                    "Halaman " <span class="font-medium text-slate-200">{current_page}</span>
                    " dari " <span class="font-medium text-slate-200">{total_pages}</span> " ("
                    <span class="font-medium text-slate-200">{total}</span> " pegawai)"
                </p>
                <div class="flex gap-2">
                    <button
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                        prop:disabled=move || page.get() <= 1
                        on:click=move |_| set_page.update(|p| *p -= 1)
                    >
                        "Sebelumnya"
                    </button>
                    <button
                        class="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs text-slate-300 transition hover:bg-white/[0.08] disabled:opacity-40"
                        prop:disabled=move || { page.get() >= total_pages }
                        on:click=move |_| set_page.update(|p| *p += 1)
                    >
                        "Selanjutnya"
                    </button>
                </div>
            </div>
        </div>
    }.into_any()
}
