//! Ukuran Pegawai Component
//!
//! Allows employees to input their personal uniform sizes (baju, celana, sepatu).

use crate::api::{
    PegawaiPakaianDinas, Ukuran, UpsertPegawaiUkuranRequest, fetch_master_ukuran,
    fetch_pegawai_ukuran, upsert_pegawai_ukuran,
};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn UkuranPegawai(
    /// Pegawai ID from JWT claims or context
    pegawai_id: String,
    /// Pegawai name for display
    pegawai_nama: String,
    /// Pegawai NIP for display
    pegawai_nip: String,
) -> impl IntoView {
    let (ukuran_baju, set_ukuran_baju) = signal(String::new());
    let (ukuran_celana, set_ukuran_celana) = signal(String::new());
    let (ukuran_sepatu, set_ukuran_sepatu) = signal(String::new());
    let (is_loading, set_is_loading) = signal(false);
    let (is_saving, set_is_saving) = signal(false);
    let (success_message, set_success_message) = signal(Option::<String>::None);
    let (error_message, set_error_message) = signal(Option::<String>::None);

    let pegawai_id_clone = pegawai_id.clone();

    // Fetch master ukuran data
    let master_ukuran = LocalResource::new(|| async move {
        let baju = fetch_master_ukuran(Some("BAJU".to_string())).await.ok();
        let celana = fetch_master_ukuran(Some("CELANA".to_string())).await.ok();
        let sepatu = fetch_master_ukuran(Some("SEPATU".to_string())).await.ok();
        (
            baju.map(|r| r.data).unwrap_or_default(),
            celana.map(|r| r.data).unwrap_or_default(),
            sepatu.map(|r| r.data).unwrap_or_default(),
        )
    });

    // Fetch existing ukuran for pegawai
    let pegawai_id_for_fetch = pegawai_id.clone();
    let existing_ukuran = LocalResource::new(move || {
        let pid = pegawai_id_for_fetch.clone();
        async move {
            match fetch_pegawai_ukuran(pid).await {
                Ok(response) => response.data,
                Err(_) => None,
            }
        }
    });

    // Effect to populate form when existing data loads
    Effect::new(move || {
        if let Some(Some(ukuran)) = existing_ukuran.get() {
            if let Some(baju) = ukuran.ukuran_baju {
                set_ukuran_baju.set(baju);
            }
            if let Some(celana) = ukuran.ukuran_celana {
                set_ukuran_celana.set(celana);
            }
            if let Some(sepatu) = ukuran.ukuran_sepatu {
                set_ukuran_sepatu.set(sepatu);
            }
        }
    });

    // Handle form submit - create it inside the render to avoid FnOnce issues
    let pegawai_id_for_submit = pegawai_id.clone();

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="mb-6">
                <h2 class="text-xl font-bold text-gray-800">"Ukuran Pakaian Dinas Saya"</h2>
                <p class="text-sm text-gray-500 mt-1">
                    "Silakan isi ukuran pakaian dinas Anda untuk keperluan pengajuan"
                </p>
            </div>

            // Employee info card
            <div class="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
                <div class="flex items-center">
                    <div class="w-12 h-12 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold text-xl mr-4">
                        {pegawai_nama.chars().next().unwrap_or('?').to_string()}
                    </div>
                    <div>
                        <h3 class="font-semibold text-gray-800">{pegawai_nama}</h3>
                        <p class="text-sm text-gray-600">"NIP: " {pegawai_nip}</p>
                    </div>
                </div>
            </div>

            // Messages
            <Show when=move || success_message.get().is_some()>
                <div class="mb-4 p-3 bg-green-50 border border-green-200 text-green-700 rounded flex items-center">
                    <i class="fas fa-check-circle mr-2"></i>
                    {move || success_message.get()}
                </div>
            </Show>

            <Show when=move || error_message.get().is_some()>
                <div class="mb-4 p-3 bg-red-50 border border-red-200 text-red-700 rounded flex items-center">
                    <i class="fas fa-exclamation-circle mr-2"></i>
                    {move || error_message.get()}
                </div>
            </Show>

            <Suspense fallback=move || {
                view! { <div class="text-center py-8">"Memuat data ukuran..."</div> }
            }>
                {move || {
                    let pegawai_id_clone = pegawai_id_for_submit.clone();
                    master_ukuran
                        .get()
                        .map(|(baju_sizes, celana_sizes, sepatu_sizes)| {
                            let pid = pegawai_id_clone.clone();
                            let on_submit = move |ev: leptos::ev::SubmitEvent| {
                                ev.prevent_default();
                                set_is_saving.set(true);
                                set_success_message.set(None);
                                set_error_message.set(None);

                                let pid = pid.clone();
                                let baju = ukuran_baju.get();
                                let celana = ukuran_celana.get();
                                let sepatu = ukuran_sepatu.get();

                                spawn_local(async move {
                                    let request = UpsertPegawaiUkuranRequest {
                                        pegawai_id: pid,
                                        ukuran_baju: if baju.is_empty() { None } else { Some(baju) },
                                        ukuran_celana: if celana.is_empty() {
                                            None
                                        } else {
                                            Some(celana)
                                        },
                                        ukuran_sepatu: if sepatu.is_empty() {
                                            None
                                        } else {
                                            Some(sepatu)
                                        },
                                    };

                                    match upsert_pegawai_ukuran(request).await {
                                        Ok(_) => {
                                            set_success_message.set(Some("Ukuran berhasil disimpan!".to_string()));
                                        }
                                        Err(e) => {
                                            set_error_message.set(Some(format!("Gagal menyimpan: {:?}", e)));
                                        }
                                    }
                                    set_is_saving.set(false);
                                });
                            };
                            view! {
                                <form on:submit=on_submit>
                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                                        // Ukuran Baju
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                <i class="fas fa-tshirt mr-2 text-blue-500"></i>
                                                "Ukuran Baju"
                                            </label>
                                            <select
                                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                prop:value=move || ukuran_baju.get()
                                                on:change=move |ev| set_ukuran_baju
                                                    .set(event_target_value(&ev))
                                            >
                                                <option value="">"-- Pilih Ukuran --"</option>
                                                <For
                                                    each=move || baju_sizes.clone()
                                                    key=|u| u.id.clone()
                                                    children=move |u: Ukuran| {
                                                        let size = u.size.clone();
                                                        view! { <option value=size.clone()>{size.clone()}</option> }
                                                    }
                                                />
                                            </select>
                                            <p class="text-xs text-gray-500 mt-1">
                                                "Ukuran standar: S, M, L, XL, XXL, XXXL"
                                            </p>
                                        </div>

                                        // Ukuran Celana
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                <i class="fas fa-male mr-2 text-green-500"></i>
                                                "Ukuran Celana"
                                            </label>
                                            <select
                                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                prop:value=move || ukuran_celana.get()
                                                on:change=move |ev| set_ukuran_celana
                                                    .set(event_target_value(&ev))
                                            >
                                                <option value="">"-- Pilih Ukuran --"</option>
                                                <For
                                                    each=move || celana_sizes.clone()
                                                    key=|u| u.id.clone()
                                                    children=move |u: Ukuran| {
                                                        let size = u.size.clone();
                                                        view! { <option value=size.clone()>{size.clone()}</option> }
                                                    }
                                                />
                                            </select>
                                            <p class="text-xs text-gray-500 mt-1">
                                                "Ukuran standar: 27-42 (angka)"
                                            </p>
                                        </div>

                                        // Ukuran Sepatu
                                        <div class="bg-gray-50 p-4 rounded-lg">
                                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                                <i class="fas fa-shoe-prints mr-2 text-orange-500"></i>
                                                "Ukuran Sepatu"
                                            </label>
                                            <select
                                                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                                prop:value=move || ukuran_sepatu.get()
                                                on:change=move |ev| set_ukuran_sepatu
                                                    .set(event_target_value(&ev))
                                            >
                                                <option value="">"-- Pilih Ukuran --"</option>
                                                <For
                                                    each=move || sepatu_sizes.clone()
                                                    key=|u| u.id.clone()
                                                    children=move |u: Ukuran| {
                                                        let size = u.size.clone();
                                                        view! { <option value=size.clone()>{size.clone()}</option> }
                                                    }
                                                />
                                            </select>
                                            <p class="text-xs text-gray-500 mt-1">
                                                "Ukuran standar: 36-46 (angka)"
                                            </p>
                                        </div>
                                    </div>

                                    <div class="mt-6 flex justify-end">
                                        <button
                                            type="submit"
                                            class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 inline-flex items-center"
                                            prop:disabled=move || is_saving.get()
                                        >
                                            <i class="fas fa-save mr-2"></i>
                                            {move || {
                                                if is_saving.get() { "Menyimpan..." } else { "Simpan Ukuran" }
                                            }}
                                        </button>
                                    </div>
                                </form>

                                // Size guide
                                <div class="mt-8 p-4 bg-yellow-50 rounded-lg border border-yellow-200">
                                    <h4 class="font-semibold text-yellow-800 mb-2">
                                        <i class="fas fa-info-circle mr-2"></i>
                                        "Panduan Pengukuran"
                                    </h4>
                                    <ul class="text-sm text-yellow-700 space-y-1">
                                        <li>
                                            <strong>"Baju:"</strong>
                                            " Ukur lingkar dada pada bagian terlebar, pilih ukuran yang sesuai."
                                        </li>
                                        <li>
                                            <strong>"Celana:"</strong>
                                            " Ukur lingkar pinggang pada posisi normal."
                                        </li>
                                        <li>
                                            <strong>"Sepatu:"</strong>
                                            " Ukur panjang kaki dari tumit ke ujung jari terpanjang."
                                        </li>
                                    </ul>
                                </div>
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}

/// Component for admin to view/edit all employees' sizes in a satker
#[component]
pub fn UkuranPegawaiSatker(
    /// Satker ID to show employees for
    satker_id: String,
    /// Satker name for display
    satker_nama: String,
) -> impl IntoView {
    use crate::api::{PegawaiWithSizes, fetch_pegawai_with_sizes};

    let (page, set_page) = signal(1);
    let satker_id_clone = satker_id.clone();

    let data_resource = LocalResource::new(move || {
        let sid = satker_id_clone.clone();
        let p = page.get();
        async move {
            match fetch_pegawai_with_sizes(sid, p, 20).await {
                Ok(response) => Some(response),
                Err(e) => {
                    leptos::logging::error!("Failed to fetch pegawai with sizes: {:?}", e);
                    None
                }
            }
        }
    });

    view! {
        <div class="p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <div class="flex items-center justify-between mb-6">
                <div>
                    <h2 class="text-xl font-bold text-gray-800">
                        "Ukuran Pakaian Pegawai - " {satker_nama}
                    </h2>
                    <p class="text-sm text-gray-500 mt-1">
                        "Daftar ukuran pakaian dinas seluruh pegawai di satker"
                    </p>
                </div>
                <a
                    href="/perlengkapan/dashboard/pakaian-dinas/laporan/rekap"
                    class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors inline-flex items-center"
                >
                    <i class="fas fa-chart-bar mr-2"></i>
                    "Rekap Ukuran"
                </a>
            </div>

            <Suspense fallback=move || {
                view! { <div class="text-center py-8">"Memuat data..."</div> }
            }>
                {move || {
                    data_resource
                        .get()
                        .flatten()
                        .map(|response| {
                            let data_len = response.data.len();
                            let data_for_for = response.data.clone();
                            if response.data.is_empty() {
                                view! {
                                    <div class="text-center py-12 text-gray-500">
                                        <i class="fas fa-users text-4xl mb-3 text-gray-300"></i>
                                        <p>"Tidak ada data pegawai di satker ini."</p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="w-full text-left border-collapse">
                                            <thead>
                                                <tr class="bg-gray-50 text-gray-600 text-sm uppercase tracking-wider">
                                                    <th class="p-3 font-semibold border-b">"No"</th>
                                                    <th class="p-3 font-semibold border-b">"NIP"</th>
                                                    <th class="p-3 font-semibold border-b">"Nama"</th>
                                                    <th class="p-3 font-semibold border-b">"Jabatan"</th>
                                                    <th class="p-3 font-semibold border-b text-center">
                                                        "Baju"
                                                    </th>
                                                    <th class="p-3 font-semibold border-b text-center">
                                                        "Celana"
                                                    </th>
                                                    <th class="p-3 font-semibold border-b text-center">
                                                        "Sepatu"
                                                    </th>
                                                    <th class="p-3 font-semibold border-b">"Status"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="text-gray-700 text-sm">
                                                <For
                                                    each=move || {
                                                        data_for_for.clone().into_iter().enumerate()
                                                    }
                                                    key=|(_, item)| item.pegawai.id.clone()
                                                    children=move |(idx, item): (usize, PegawaiWithSizes)| {
                                                        let has_sizes = item.ukuran.is_some();
                                                        let ukuran = item.ukuran.clone();
                                                        view! {
                                                            <tr class="hover:bg-gray-50 border-b last:border-0 transition-colors">
                                                                <td class="p-3">
                                                                    {((page.get() - 1) * 20 + idx as i32 + 1).to_string()}
                                                                </td>
                                                                <td class="p-3 font-mono text-sm">{item.pegawai.nip}</td>
                                                                <td class="p-3 font-medium">{item.pegawai.nama}</td>
                                                                <td class="p-3">
                                                                    {item.pegawai.jabatan.unwrap_or_else(|| "-".to_string())}
                                                                </td>
                                                                <td class="p-3 text-center">
                                                                    {ukuran
                                                                        .as_ref()
                                                                        .and_then(|u| u.ukuran_baju.clone())
                                                                        .unwrap_or_else(|| "-".to_string())}
                                                                </td>
                                                                <td class="p-3 text-center">
                                                                    {ukuran
                                                                        .as_ref()
                                                                        .and_then(|u| u.ukuran_celana.clone())
                                                                        .unwrap_or_else(|| "-".to_string())}
                                                                </td>
                                                                <td class="p-3 text-center">
                                                                    {ukuran
                                                                        .and_then(|u| u.ukuran_sepatu)
                                                                        .unwrap_or_else(|| "-".to_string())}
                                                                </td>
                                                                <td class="p-3">
                                                                    {if has_sizes {
                                                                        view! {
                                                                            <span class="px-2 py-1 text-xs font-medium rounded-full bg-green-100 text-green-800">
                                                                                "Lengkap"
                                                                            </span>
                                                                        }
                                                                            .into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span class="px-2 py-1 text-xs font-medium rounded-full bg-yellow-100 text-yellow-800">
                                                                                "Belum Isi"
                                                                            </span>
                                                                        }
                                                                            .into_any()
                                                                    }}
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>

                                        // Pagination
                                        <div class="flex items-center justify-between mt-6 pt-4 border-t border-gray-100">
                                            <div class="text-sm text-gray-500">
                                                "Menampilkan "
                                                <span class="font-medium">{data_len}</span>
                                                " dari " <span class="font-medium">{response.total}</span>
                                                " pegawai"
                                            </div>
                                            <div class="flex gap-2">
                                                <button
                                                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                    prop:disabled=move || page.get() <= 1
                                                    on:click=move |_| set_page.update(|p| *p -= 1)
                                                >
                                                    "Sebelumnya"
                                                </button>
                                                <button
                                                    class="px-3 py-1 border rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                                                    prop:disabled=move || page.get() >= response.total_pages
                                                    on:click=move |_| set_page.update(|p| *p += 1)
                                                >
                                                    "Selanjutnya"
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Suspense>
        </div>
    }
}
