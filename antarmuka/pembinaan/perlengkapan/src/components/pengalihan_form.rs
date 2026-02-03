use crate::api::{CreatePengalihanRequest, create_pengalihan, fetch_assets};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use uuid::Uuid;
use chrono::NaiveDate;

#[component]
pub fn PengalihanForm() -> impl IntoView {
    let navigate = use_navigate();
    let (error, set_error) = signal(None::<String>);
    let (loading, set_loading) = signal(false);

    // Form fields
    let (asset_id, set_asset_id) = signal("".to_string());
    let (pihak_lama, set_pihak_lama) = signal("".to_string());
    let (pihak_baru, set_pihak_baru) = signal("".to_string());
    let (tanggal_pengalihan, set_tanggal_pengalihan) = signal("".to_string());
    let (dasar_pengalihan, set_dasar_pengalihan) = signal("".to_string());
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

        let tgl_pengalihan = match NaiveDate::parse_from_str(&tanggal_pengalihan.get(), "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                set_error.set(Some("Tanggal pengalihan tidak valid".to_string()));
                set_loading.set(false);
                return;
            }
        };

        let req = CreatePengalihanRequest {
            asset_id: asset_uuid,
            pihak_lama: pihak_lama.get(),
            pihak_baru: pihak_baru.get(),
            tanggal_pengalihan: tgl_pengalihan,
            dasar_pengalihan: if dasar_pengalihan.get().is_empty() {
                None
            } else {
                Some(dasar_pengalihan.get())
            },
            keterangan: if keterangan.get().is_empty() {
                None
            } else {
                Some(keterangan.get())
            },
        };

        let navigate = navigate.clone();
        spawn_local(async move {
            match create_pengalihan(req).await {
                Ok(_) => {
                    set_loading.set(false);
                    navigate(
                        "/dashboard/pengelolaan/pengalihan/daftar",
                        Default::default(),
                    );
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal menyimpan pengalihan: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="max-w-2xl mx-auto bg-white p-6 rounded-lg shadow">
            <h2 class="text-2xl font-bold mb-6">"Form Pengalihan Aset"</h2>

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
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="pihak_lama">
                        "Pihak Lama"
                    </label>
                    <input
                        id="pihak_lama"
                        type="text"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || pihak_lama.get()
                        on:input=move |ev| set_pihak_lama.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="pihak_baru">
                        "Pihak Baru"
                    </label>
                    <input
                        id="pihak_baru"
                        type="text"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || pihak_baru.get()
                        on:input=move |ev| set_pihak_baru.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="tanggal_pengalihan">
                        "Tanggal Pengalihan"
                    </label>
                    <input
                        id="tanggal_pengalihan"
                        type="date"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || tanggal_pengalihan.get()
                        on:input=move |ev| set_tanggal_pengalihan.set(event_target_value(&ev))
                        required
                    />
                </div>

                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-bold mb-2" for="dasar_pengalihan">
                        "Dasar Pengalihan (SK/Dokumen)"
                    </label>
                    <input
                        id="dasar_pengalihan"
                        type="text"
                        class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
                        prop:value=move || dasar_pengalihan.get()
                        on:input=move |ev| set_dasar_pengalihan.set(event_target_value(&ev))
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
                        href="/dashboard/pengelolaan/pengalihan"
                        class="inline-block align-baseline font-bold text-sm text-blue-600 hover:text-blue-800"
                    >
                        "Batal"
                    </a>
                </div>
            </form>
        </div>
    }
}
