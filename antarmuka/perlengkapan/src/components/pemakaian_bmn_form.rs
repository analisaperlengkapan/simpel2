//! # Pemakaian BMN Form Component
//!
//! Dynamic form for creating BMN usage permits with type-specific fields.
//!
//! **IMPORTANT - ROLE STRUCTURE:**
//! - This form is ONLY accessible by Operator Satker
//! - Operator Satker creates permits ON BEHALF OF employees (pegawai)
//! - There is NO self-service for employees in perlengkapan domain
//! - Operator Satker inputs pegawai NIP and name manually or selects from dropdown
//!
//! Requirements: REQ-P001, REQ-P002, REQ-P014

use crate::api::{
    BmnAvailabilityResponse, CreateBmnItemRequest, CreateIzinPemakaianRequest,
    bank_aset::lookup_by_nup, check_bmn_availability, create_pemakaian_bmn,
};
use crate::features::auth::AuthService;
use crate::routes;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use lib_ui::components::icon::AppIcon;
use phosphor_leptos::{CHECK_CIRCLE, PAPER_PLANE_TILT, PLUS, SPINNER, WARNING_CIRCLE, X};

#[component]
pub fn PemakaianBmnForm() -> impl IntoView {
    // Form state
    let (jenis_bmn, set_jenis_bmn) = signal("KENDARAAN_BERMOTOR".to_string());
    let (bmn_nup, set_bmn_nup) = signal("".to_string());
    let (bmn_availability, set_bmn_availability) = signal(None::<BmnAvailabilityResponse>);
    let (checking_availability, set_checking_availability) = signal(false);

    // Auto-filled from `GET /bank-aset/lookup?nup=...` when the operator enters
    // a NUP; pre-loaded into the request payload so the operator doesn't have
    // to retype the catalog data.
    let (bmn_kode_barang, set_bmn_kode_barang) = signal(String::new());
    let (bmn_nama_barang, set_bmn_nama_barang) = signal(String::new());
    let (bmn_merk, set_bmn_merk) = signal(String::new());
    let (bmn_tahun_perolehan, set_bmn_tahun_perolehan) = signal(String::new());

    // Pegawai info
    let (pegawai_nip, set_pegawai_nip) = signal("".to_string());
    let (pegawai_nama, set_pegawai_nama) = signal("".to_string());
    let (pegawai_golongan, set_pegawai_golongan) = signal("".to_string());
    let (pegawai_pangkat, set_pegawai_pangkat) = signal("".to_string());
    let (pegawai_unit_kerja, set_pegawai_unit_kerja) = signal("".to_string());
    let (foto_pegawai, set_foto_pegawai) = signal("".to_string());

    // Additional BMN items (multi-BMN per pegawai)
    let (additional_bmn_items, set_additional_bmn_items) =
        signal::<Vec<CreateBmnItemRequest>>(vec![]);

    // Common fields
    let (tanggal_mulai, set_tanggal_mulai) = signal("".to_string());
    let (tanggal_selesai, set_tanggal_selesai) = signal("".to_string());
    let (keperluan, set_keperluan) = signal("".to_string());
    let (lokasi_pemakaian, set_lokasi_pemakaian) = signal("".to_string());

    // Vehicle-specific fields
    let (no_polisi, set_no_polisi) = signal("".to_string());
    let (no_bpkb, set_no_bpkb) = signal("".to_string());
    let (no_stnk, set_no_stnk) = signal("".to_string());
    let (no_rangka, set_no_rangka) = signal("".to_string());
    let (no_mesin, set_no_mesin) = signal("".to_string());

    // Housing-specific fields
    let (alamat, set_alamat) = signal("".to_string());
    let (luas_tanah, set_luas_tanah) = signal("".to_string());
    let (luas_bangunan, set_luas_bangunan) = signal("".to_string());

    // Laptop-specific fields
    let (serial_number, set_serial_number) = signal("".to_string());

    // UI state
    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    // Cari asetnya di SIMAN, lalu cek ketersediaannya.
    //
    // Urutannya WAJIB berurutan, bukan paralel seperti sebelumnya. Cek
    // ketersediaan butuh `kode_barang`, dan kode barang baru diketahui setelah
    // lookup SIMAN menjawab. Versi lama menembakkan keduanya bersamaan lalu
    // bertanya "apakah NUP 2 sedang dipakai" — pertanyaan yang tak punya
    // jawaban: NUP adalah nomor urut DI DALAM satu satker untuk satu kode
    // barang, jadi 44.017 aset di 553 satker sama-sama ber-NUP 1.
    let check_availability = move |_| {
        let nup = bmn_nup.get();
        if nup.trim().is_empty() {
            return;
        }

        set_checking_availability.set(true);
        set_bmn_availability.set(None);
        set_error.set(None);
        // Reset previous lookup so the operator sees a clean slate while
        // the new query is in flight.
        set_bmn_kode_barang.set(String::new());
        set_bmn_nama_barang.set(String::new());
        set_bmn_merk.set(String::new());
        set_bmn_tahun_perolehan.set(String::new());

        leptos::task::spawn_local(async move {
            let kode_barang = match lookup_by_nup(&nup).await {
                Ok(Some(item)) => {
                    let kode = item.kode_barang.clone().unwrap_or_default();
                    set_bmn_kode_barang.set(kode.clone());
                    set_bmn_nama_barang.set(item.nama_barang.unwrap_or_default());
                    set_bmn_merk.set(item.merk.unwrap_or_default());
                    set_bmn_tahun_perolehan.set(item.tahun_perolehan.unwrap_or_default());
                    kode
                }
                Ok(None) => {
                    set_error.set(Some(format!(
                        "BMN dengan NUP {nup} tidak ditemukan pada data SIMAN satker Anda"
                    )));
                    set_checking_availability.set(false);
                    return;
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal mencari BMN: {e}")));
                    set_checking_availability.set(false);
                    return;
                }
            };

            if kode_barang.is_empty() {
                // Tanpa kode barang asetnya tak bisa dipastikan, dan menebak
                // lebih berbahaya daripada berhenti.
                set_error.set(Some(
                    "Data SIMAN untuk NUP ini tidak memuat kode barang, sehingga asetnya \
                     tidak dapat dipastikan. Hubungi pengelola BMN satker."
                        .to_string(),
                ));
                set_checking_availability.set(false);
                return;
            }

            match check_bmn_availability(&nup, &kode_barang).await {
                Ok(response) => {
                    set_bmn_availability.set(Some(response.data));
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal memeriksa ketersediaan: {e}")));
                }
            }
            set_checking_availability.set(false);
        });
    };

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();

        // Validate availability
        if let Some(availability) = bmn_availability.get() {
            if !availability.is_available {
                set_error.set(Some(format!(
                    "BMN sedang digunakan oleh {} hingga {}",
                    availability.active_permit_holder.unwrap_or_default(),
                    availability.active_permit_expires.unwrap_or_default()
                )));
                return;
            }
        } else {
            set_error.set(Some(
                "Silakan periksa ketersediaan BMN terlebih dahulu".to_string(),
            ));
            return;
        }

        set_loading.set(true);
        set_error.set(None);
        set_success.set(false);

        // Pull satker context from the JWT-backed session. The operator MUST
        // be attached to a satker on the authenc side for the permit to be
        // valid — surface a clear error if the mapping hasn't synced yet.
        let session = AuthService::load_session();
        let satker_id = session.as_ref().and_then(|s| s.satker_id.clone());
        let satker_nama = session.as_ref().and_then(|s| s.satker_nama.clone());
        let satker_id = match satker_id {
            Some(id) => id,
            None => {
                set_error.set(Some(
                    "Anda belum terdaftar di satker manapun (satker_id kosong). \
                     Hubungi admin perlengkapan untuk pemetaan satker."
                        .to_string(),
                ));
                set_loading.set(false);
                return;
            }
        };

        // Build request based on jenis_bmn
        let request = CreateIzinPemakaianRequest {
            pegawai_nip: pegawai_nip.get(),
            pegawai_nama: pegawai_nama.get(),
            pegawai_satker_id: satker_id,
            pegawai_satker_nama: satker_nama.unwrap_or_default(),
            pegawai_jabatan: None,
            pegawai_golongan: if pegawai_golongan.get().is_empty() {
                None
            } else {
                Some(pegawai_golongan.get())
            },
            pegawai_pangkat: if pegawai_pangkat.get().is_empty() {
                None
            } else {
                Some(pegawai_pangkat.get())
            },
            pegawai_unit_kerja: if pegawai_unit_kerja.get().is_empty() {
                None
            } else {
                Some(pegawai_unit_kerja.get())
            },
            foto_pegawai: if foto_pegawai.get().is_empty() {
                None
            } else {
                Some(foto_pegawai.get())
            },
            jenis_bmn: jenis_bmn.get(),
            bmn_nup: bmn_nup.get(),
            bmn_kode_barang: bmn_kode_barang.get(),
            bmn_nama_barang: bmn_nama_barang.get(),
            bmn_merk: if bmn_merk.get().is_empty() {
                None
            } else {
                Some(bmn_merk.get())
            },
            bmn_tahun_perolehan: bmn_tahun_perolehan.get().parse::<i32>().ok(),
            no_polisi: if jenis_bmn.get() == "KENDARAAN_BERMOTOR" {
                Some(no_polisi.get())
            } else {
                None
            },
            no_bpkb: if jenis_bmn.get() == "KENDARAAN_BERMOTOR" {
                Some(no_bpkb.get())
            } else {
                None
            },
            no_stnk: if jenis_bmn.get() == "KENDARAAN_BERMOTOR" {
                Some(no_stnk.get())
            } else {
                None
            },
            no_rangka: if jenis_bmn.get() == "KENDARAAN_BERMOTOR" {
                Some(no_rangka.get())
            } else {
                None
            },
            no_mesin: if jenis_bmn.get() == "KENDARAAN_BERMOTOR" {
                Some(no_mesin.get())
            } else {
                None
            },
            alamat: if jenis_bmn.get() == "RUMAH_NEGARA" {
                Some(alamat.get())
            } else {
                None
            },
            luas_tanah: if jenis_bmn.get() == "RUMAH_NEGARA" {
                luas_tanah.get().parse().ok()
            } else {
                None
            },
            luas_bangunan: if jenis_bmn.get() == "RUMAH_NEGARA" {
                luas_bangunan.get().parse().ok()
            } else {
                None
            },
            serial_number: if jenis_bmn.get() == "LAPTOP" {
                Some(serial_number.get())
            } else {
                None
            },
            spesifikasi: None,
            tanggal_mulai: tanggal_mulai.get(),
            tanggal_selesai: tanggal_selesai.get(),
            keperluan: keperluan.get(),
            lokasi_pemakaian: Some(lokasi_pemakaian.get()),
            file_pendukung: None,
            is_renewal: None,
            previous_permit_id: None,
            additional_bmn_items: additional_bmn_items.get(),
        };

        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match create_pemakaian_bmn(request).await {
                Ok(_response) => {
                    set_success.set(true);
                    set_loading.set(false);

                    // Redirect after success — `navigate` captured at component
                    // level to stay within the reactive scope.
                    gloo_timers::callback::Timeout::new(1500, move || {
                        // navigate() resolves against the router base, so this
                        // must be base-relative (full-path form would double
                        // the /perlengkapan/simpel/v2 prefix and 404).
                        navigate("/pengelolaan/pemakaian", Default::default());
                    })
                    .forget();
                }
                Err(e) => {
                    set_error.set(Some(format!("Gagal mengajukan permohonan: {}", e)));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="max-w-4xl mx-auto p-6 bg-white rounded-xl shadow-sm border border-gray-100">
            <h2 class="text-2xl font-bold text-gray-800 mb-6">"Permohonan Izin Pemakaian BMN"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2">
                    <AppIcon icon=CHECK_CIRCLE />
                    "Permohonan izin berhasil diajukan!"
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div
                    data-testid="form-error"
                    class="mb-4 p-4 bg-red-50 text-red-700 rounded-lg border border-red-100 flex items-center gap-2"
                >
                    <AppIcon icon=WARNING_CIRCLE />
                    {error.get()}
                </div>
            </Show>

            <form on:submit=on_submit class="space-y-6">
                // Jenis BMN Selection
                <div class="bg-blue-50 p-4 rounded-lg">
                    <label class="block text-sm font-medium text-gray-700 mb-2">"Jenis BMN"</label>
                    <select
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                        prop:value=move || jenis_bmn.get()
                        on:change=move |ev| set_jenis_bmn.set(event_target_value(&ev))
                    >
                        <option value="KENDARAAN_BERMOTOR">"Kendaraan Bermotor"</option>
                        <option value="RUMAH_NEGARA">"Rumah Negara"</option>
                        <option value="LAPTOP">"Laptop/Komputer"</option>
                        <option value="LAINNYA">"Lainnya"</option>
                    </select>
                </div>

                // Pegawai Information.
                // NOTE: Operator Satker fills this on behalf of employee.
                // TODO(pegawai-lookup): replace manual NIP entry with an
                // autocomplete that hits
                // `GET /admin/pegawai?satker_id={session.satker_id}&q={query}`
                // (driven by the MySIMKARI sync in layanan/integrasi). Until
                // that endpoint + the satker→pegawai cache land, operators
                // type NIP/nama by hand below.
                <div class="border-t pt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">
                        "Informasi Pegawai yang Akan Menggunakan BMN"
                        <span class="text-sm text-gray-500 ml-2">
                            "(Diisi oleh Operator Satker)"
                        </span>
                    </h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "NIP"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="NIP Pegawai"
                                prop:value=move || pegawai_nip.get()
                                on:input=move |ev| set_pegawai_nip.set(event_target_value(&ev))
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Nama Pegawai"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Nama Lengkap"
                                prop:value=move || pegawai_nama.get()
                                on:input=move |ev| set_pegawai_nama.set(event_target_value(&ev))
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Golongan"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Contoh: III/c"
                                prop:value=move || pegawai_golongan.get()
                                on:input=move |ev| set_pegawai_golongan.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Pangkat"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Contoh: Penata"
                                prop:value=move || pegawai_pangkat.get()
                                on:input=move |ev| set_pegawai_pangkat.set(event_target_value(&ev))
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Unit Kerja"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Unit kerja pegawai"
                                prop:value=move || pegawai_unit_kerja.get()
                                on:input=move |ev| {
                                    set_pegawai_unit_kerja.set(event_target_value(&ev))
                                }
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Foto Pegawai (URL)"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="URL foto pegawai"
                                prop:value=move || foto_pegawai.get()
                                on:input=move |ev| set_foto_pegawai.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                </div>

                // BMN Selection with Availability Check
                <div class="border-t pt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"Pilih BMN"</h3>
                    <div class="flex gap-2">
                        <div class="flex-1">
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "NUP BMN"
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Nomor Urut Pendaftaran"
                                prop:value=move || bmn_nup.get()
                                on:input=move |ev| set_bmn_nup.set(event_target_value(&ev))
                                required
                            />
                        </div>
                        <div class="flex items-end">
                            <button
                                type="button"
                                class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
                                on:click=check_availability
                                prop:disabled=move || {
                                    checking_availability.get() || bmn_nup.get().is_empty()
                                }
                            >
                                <Show
                                    when=move || checking_availability.get()
                                    fallback=|| view! { "Cek Ketersediaan" }
                                >
                                    <span class="fa-spin">
                                        <AppIcon icon=SPINNER />
                                    </span>
                                </Show>
                            </button>
                        </div>
                    </div>

                    // Identitas aset yang benar-benar sedang diperiksa.
                    // Dulu kode barang & nama barang hanya diisi diam-diam ke
                    // payload dan tak pernah ditampilkan, jadi operator tak
                    // punya cara melihat aset mana yang dimaksud sistem.
                    <Show when=move || !bmn_kode_barang.get().is_empty()>
                        <div
                            data-testid="bmn-identitas"
                            class="mt-2 p-3 bg-gray-50 border border-gray-200 rounded-lg text-sm text-gray-700"
                        >
                            <span class="font-medium">{move || bmn_nama_barang.get()}</span>
                            " — kode barang "
                            <span class="font-mono">{move || bmn_kode_barang.get()}</span>
                            ", NUP "
                            <span class="font-mono">{move || bmn_nup.get()}</span>
                        </div>
                    </Show>

                    // Availability Status
                    <Show when=move || {
                        bmn_availability.get().is_some()
                    }>
                        {move || {
                            let availability = bmn_availability.get().unwrap();
                            if availability.is_available {
                                view! {
                                    <div
                                        data-testid="bmn-ketersediaan"
                                        class="mt-2 p-3 bg-green-50 text-green-700 rounded-lg border border-green-100 flex items-center gap-2"
                                    >
                                        <AppIcon icon=CHECK_CIRCLE />
                                        "BMN tersedia untuk digunakan"
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div
                                        data-testid="bmn-ketersediaan"
                                        class="mt-2 p-3 bg-red-50 text-red-700 rounded-lg border border-red-100"
                                    >
                                        <div class="flex items-center gap-2 mb-1">
                                            <AppIcon icon=WARNING_CIRCLE />
                                            <span class="font-semibold">"BMN sedang digunakan"</span>
                                        </div>
                                        <p class="text-sm">
                                            "Pemegang: "
                                            {availability.active_permit_holder.unwrap_or_default()}
                                        </p>
                                        <p class="text-sm">
                                            "Berlaku hingga: "
                                            {availability.active_permit_expires.unwrap_or_default()}
                                        </p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Show>
                </div>

                // Type-specific fields
                <div class="border-t pt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"Detail BMN"</h3>

                    // Vehicle-specific fields
                    <Show when=move || jenis_bmn.get() == "KENDARAAN_BERMOTOR">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nomor Polisi" <span class="text-red-500">"*"</span>
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    placeholder="B 1234 XYZ"
                                    prop:value=move || no_polisi.get()
                                    on:input=move |ev| set_no_polisi.set(event_target_value(&ev))
                                    required
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nomor BPKB"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    prop:value=move || no_bpkb.get()
                                    on:input=move |ev| set_no_bpkb.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nomor STNK"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    prop:value=move || no_stnk.get()
                                    on:input=move |ev| set_no_stnk.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nomor Rangka"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    prop:value=move || no_rangka.get()
                                    on:input=move |ev| set_no_rangka.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Nomor Mesin"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    prop:value=move || no_mesin.get()
                                    on:input=move |ev| set_no_mesin.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </Show>

                    // Housing-specific fields
                    <Show when=move || jenis_bmn.get() == "RUMAH_NEGARA">
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">
                                    "Alamat" <span class="text-red-500">"*"</span>
                                </label>
                                <textarea
                                    class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                    rows="3"
                                    placeholder="Alamat lengkap rumah negara"
                                    prop:value=move || alamat.get()
                                    on:input=move |ev| set_alamat.set(event_target_value(&ev))
                                    required
                                ></textarea>
                            </div>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Luas Tanah (m²)" <span class="text-red-500">"*"</span>
                                    </label>
                                    <input
                                        type="number"
                                        step="0.01"
                                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                        prop:value=move || luas_tanah.get()
                                        on:input=move |ev| {
                                            set_luas_tanah.set(event_target_value(&ev))
                                        }
                                        required
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">
                                        "Luas Bangunan (m²)" <span class="text-red-500">"*"</span>
                                    </label>
                                    <input
                                        type="number"
                                        step="0.01"
                                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                        prop:value=move || luas_bangunan.get()
                                        on:input=move |ev| {
                                            set_luas_bangunan.set(event_target_value(&ev))
                                        }
                                        required
                                    />
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Laptop-specific fields
                    <Show when=move || jenis_bmn.get() == "LAPTOP">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Serial Number" <span class="text-red-500">"*"</span>
                            </label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                placeholder="Serial number laptop"
                                prop:value=move || serial_number.get()
                                on:input=move |ev| set_serial_number.set(event_target_value(&ev))
                                required
                            />
                        </div>
                    </Show>
                </div>

                // Common fields
                <div class="border-t pt-6">
                    <h3 class="text-lg font-semibold text-gray-800 mb-4">"Periode Pemakaian"</h3>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Tanggal Mulai"
                            </label>
                            <input
                                type="date"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                prop:value=move || tanggal_mulai.get()
                                on:input=move |ev| set_tanggal_mulai.set(event_target_value(&ev))
                                required
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">
                                "Tanggal Selesai"
                            </label>
                            <input
                                type="date"
                                class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                                prop:value=move || tanggal_selesai.get()
                                on:input=move |ev| set_tanggal_selesai.set(event_target_value(&ev))
                                required
                            />
                        </div>
                    </div>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Keperluan"</label>
                    <textarea
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                        rows="4"
                        placeholder="Jelaskan keperluan penggunaan BMN (minimal 10 karakter)"
                        prop:value=move || keperluan.get()
                        on:input=move |ev| set_keperluan.set(event_target_value(&ev))
                        required
                    ></textarea>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 mb-1">
                        "Lokasi Pemakaian"
                    </label>
                    <input
                        type="text"
                        class="w-full px-4 py-2 border rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none"
                        placeholder="Lokasi penggunaan BMN"
                        prop:value=move || lokasi_pemakaian.get()
                        on:input=move |ev| set_lokasi_pemakaian.set(event_target_value(&ev))
                    />
                </div>

                // Additional BMN Items (multi-BMN per pegawai)
                <div class="border-t pt-6">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-semibold text-gray-800">
                            "BMN Tambahan"
                            <span class="text-sm font-normal text-gray-500 ml-2">
                                "(Opsional, untuk pegawai yang menggunakan lebih dari 1 BMN)"
                            </span>
                        </h3>
                        <button
                            type="button"
                            class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 flex items-center gap-1"
                            on:click=move |_| {
                                let mut items = additional_bmn_items.get();
                                items
                                    .push(CreateBmnItemRequest {
                                        bmn_nup: String::new(),
                                        bmn_kode_barang: String::new(),
                                        bmn_nama_barang: String::new(),
                                        bmn_merk: None,
                                        bmn_tahun_perolehan: None,
                                        bmn_kondisi: None,
                                        detail_bmn: None,
                                        keterangan: None,
                                    });
                                set_additional_bmn_items.set(items);
                            }
                        >
                            <AppIcon icon=PLUS />
                            "Tambah BMN"
                        </button>
                    </div>

                    {move || {
                        let items = additional_bmn_items.get();
                        if items.is_empty() {
                            view! { <div></div> }.into_any()
                        } else {
                            view! {
                                <div class="space-y-3">
                                    {items
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, _item)| {
                                            view! {
                                                <div class="p-4 bg-gray-50 rounded-lg border relative">
                                                    <button
                                                        type="button"
                                                        class="absolute top-2 right-2 text-red-500 hover:text-red-700"
                                                        on:click=move |_| {
                                                            let mut items = additional_bmn_items.get();
                                                            if idx < items.len() {
                                                                items.remove(idx);
                                                                set_additional_bmn_items.set(items);
                                                            }
                                                        }
                                                    >
                                                        <AppIcon icon=X />
                                                    </button>
                                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
                                                        <div>
                                                            <label class="block text-xs font-medium text-gray-600 mb-1">
                                                                "Kode Barang"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="w-full px-3 py-1.5 text-sm border rounded-lg"
                                                                placeholder="Kode barang"
                                                                on:input=move |ev| {
                                                                    let mut items = additional_bmn_items.get();
                                                                    if let Some(item) = items.get_mut(idx) {
                                                                        item.bmn_kode_barang = event_target_value(&ev);
                                                                    }
                                                                    set_additional_bmn_items.set(items);
                                                                }
                                                            />
                                                        </div>
                                                        <div>
                                                            <label class="block text-xs font-medium text-gray-600 mb-1">
                                                                "Nama Barang"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="w-full px-3 py-1.5 text-sm border rounded-lg"
                                                                placeholder="Nama barang"
                                                                on:input=move |ev| {
                                                                    let mut items = additional_bmn_items.get();
                                                                    if let Some(item) = items.get_mut(idx) {
                                                                        item.bmn_nama_barang = event_target_value(&ev);
                                                                    }
                                                                    set_additional_bmn_items.set(items);
                                                                }
                                                            />
                                                        </div>
                                                        <div>
                                                            <label class="block text-xs font-medium text-gray-600 mb-1">
                                                                "NUP"
                                                            </label>
                                                            <input
                                                                type="text"
                                                                class="w-full px-3 py-1.5 text-sm border rounded-lg"
                                                                placeholder="NUP"
                                                                on:input=move |ev| {
                                                                    let mut items = additional_bmn_items.get();
                                                                    if let Some(item) = items.get_mut(idx) {
                                                                        item.bmn_nup = event_target_value(&ev);
                                                                    }
                                                                    set_additional_bmn_items.set(items);
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>

                // Submit buttons
                <div class="pt-6 flex justify-end gap-3 border-t">
                    <a
                        href=routes::path::PENGELOLAAN_PEMAKAIAN
                        class="px-6 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                    >
                        "Batal"
                    </a>
                    <button
                        type="submit"
                        class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center gap-2"
                        prop:disabled=move || {
                            loading.get()
                                || bmn_availability.get().map(|a| !a.is_available).unwrap_or(true)
                        }
                    >
                        <Show
                            when=move || loading.get()
                            fallback=|| view! { <AppIcon icon=PAPER_PLANE_TILT /> }
                        >
                            <span class="fa-spin">
                                <AppIcon icon=SPINNER />
                            </span>
                        </Show>
                        "Ajukan Permohonan"
                    </button>
                </div>
            </form>
        </div>
    }
}
