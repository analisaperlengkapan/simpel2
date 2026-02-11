//! Roadmap Sarpras Form Component
//!
//! Form for creating 5-year infrastructure roadmap with multi-year planning.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItemInput {
    pub kode_barang: String,
    pub nama_barang: String,
    pub tahun_rencana: i32,
    pub jumlah_kebutuhan: i32,
    pub estimasi_anggaran: Option<f64>,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoadmapBatchRequest {
    pub satker_id: Uuid,
    pub periode_mulai: i32,
    pub periode_akhir: i32,
    pub items: Vec<RoadmapItemInput>,
}

#[component]
pub fn RoadmapSarprasForm(
    #[prop(optional)] on_success: Option<Callback<()>>,
) -> impl IntoView {
    let (satker_id, set_satker_id) = signal(Uuid::nil());
    let (periode_mulai, set_periode_mulai) = signal(2025);
    let (items, set_items) = signal::<Vec<RoadmapItemInput>>(Vec::new());
    let (is_submitting, set_is_submitting) = signal(false);
    let (error_message, set_error_message) = signal::<Option<String>>(None);
    let (success_message, set_success_message) = signal::<Option<String>>(None);

    // Calculate periode_akhir (always 5 years)
    let periode_akhir = Memo::new(move |_| periode_mulai.get() + 4);

    // Add new item
    let add_item = move |_| {
        set_items.update(|items| {
            items.push(RoadmapItemInput {
                kode_barang: String::new(),
                nama_barang: String::new(),
                tahun_rencana: periode_mulai.get(),
                jumlah_kebutuhan: 1,
                estimasi_anggaran: None,
                keterangan: None,
            });
        });
    };

    // Remove item
    let remove_item = move |index: usize| {
        set_items.update(|items| {
            items.remove(index);
        });
    };

    // Update item field
    let update_item = move |index: usize, field: String, value: String| {
        set_items.update(|items| {
            if let Some(item) = items.get_mut(index) {
                match field.as_str() {
                    "kode_barang" => item.kode_barang = value,
                    "nama_barang" => item.nama_barang = value,
                    "tahun_rencana" => {
                        if let Ok(year) = value.parse::<i32>() {
                            item.tahun_rencana = year;
                        }
                    }
                    "jumlah_kebutuhan" => {
                        if let Ok(qty) = value.parse::<i32>() {
                            item.jumlah_kebutuhan = qty;
                        }
                    }
                    "estimasi_anggaran" => {
                        item.estimasi_anggaran = value.parse::<f64>().ok();
                    }
                    "keterangan" => {
                        item.keterangan = if value.is_empty() { None } else { Some(value) };
                    }
                    _ => {}
                }
            }
        });
    };

    // Submit form
    let submit_form = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        set_is_submitting.set(true);
        set_error_message.set(None);
        set_success_message.set(None);

        let request = CreateRoadmapBatchRequest {
            satker_id: satker_id.get(),
            periode_mulai: periode_mulai.get(),
            periode_akhir: periode_akhir.get(),
            items: items.get(),
        };

        spawn_local(async move {
            match create_roadmap_batch(request).await {
                Ok(_) => {
                    set_success_message.set(Some("Roadmap berhasil dibuat!".to_string()));
                    set_items.set(Vec::new());
                    if let Some(callback) = on_success {
                        callback.run(());
                    }
                }
                Err(e) => {
                    set_error_message.set(Some(format!("Gagal membuat roadmap: {}", e)));
                }
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <div class="bg-white rounded-xl shadow-sm border border-gray-100 p-6">
            <h2 class="text-2xl font-bold text-gray-900 mb-6">
                "Buat Roadmap Sarpras 5 Tahun"
            </h2>

            {move || error_message.get().map(|msg| view! {
                <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-lg text-red-700">
                    {msg}
                </div>
            })}

            {move || success_message.get().map(|msg| view! {
                <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg text-green-700">
                    {msg}
                </div>
            })}

            <form on:submit=submit_form>
                // Period Selection
                <div class="mb-6 p-4 bg-blue-50 rounded-lg">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "Periode Roadmap (5 Tahun)"
                    </label>
                    <div class="flex items-center gap-4">
                        <div>
                            <label class="text-xs text-gray-600">"Tahun Mulai"</label>
                            <input
                                type="number"
                                class="mt-1 block w-32 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                value=move || periode_mulai.get()
                                on:input=move |ev| {
                                    if let Ok(year) = event_target_value(&ev).parse::<i32>() {
                                        set_periode_mulai.set(year);
                                    }
                                }
                                min="2020"
                                max="2100"
                            />
                        </div>
                        <span class="text-gray-500 mt-6">"-"</span>
                        <div>
                            <label class="text-xs text-gray-600">"Tahun Akhir"</label>
                            <input
                                type="number"
                                class="mt-1 block w-32 rounded-md border-gray-300 bg-gray-100 shadow-sm"
                                value=move || periode_akhir.get()
                                disabled
                            />
                        </div>
                    </div>
                </div>

                // Items List
                <div class="mb-6">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-semibold text-gray-900">
                            "Item Roadmap"
                        </h3>
                        <button
                            type="button"
                            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                            on:click=add_item
                        >
                            "+ Tambah Item"
                        </button>
                    </div>

                    {move || {
                        let items_list = items.get();
                        if items_list.is_empty() {
                            view! {
                                <div class="text-center py-8 text-gray-500">
                                    "Belum ada item. Klik 'Tambah Item' untuk memulai."
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-4">
                                    <For
                                        each=move || items.get().into_iter().enumerate()
                                        key=|(i, _)| *i
                                        children=move |(index, item)| {
                                            view! {
                                                <div class="p-4 border border-gray-200 rounded-lg bg-gray-50">
                                                    <div class="flex justify-between items-start mb-4">
                                                        <h4 class="font-medium text-gray-900">
                                                            {format!("Item #{}", index + 1)}
                                                        </h4>
                                                        <button
                                                            type="button"
                                                            class="text-red-600 hover:text-red-800"
                                                            on:click=move |_| remove_item(index)
                                                        >
                                                            "✕ Hapus"
                                                        </button>
                                                    </div>

                                                    <div class="grid grid-cols-2 gap-4">
                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Kode Barang"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                value=item.kode_barang.clone()
                                                                on:input=move |ev| {
                                                                    update_item(index, "kode_barang".to_string(), event_target_value(&ev));
                                                                }
                                                                required
                                                            />
                                                        </div>

                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Nama Barang"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                value=item.nama_barang.clone()
                                                                on:input=move |ev| {
                                                                    update_item(index, "nama_barang".to_string(), event_target_value(&ev));
                                                                }
                                                                required
                                                            />
                                                        </div>

                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Tahun Rencana"
                                                            </label>
                                                            <select
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                on:change=move |ev| {
                                                                    update_item(index, "tahun_rencana".to_string(), event_target_value(&ev));
                                                                }
                                                            >
                                                                {move || {
                                                                    let start = periode_mulai.get();
                                                                    let end = periode_akhir.get();
                                                                    (start..=end).map(|year| {
                                                                        view! {
                                                                            <option value=year.to_string() selected=year == item.tahun_rencana>
                                                                                {year.to_string()}
                                                                            </option>
                                                                        }
                                                                    }).collect_view()
                                                                }}
                                                            </select>
                                                        </div>

                                                        <div>
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Jumlah Kebutuhan"
                                                            </label>
                                                            <input
                                                                type="number"
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                value=item.jumlah_kebutuhan
                                                                on:input=move |ev| {
                                                                    update_item(index, "jumlah_kebutuhan".to_string(), event_target_value(&ev));
                                                                }
                                                                min="1"
                                                                required
                                                            />
                                                        </div>

                                                        <div class="col-span-2">
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Estimasi Anggaran (Rp)"
                                                            </label>
                                                            <input
                                                                type="number"
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                value=item.estimasi_anggaran.unwrap_or(0.0)
                                                                on:input=move |ev| {
                                                                    update_item(index, "estimasi_anggaran".to_string(), event_target_value(&ev));
                                                                }
                                                                step="0.01"
                                                            />
                                                        </div>

                                                        <div class="col-span-2">
                                                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                                                "Keterangan"
                                                            </label>
                                                            <textarea
                                                                class="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                                                                rows="2"
                                                                on:input=move |ev| {
                                                                    update_item(index, "keterangan".to_string(), event_target_value(&ev));
                                                                }
                                                            >
                                                                {item.keterangan.clone().unwrap_or_default()}
                                                            </textarea>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                    }}
                </div>

                // Submit Button
                <div class="flex justify-end gap-4">
                    <button
                        type="submit"
                        class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:bg-gray-400 disabled:cursor-not-allowed"
                        disabled=move || is_submitting.get() || items.get().is_empty()
                    >
                        {move || if is_submitting.get() {
                            "Menyimpan..."
                        } else {
                            "Simpan Roadmap"
                        }}
                    </button>
                </div>
            </form>
        </div>
    }
}

// API function (to be added to api.rs)
async fn create_roadmap_batch(request: CreateRoadmapBatchRequest) -> Result<(), String> {
    use gloo_net::http::Request;

    let response = Request::post("/api/v1/roadmap-sarpras/batch")
        .json(&request)
        .map_err(|e| format!("Failed to serialize request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(error_text)
    }
}
