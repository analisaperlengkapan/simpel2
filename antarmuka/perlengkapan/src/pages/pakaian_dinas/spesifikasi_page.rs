//! Spesifikasi & subspesifikasi for one jenis pakaian dinas.
//!
//! This page used to only LIST. That was a dead end, and a measurable one: a
//! campaign cannot be created without at least one spesifikasi
//! (`pakaian_dinas_pengajuan_list.rs` refuses an empty selection), and nothing
//! anywhere in the frontend could create one — the backend's `POST/PUT/DELETE
//! /pakaian-dinas/spesifikasi` and `/subspesifikasi` had zero callers. Staging
//! showed the consequence: 8 jenis, 0 spesifikasi, 0 subspesifikasi, so the
//! whole pakaian dinas workflow could not be started by anyone.
//!
//! Both levels are admin-only on the server (`require_admin`), so the controls
//! are hidden from everyone else rather than offered and then refused.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_fetch::QueryClient;
use leptos_router::hooks::use_params_map;

use crate::api::PaginatedResponse;
use crate::api::pakaian_dinas::{
    CreateSpesifikasiRequest, CreateSubSpesifikasiRequest, SpesifikasiPakaianDinas,
    SubSpesifikasiPakaianDinas, create_spesifikasi_pakaian, create_subspesifikasi,
    delete_spesifikasi_pakaian, delete_subspesifikasi, fetch_spesifikasi_pakaian,
    fetch_subspesifikasi,
};
use crate::components::layout::{
    EmptyState, ErrorState, FormField, LoadingState, PageLayout, SectionCard,
};
use crate::features::auth::AuthService;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CARET_DOWN, CARET_RIGHT, LIST, PLUS, TRASH, WARNING_CIRCLE};

/// The three size families the server accepts (`services.rs` rejects anything
/// else with 400), so they are offered rather than typed.
const UKURAN_GROUPS: [(&str, &str); 3] =
    [("BAJU", "Baju"), ("CELANA", "Celana"), ("SEPATU", "Sepatu")];

/// Likewise for gender. `SEMUA` means the item applies to everyone; `L`/`P`
/// narrow it, which is what lets the size form show a person only the items
/// that concern them.
const GENDERS: [(&str, &str); 3] = [("SEMUA", "Semua"), ("L", "Laki-laki"), ("P", "Perempuan")];

const FIELD: &str = "focus-ring w-full rounded-lg border border-white/10 bg-white/[0.04] \
                     px-3.5 py-2.5 text-sm text-slate-100 placeholder-slate-500";

fn label_of(table: &[(&str, &str)], code: &str) -> String {
    table
        .iter()
        .find(|(k, _)| *k == code)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| code.to_string())
}

async fn query_spesifikasi(
    key: (String, i32),
) -> Result<PaginatedResponse<SpesifikasiPakaianDinas>, crate::api::AppError> {
    let (jenis_id, _trigger) = key;
    fetch_spesifikasi_pakaian(1, 100, Some(jenis_id)).await
}

async fn query_subspesifikasi(
    key: (String, i32),
) -> Result<PaginatedResponse<SubSpesifikasiPakaianDinas>, crate::api::AppError> {
    let (spesifikasi_id, _trigger) = key;
    fetch_subspesifikasi(Some(spesifikasi_id)).await
}

#[component]
pub fn SpesifikasiPage() -> impl IntoView {
    let params = use_params_map();
    let jenis_id = move || params.with(|p| p.get("id").unwrap_or_default().to_string());

    let is_admin = AuthService::load_session()
        .map(|s| s.is_admin())
        .unwrap_or(false);
    let refresh = RwSignal::new(0);
    let (show_form, set_show_form) = signal(false);
    let (form_nama, set_form_nama) = signal(String::new());
    let form_group = RwSignal::new("BAJU".to_string());
    let form_gender = RwSignal::new("SEMUA".to_string());
    let (form_deskripsi, set_form_deskripsi) = signal(String::new());
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    // Only one row's subspesifikasi is expanded at a time; the table stays
    // readable and only the open row fetches.
    let terbuka = RwSignal::new(Option::<String>::None);

    let client: QueryClient = expect_context();
    let data = client.local_resource(query_spesifikasi, move || (jenis_id(), refresh.get()));

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        set_error.set(None);
        let request = CreateSpesifikasiRequest {
            jenis_pakaian_dinas_id: jenis_id(),
            nama: form_nama.get(),
            gender: form_gender.get(),
            ukuran_group: form_group.get(),
            deskripsi: {
                let d = form_deskripsi.get();
                if d.trim().is_empty() { None } else { Some(d) }
            },
            is_active: true,
        };
        spawn_local(async move {
            match create_spesifikasi_pakaian(request).await {
                Ok(_) => {
                    set_form_nama.set(String::new());
                    set_form_deskripsi.set(String::new());
                    set_show_form.set(false);
                    refresh.update(|v| *v += 1);
                }
                Err(e) => set_error.set(Some(format!("Gagal menyimpan: {e}"))),
            }
            set_saving.set(false);
        });
    };

    view! {
        <PageLayout
            title="Spesifikasi Pakaian Dinas"
            icon="fas fa-list"
            description="Rincian pakaian yang diminta pada satu jenis — tiap baris menjadi satu kolom ukuran saat satker mengisi."
        >
            <Show when=move || is_admin>
                <div class="mb-5 flex justify-end">
                    <button
                        type="button"
                        data-testid="tambah-spesifikasi"
                        class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-4 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90"
                        on:click=move |_| set_show_form.update(|v| *v = !*v)
                    >
                        <AppIcon icon=PLUS />
                        "Tambah Spesifikasi"
                    </button>
                </div>
            </Show>

            <Show when=move || show_form.get() && is_admin>
                <SectionCard title="Tambah Spesifikasi">
                    <Show when=move || error.get().is_some()>
                        <div class="mb-4 flex items-center gap-2 rounded-xl border border-danger-500/30 bg-danger-500/[0.08] px-4 py-3 text-sm text-danger-300">
                            <AppIcon icon=WARNING_CIRCLE />
                            {move || error.get()}
                        </div>
                    </Show>
                    <form on:submit=on_submit>
                        <div class="grid grid-cols-1 gap-5 sm:grid-cols-2">
                            <FormField label="Nama Spesifikasi" for_id="spek-nama" required=true>
                                <input
                                    id="spek-nama"
                                    type="text"
                                    class=FIELD
                                    placeholder="Contoh: Pakaian Dinas, Celana, Sepatu Dinas"
                                    prop:value=move || form_nama.get()
                                    on:input=move |ev| set_form_nama.set(event_target_value(&ev))
                                    required=true
                                />
                            </FormField>
                            <FormField
                                label="Grup Ukuran"
                                for_id="spek-grup"
                                helper="Menentukan daftar ukuran yang ditawarkan saat pengisian."
                                required=true
                            >
                                <select
                                    id="spek-grup"
                                    class=FIELD
                                    prop:value=move || form_group.get()
                                    on:change=move |ev| form_group.set(event_target_value(&ev))
                                >
                                    {UKURAN_GROUPS
                                        .iter()
                                        .map(|(k, v)| view! { <option value=*k>{*v}</option> })
                                        .collect_view()}
                                </select>
                            </FormField>
                            <FormField
                                label="Gender"
                                for_id="spek-gender"
                                helper="\"Semua\" berlaku untuk seluruh pegawai; L/P hanya muncul bagi yang sesuai."
                                required=true
                            >
                                <select
                                    id="spek-gender"
                                    class=FIELD
                                    prop:value=move || form_gender.get()
                                    on:change=move |ev| form_gender.set(event_target_value(&ev))
                                >
                                    {GENDERS
                                        .iter()
                                        .map(|(k, v)| view! { <option value=*k>{*v}</option> })
                                        .collect_view()}
                                </select>
                            </FormField>
                            <FormField label="Keterangan" for_id="spek-keterangan">
                                <input
                                    id="spek-keterangan"
                                    type="text"
                                    class=FIELD
                                    placeholder="Opsional"
                                    prop:value=move || form_deskripsi.get()
                                    on:input=move |ev| {
                                        set_form_deskripsi.set(event_target_value(&ev))
                                    }
                                />
                            </FormField>
                        </div>
                        <div class="mt-5 flex gap-3 border-t border-white/[0.04] pt-4">
                            <button
                                type="submit"
                                class="inline-flex items-center gap-2 rounded-lg bg-gold-gradient px-5 py-2.5 text-sm font-bold text-navy-950 shadow-sm transition hover:opacity-90 disabled:opacity-50"
                                prop:disabled=move || saving.get()
                            >
                                {move || if saving.get() { "Menyimpan..." } else { "Simpan" }}
                            </button>
                            <button
                                type="button"
                                class="rounded-lg border border-white/10 bg-white/[0.04] px-5 py-2.5 text-sm text-slate-300 transition hover:bg-white/[0.08]"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Batal"
                            </button>
                        </div>
                    </form>
                </SectionCard>
            </Show>

            <Suspense fallback=move || {
                view! { <LoadingState /> }
            }>
                {move || match data.get() {
                    None => view! { <LoadingState /> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(response)) => {
                        let items = response.data;
                        if items.is_empty() {
                            view! {
                                <EmptyState
                                    icon="fas fa-list"
                                    title="Belum Ada Spesifikasi"
                                    description="Tanpa spesifikasi, pengajuan pakaian dinas tidak dapat dibuat. Tambahkan minimal satu."
                                />
                            }
                                .into_any()
                        } else {
                            view! {
                                <SectionCard title="Daftar Spesifikasi">
                                    <div class="overflow-hidden rounded-2xl border border-white/[0.06] bg-surface-panel">
                                        <div class="overflow-x-auto">
                                            <table
                                                data-testid="spesifikasi-tabel"
                                                class="min-w-full divide-y divide-white/[0.04]"
                                            >
                                                <thead class="bg-white/[0.02]">
                                                    <tr>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                                            "Nama"
                                                        </th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                                            "Grup Ukuran"
                                                        </th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                                            "Gender"
                                                        </th>
                                                        <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-400">
                                                            "Keterangan"
                                                        </th>
                                                        // Kolom aksi hanya ada
                                                        // bila ada aksi: bagi
                                                        // peran baca-saja ia
                                                        // kolom kosong yang
                                                        // tak pernah terisi.
                                                        <Show when=move || is_admin>
                                                            <th class="px-4 py-3 text-right text-xs font-semibold uppercase tracking-wide text-slate-400">
                                                                "Aksi"
                                                            </th>
                                                        </Show>
                                                    </tr>
                                                </thead>
                                                <tbody class="divide-y divide-white/[0.04]">
                                                    {items
                                                        .into_iter()
                                                        .map(|item| {
                                                            baris_spesifikasi(item, is_admin, terbuka, refresh)
                                                        })
                                                        .collect_view()}
                                                </tbody>
                                            </table>
                                        </div>
                                    </div>
                                </SectionCard>
                            }
                                .into_any()
                        }
                    }
                }}
            </Suspense>
        </PageLayout>
    }
}

/// One spesifikasi row, plus the subspesifikasi drawer it opens.
fn baris_spesifikasi(
    item: SpesifikasiPakaianDinas,
    is_admin: bool,
    terbuka: RwSignal<Option<String>>,
    refresh: RwSignal<i32>,
) -> AnyView {
    let id = item.id.clone();
    let id_toggle = id.clone();
    let id_hapus = id.clone();
    // `StoredValue` so the predicate stays `Copy` — it is read by both the
    // caret and the drawer's `Show`, and a captured `String` would move into
    // the first of them.
    let id_sv = StoredValue::new(id.clone());
    let nama = item.nama.clone();
    let is_open =
        move || terbuka.with(|v| id_sv.with_value(|id| v.as_deref() == Some(id.as_str())));

    view! {
        <tr class="transition hover:bg-white/[0.02]">
            <td class="px-4 py-3 text-sm text-slate-100">
                <button
                    type="button"
                    class="inline-flex items-center gap-2 text-left font-medium hover:text-gold-300"
                    on:click=move |_| {
                        terbuka
                            .update(|v| {
                                *v = if v.as_deref() == Some(id_toggle.as_str()) {
                                    None
                                } else {
                                    Some(id_toggle.clone())
                                };
                            })
                    }
                >
                    <span class="text-slate-400">
                        {move || {
                            if is_open() {
                                view! { <AppIcon icon=CARET_DOWN size=14 /> }
                            } else {
                                view! { <AppIcon icon=CARET_RIGHT size=14 /> }
                            }
                        }}
                    </span>
                    {nama}
                </button>
            </td>
            <td class="px-4 py-3 text-sm text-slate-300">
                {label_of(&UKURAN_GROUPS, &item.ukuran_group)}
            </td>
            <td class="px-4 py-3 text-sm text-slate-300">{label_of(&GENDERS, &item.gender)}</td>
            <td class="px-4 py-3 text-sm text-slate-400">
                {item.deskripsi.clone().unwrap_or_else(|| "-".to_string())}
            </td>
            <Show when=move || is_admin>
                <td class="px-4 py-3 text-right">
                    <button
                        type="button"
                        aria-label="Hapus spesifikasi"
                        class="text-danger-400 transition hover:text-danger-300"
                        on:click={
                            let id_hapus = id_hapus.clone();
                            move |_| {
                                let id_hapus = id_hapus.clone();
                                spawn_local(async move {
                                    if delete_spesifikasi_pakaian(id_hapus).await.is_ok() {
                                        refresh.update(|v| *v += 1);
                                    }
                                });
                            }
                        }
                    >
                        <AppIcon icon=TRASH />
                    </button>
                </td>
            </Show>
        </tr>
        <Show when=is_open>
            <tr>
                <td colspan=move || if is_admin { "5" } else { "4" } class="bg-white/[0.015] px-4 py-4">
                    <SubSpesifikasiPanel spesifikasi_id=id.clone() is_admin=is_admin />
                </td>
            </tr>
        </Show>
    }
    .into_any()
}

/// Subspesifikasi under one spesifikasi — the gendered variants ("Lengan
/// Pendek - L", "Celana Wanita") that simpelv1 has carried since 2023.
#[component]
fn SubSpesifikasiPanel(spesifikasi_id: String, is_admin: bool) -> impl IntoView {
    // `Show` re-invokes its children, so these are stored once and read per
    // call instead of being moved into the first invocation.
    let id_nama = StoredValue::new(format!("sub-nama-{spesifikasi_id}"));
    let id_gender = StoredValue::new(format!("sub-gender-{spesifikasi_id}"));
    let sid = StoredValue::new(spesifikasi_id);
    let refresh = RwSignal::new(0);
    let (nama, set_nama) = signal(String::new());
    let gender = RwSignal::new("SEMUA".to_string());
    let (saving, set_saving) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let client: QueryClient = expect_context();
    let data = client.local_resource(query_subspesifikasi, move || {
        (sid.get_value(), refresh.get())
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_saving.set(true);
        set_error.set(None);
        let request = CreateSubSpesifikasiRequest {
            spesifikasi_id: sid.get_value(),
            nama: nama.get(),
            gender: gender.get(),
            is_active: true,
        };
        spawn_local(async move {
            match create_subspesifikasi(request).await {
                Ok(_) => {
                    set_nama.set(String::new());
                    refresh.update(|v| *v += 1);
                }
                Err(e) => set_error.set(Some(format!("Gagal menyimpan: {e}"))),
            }
            set_saving.set(false);
        });
    };

    view! {
        <div class="flex flex-col gap-3">
            <p class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-slate-400">
                <AppIcon icon=LIST size=14 />
                "Subspesifikasi"
            </p>

            <Suspense fallback=move || {
                view! { <p class="text-xs text-slate-500">"Memuat…"</p> }
            }>
                {move || match data.get() {
                    None => view! { <p class="text-xs text-slate-500">"Memuat…"</p> }.into_any(),
                    Some(Err(e)) => view! { <ErrorState error=e /> }.into_any(),
                    Some(Ok(resp)) => {
                        let rows = resp.data;
                        if rows.is_empty() {
                            view! {
                                <p class="text-xs text-slate-500 italic">
                                    "Belum ada subspesifikasi. Opsional — hanya perlu bila satu spesifikasi punya varian per gender."
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <ul
                                    data-testid="subspesifikasi-daftar"
                                    class="flex flex-wrap gap-2"
                                >
                                    {rows
                                        .into_iter()
                                        .map(|sub| {
                                            let sub_id = sub.id.clone();
                                            view! {
                                                <li class="inline-flex items-center gap-2 rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-sm text-slate-200">
                                                    <span>{sub.nama.clone()}</span>
                                                    <span class="text-xs text-slate-500">
                                                        {label_of(&GENDERS, &sub.gender)}
                                                    </span>
                                                    <Show when=move || is_admin>
                                                        <button
                                                            type="button"
                                                            aria-label="Hapus subspesifikasi"
                                                            class="text-danger-400 transition hover:text-danger-300"
                                                            on:click={
                                                                let sub_id = sub_id.clone();
                                                                move |_| {
                                                                    let sub_id = sub_id.clone();
                                                                    spawn_local(async move {
                                                                        if delete_subspesifikasi(sub_id).await.is_ok() {
                                                                            refresh.update(|v| *v += 1);
                                                                        }
                                                                    });
                                                                }
                                                            }
                                                        >
                                                            <AppIcon icon=TRASH size=14 />
                                                        </button>
                                                    </Show>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            }
                                .into_any()
                        }
                    }
                }}
            </Suspense>

            <Show when=move || is_admin>
                <form class="flex flex-wrap items-end gap-3" on:submit=on_submit>
                    <div class="min-w-[16rem] flex-1">
                        <FormField label="Nama Subspesifikasi" for_id=id_nama.get_value()>
                            <input
                                id=id_nama.get_value()
                                type="text"
                                class=FIELD
                                placeholder="Contoh: Lengan Pendek - L"
                                prop:value=move || nama.get()
                                on:input=move |ev| set_nama.set(event_target_value(&ev))
                                required=true
                            />
                        </FormField>
                    </div>
                    <div class="w-44">
                        <FormField label="Gender" for_id=id_gender.get_value()>
                            <select
                                id=id_gender.get_value()
                                class=FIELD
                                prop:value=move || gender.get()
                                on:change=move |ev| gender.set(event_target_value(&ev))
                            >
                                {GENDERS
                                    .iter()
                                    .map(|(k, v)| view! { <option value=*k>{*v}</option> })
                                    .collect_view()}
                            </select>
                        </FormField>
                    </div>
                    <button
                        type="submit"
                        class="rounded-lg border border-white/10 bg-white/[0.06] px-4 py-2.5 text-sm text-slate-200 transition hover:bg-white/[0.1] disabled:opacity-50"
                        prop:disabled=move || saving.get()
                    >
                        {move || if saving.get() { "Menyimpan…" } else { "Tambah" }}
                    </button>
                </form>
                <Show when=move || error.get().is_some()>
                    <p class="text-xs text-danger-300">{move || error.get()}</p>
                </Show>
            </Show>
        </div>
    }
}
