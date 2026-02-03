use crate::api::{CreatePemeliharaanRequest, create_pemeliharaan, fetch_assets};
use chrono::NaiveDate;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use uuid::Uuid;

#[component]
pub fn PemeliharaanForm() -> impl IntoView {
    let navigate = use_navigate();
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Form fields
    let (asset_id, set_asset_id) = signal("".to_string());
    let (jenis, set_jenis) = signal("".to_string());
    let (biaya, set_biaya) = signal("".to_string());
    let (tanggal_mulai, set_tanggal_mulai) = signal("".to_string());
    let (tanggal_selesai, set_tanggal_selesai) = signal("".to_string());
    let (pelaksana, set_pelaksana) = signal("".to_string());
    let (keterangan, set_keterangan) = signal("".to_string());

    // Load assets for dropdown
    let assets_resource = LocalResource::new(|| async move {
        fetch_assets(1, 100, None).await.map_err(|e| e.to_string())
    });

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let asset_uuid = match Uuid::parse_str(&asset_id.get()) {
            Ok(u) => u,
            Err(_) => {
                set_error.set(Some("Aset tidak valid".to_string()));
                set_loading.set(false);
                return;
            }
        };

        let tgl_mulai = match NaiveDate::parse_from_str(&tanggal_mulai.get(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_error.set(Some("Tanggal mulai tidak valid".to_string()));
                set_loading.set(false);
                return;
            }
        };

        let tgl_selesai = if tanggal_selesai.get().is_empty() {
            None
        } else {
            match NaiveDate::parse_from_str(&tanggal_selesai.get(), "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    set_error.set(Some("Tanggal selesai tidak valid".to_string()));
                    set_loading.set(false);
                    return;
                }
            }
        };

        let req = CreatePemeliharaanRequest {
            asset_id: asset_uuid,
            jenis_pemeliharaan: jenis.get(),
            biaya: biaya.get().parse::<f64>().ok(),
            tanggal_mulai: tgl_mulai,
            tanggal_selesai: tgl_selesai,
            pelaksana: pelaksana.get(),
            keterangan: if keterangan.get().is_empty() {
                None
            } else {
                Some(keterangan.get())
            },
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_pemeliharaan(req).await {
                Ok(_) => {
                    set_loading.set(false);
                    navigate(
                        "/dashboard/pengelolaan/pemeliharaan/daftar",
                        Default::default(),
                    );
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan pemeliharaan: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="max-w-2xl mx-auto bg-white p-6 rounded-lg shadow">
            <h2 class="text-2xl font-bold mb-6">"Catat Pemeliharaan Aset"</h2>

            {move || error.get().map(|e| view! {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
                    <span class="block sm:inline">{e}</span>
                </div>
            })}

            <form on:submit=on_submit>
                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="asset_id">
                        "Pilih Aset"
                    </label>
                    <select
                        id="asset_id"
                        class="shadow border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || asset_id.get()
                        on:change=move |ev| set_asset_id.set(event_target_value(&ev))
                        required
                    >
                        <option value="">"Pilih Aset..."</option>
                        <Suspense fallback=move || view! { <option>"Loading..."</option> }>
                            {move || {
                                assets_resource.get().map(|res| match res {
                                    Ok(resp) => {
                                        resp.data.into_iter().map(|asset| {
                                            view! {
                                                <option value={asset.id.to_string()}>
                                                    {format!("{} - {}", asset.no_aset, asset.nama_aset.unwrap_or_default())}
                                                </option>
                                            }
                                        }).collect_view().into_any()
                                    },
                                    Err(_) => view! { <option>"Error loading assets"</option> }.into_any()
                                })
                            }}
                        </Suspense>
                    </select>
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="jenis">
                        "Jenis Pemeliharaan"
                    </label>
                    <select
                        id="jenis"
                        class="shadow border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || jenis.get()
                        on:change=move |ev| set_jenis.set(event_target_value(&ev))
                        required
                    >
                        <option value="">"Pilih Jenis..."</option>
                        <option value="Rutin">"Rutin"</option>
                        <option value="Perbaikan Ringan">"Perbaikan Ringan"</option>
                        <option value="Perbaikan Berat">"Perbaikan Berat"</option>
                        <option value="Renovasi">"Renovasi"</option>
                    </select>
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="pelaksana">
                        "Pelaksana"
                    </label>
                    <input
                        id="pelaksana"
                        type="text"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || pelaksana.get()
                        on:input=move |ev| set_pelaksana.set(event_target_value(&ev))
                        placeholder="Nama Vendor / Internal"
                        required
                    />
                </div>

                <div class="flex gap-4 mb-4">
                    <div class="w-1/2">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="tanggal_mulai">
                            "Tanggal Mulai"
                        </label>
                        <input
                            id="tanggal_mulai"
                            type="date"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            prop:value=move || tanggal_mulai.get()
                            on:input=move |ev| set_tanggal_mulai.set(event_target_value(&ev))
                            required
                        />
                    </div>
                    <div class="w-1/2">
                        <label class="block text-gray-700 text-sm font-bold mb-2" for="tanggal_selesai">
                            "Tanggal Selesai (Estimasi)"
                        </label>
                        <input
                            id="tanggal_selesai"
                            type="date"
                            class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                            prop:value=move || tanggal_selesai.get()
                            on:input=move |ev| set_tanggal_selesai.set(event_target_value(&ev))
                        />
                    </div>
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="biaya">
                        "Biaya (Rp)"
                    </label>
                    <input
                        id="biaya"
                        type="number"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || biaya.get()
                        on:input=move |ev| set_biaya.set(event_target_value(&ev))
                        placeholder="0"
                    />
                </div>

                <div class="mb-6">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="keterangan">
                        "Keterangan"
                    </label>
                    <textarea
                        id="keterangan"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || keterangan.get()
                        on:input=move |ev| set_keterangan.set(event_target_value(&ev))
                        rows="3"
                    ></textarea>
                </div>

                <div class="flex items-center justify-between">
                    <button
                        type="submit"
                        class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded focus:outline-none focus:shadow-outline"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Menyimpan..." } else { "Simpan" }}
                    </button>
                    <a
                        href="/dashboard/pengelolaan/pemeliharaan"
                        class="inline-block align-baseline font-bold text-sm text-blue-600 hover:text-blue-800"
                    >
                        "Batal"
                    </a>
                </div>
            </form>
        </div>
    }
}
