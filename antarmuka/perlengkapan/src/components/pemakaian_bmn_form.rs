//! # Pemakaian BMN Form Component
//!
//! Dynamic form for creating BMN usage permits with type-specific fields.
//!
//! **IMPORTANT - ROLE STRUCTURE:**
//! - This form is ONLY accessible by Operator Satker
//! - Operator Satker creates permits ON BEHALF OF employees (pegawai)
//! - There is NO self-service for employees in perlengkapan domain
//! - Operator Satker enters the pegawai's NIP and the identity fills itself
//!   from `integrasi.mysimkari_pegawai`; the same holds for every BMN row
//!
//! Requirements: REQ-P001, REQ-P002, REQ-P014

use crate::api::{
    BmnAvailabilityResponse, CreateBmnItemRequest, CreateIzinPemakaianRequest,
    check_bmn_availability, create_pemakaian_bmn,
};
use crate::features::auth::AuthService;
use crate::routes;
use leptos::prelude::*;

use crate::api::bank_aset::BankAsetItem;
use crate::api::pemakaian_bmn::CekPegawaiResponse;
use crate::components::reference_picker::{AsetPicker, PegawaiPicker, merk_tipe};
use leptos_router::hooks::use_navigate;
use lib_ui::components::icon::AppIcon;
use lib_perlengkapan::jenis_bmn::{jenis_bmn_dari_jenis_aset, label_jenis_bmn};
use phosphor_leptos::{CHECK_CIRCLE, PAPER_PLANE_TILT, PLUS, SPINNER, WARNING_CIRCLE, X};

/// Satu BMN dalam permohonan.
///
/// Sejak V014 tidak ada lagi "BMN utama" dan "BMN tambahan": permohonan adalah
/// satu pegawai dengan DAFTAR BMN, dan baris pertama sama kedudukannya dengan
/// baris kelima. Dulu BMN pertama punya formulirnya sendiri (jenis, nomor
/// polisi, serial number, lokasi) sementara BMN tambahan hanya bisa memilih
/// aset — jadi kendaraan kedua tak punya tempat untuk nomor polisinya, dan
/// tiap fitur yang menyentuh izin hanya pernah mengenal BMN pertama.
///
/// Isi baris tinggal di sinyalnya sendiri dan daftar hanya menyimpan kuncinya:
/// dengan `For` berkunci, identitas melekat pada barisnya, bukan posisinya.
/// Merender lewat indeks posisi pernah terbukti salah di sini — hapus baris
/// pertama dari "AAA","BBB", dan baris tersisa TETAP MENAMPILKAN "AAA"
/// sementara muatan yang dikirim berisi "BBB".
#[derive(Clone, Copy)]
struct BarisBmn {
    kunci: usize,
    isi: RwSignal<CreateBmnItemRequest>,
    ketersediaan: RwSignal<Option<BmnAvailabilityResponse>>,
    memeriksa: RwSignal<bool>,
    galat: RwSignal<Option<String>>,
}

impl BarisBmn {
    fn baru(kunci: usize) -> Self {
        Self {
            kunci,
            isi: RwSignal::new(baris_kosong()),
            ketersediaan: RwSignal::new(None),
            memeriksa: RwSignal::new(false),
            galat: RwSignal::new(None),
        }
    }
}

/// Baris kosong. `jenis_bmn` kosong sampai aset dipilih — jenisnya diturunkan
/// dari aset itu, bukan dipilih terpisah.
fn baris_kosong() -> CreateBmnItemRequest {
    CreateBmnItemRequest {
        jenis_bmn: String::new(),
        bmn_nup: String::new(),
        bmn_kode_barang: String::new(),
        bmn_nama_barang: String::new(),
        bmn_merk: None,
        bmn_tipe: None,
        bmn_tahun_perolehan: None,
        bmn_kondisi: None,
        no_polisi: None,
        no_bpkb: None,
        no_stnk: None,
        no_rangka: None,
        no_mesin: None,
        alamat: None,
        luas_tanah: None,
        luas_bangunan: None,
        serial_number: None,
        spesifikasi: None,
        lokasi_pemakaian: None,
        tanggal_mulai: None,
        tanggal_selesai: None,
        detail_bmn: None,
        keterangan: None,
    }
}

/// String kosong bukan nilai; SIMAN memakainya untuk "tidak tercatat".
fn bukan_kosong(s: String) -> Option<String> {
    let s = s.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

const KELAS_ISIAN: &str = "w-full px-4 py-2 rounded-lg border border-white/10 bg-slate-900/70 text-slate-100 placeholder:text-slate-500 outline-none transition-colors hover:border-white/20 focus:border-gold-400/60 focus:ring-2 focus:ring-gold-400/40";

/// Satu isian teks/angka/tanggal yang membaca dan menulis satu field baris.
///
/// `id` WAJIB unik per baris: label `for` yang kembar membuat label baris kedua
/// memfokuskan kotak baris pertama.
fn isian(
    id: String,
    label: &'static str,
    tipe: &'static str,
    wajib: bool,
    nilai: impl Fn() -> String + Send + Sync + 'static,
    tulis: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div>
            <label for=id.clone() class="block text-sm font-medium text-slate-200 mb-1">
                {label}
                {wajib.then(|| view! { <span class="text-danger-400">" *"</span> })}
            </label>
            <input
                id=id
                type=tipe
                step=(tipe == "number").then_some("0.01")
                class=KELAS_ISIAN
                prop:value=nilai
                on:input=move |ev| tulis(event_target_value(&ev))
                required=wajib
            />
        </div>
    }
}

/// Satu kartu BMN: pemilih aset, ketersediaannya, dan isian yang dibutuhkan
/// jenis asetnya.
#[component]
fn KartuBmn(
    baris: BarisBmn,
    /// Nomor urut 1-based — urutan ini pula yang dicetak di halaman 2 SK.
    #[prop(into)]
    nomor: Signal<usize>,
    #[prop(into)] bisa_dihapus: Signal<bool>,
    #[prop(into)] on_hapus: Callback<usize>,
) -> impl IntoView {
    let BarisBmn {
        kunci,
        isi,
        ketersediaan,
        memeriksa,
        galat,
    } = baris;
    let slug = format!("aset-{kunci}");
    let id = move |nama: &str| format!("{nama}-{kunci}");

    // Aset dipilih dari SIMAN, lalu ketersediaannya diperiksa — berurutan,
    // karena cek ketersediaan butuh kode barang yang baru diketahui setelah
    // aset dipilih. NUP saja tidak mengidentifikasi aset: 44.017 aset di 553
    // satker sama-sama ber-NUP 1.
    let on_pick = Callback::new(move |picked: Option<BankAsetItem>| {
        galat.set(None);
        ketersediaan.set(None);
        let Some(aset) = picked else {
            isi.update(|it| {
                // Kosongkan identitas aset, pertahankan jangka waktu khusus
                // yang mungkin sudah diisi untuk baris ini.
                let (mulai, selesai) = (it.tanggal_mulai.take(), it.tanggal_selesai.take());
                *it = baris_kosong();
                it.tanggal_mulai = mulai;
                it.tanggal_selesai = selesai;
            });
            return;
        };

        let nup = aset.no_aset.clone();
        let kode_barang = aset.kode_barang.clone().unwrap_or_default();
        let jenis = jenis_bmn_dari_jenis_aset(&aset.kategori_aset).to_string();
        isi.update(|it| {
            it.jenis_bmn = jenis.clone();
            it.bmn_nup = nup.clone();
            it.bmn_kode_barang = kode_barang.clone();
            it.bmn_nama_barang = aset.nama_aset.clone().unwrap_or_default();
            it.bmn_merk = bukan_kosong(merk_tipe(&aset));
            it.bmn_tahun_perolehan = tahun_perolehan(&aset).parse().ok();
            it.bmn_kondisi = aset.kondisi.clone().and_then(bukan_kosong);
            // SIMAN mencatat alamat aset; untuk rumah negara itulah alamat
            // izinnya, jadi diisikan alih-alih diketik ulang (tetap bisa
            // dikoreksi). Field khusus jenis lain dikosongkan — mereka milik
            // aset yang baru saja diganti.
            it.alamat = if jenis == "RUMAH_NEGARA" {
                aset.lokasi.clone().and_then(bukan_kosong)
            } else {
                None
            };
            it.no_polisi = None;
            it.no_bpkb = None;
            it.no_stnk = None;
            it.no_rangka = None;
            it.no_mesin = None;
            it.luas_tanah = None;
            it.luas_bangunan = None;
            it.serial_number = None;
        });

        if kode_barang.is_empty() {
            // Tanpa kode barang asetnya tak bisa dipastikan, dan menebak lebih
            // berbahaya daripada berhenti.
            galat.set(Some(
                "Data SIMAN untuk aset ini tidak memuat kode barang, sehingga asetnya \
                 tidak dapat dipastikan. Hubungi pengelola BMN satker."
                    .to_string(),
            ));
            return;
        }

        memeriksa.set(true);
        leptos::task::spawn_local(async move {
            match check_bmn_availability(&nup, &kode_barang).await {
                Ok(response) => ketersediaan.set(Some(response.data)),
                Err(e) => galat.set(Some(format!("Gagal memeriksa ketersediaan: {e}"))),
            }
            memeriksa.set(false);
        });
    });

    let jenis = Memo::new(move |_| isi.with(|it| it.jenis_bmn.clone()));
    let sudah_dipilih = move || isi.with(|it| !it.bmn_nup.is_empty());

    view! {
        <div
            data-testid=format!("kartu-bmn-{kunci}")
            class="relative rounded-lg border border-white/10 bg-white/[0.03] p-4 space-y-4"
        >
            <div class="flex items-center justify-between">
                <h4 class="text-sm font-semibold text-slate-200">
                    {move || format!("BMN {}", nomor.get())}
                    <Show when=move || !jenis.get().is_empty()>
                        <span
                            data-testid=format!("jenis-bmn-{kunci}")
                            class="ml-2 rounded-full bg-info-500/15 px-2 py-0.5 text-xs font-medium text-info-300"
                        >
                            {move || label_jenis_bmn(&jenis.get())}
                        </span>
                    </Show>
                </h4>
                <Show when=move || bisa_dihapus.get()>
                    <button
                        type="button"
                        aria-label="Hapus BMN"
                        class="text-danger-400 hover:text-danger-300"
                        on:click=move |_| on_hapus.run(kunci)
                    >
                        <AppIcon icon=X />
                    </button>
                </Show>
            </div>

            <AsetPicker slug=slug label="Cari BMN".to_string() on_pick=on_pick />

            <Show when=move || memeriksa.get()>
                <p class="flex items-center gap-2 text-sm text-slate-400">
                    <span class="fa-spin">
                        <AppIcon icon=SPINNER />
                    </span>
                    "Memeriksa ketersediaan…"
                </p>
            </Show>
            <Show when=move || galat.get().is_some()>
                <p class="text-sm text-danger-300">{move || galat.get()}</p>
            </Show>
            {move || {
                ketersediaan
                    .get()
                    .map(|k| {
                        let testid = format!("bmn-ketersediaan-{kunci}");
                        if k.is_available {
                            view! {
                                <div
                                    data-testid=testid
                                    class="p-3 bg-success-500/10 text-success-300 rounded-lg border border-success-500/20 flex items-center gap-2"
                                >
                                    <AppIcon icon=CHECK_CIRCLE />
                                    "BMN tersedia untuk digunakan"
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div
                                    data-testid=testid
                                    class="p-3 bg-danger-500/10 text-danger-300 rounded-lg border border-danger-500/20"
                                >
                                    <div class="flex items-center gap-2 mb-1">
                                        <AppIcon icon=WARNING_CIRCLE />
                                        <span class="font-semibold">"BMN sedang digunakan"</span>
                                    </div>
                                    <p class="text-sm">
                                        "Pemegang: " {k.active_permit_holder.unwrap_or_default()}
                                    </p>
                                    <p class="text-sm">
                                        "Berlaku hingga: " {k.active_permit_expires.unwrap_or_default()}
                                    </p>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}

            // Isian yang dibutuhkan jenis asetnya. Hanya yang TIDAK dicatat
            // SIMAN: nomor polisi, BPKB, serial number, dan luas tidak ada di
            // `integrasi.siman_aset`, jadi memang harus diisi di sini.
            <Show when=move || jenis.get() == "KENDARAAN_BERMOTOR">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {isian(id("no-polisi"), "Nomor Polisi", "text", true,
                        move || isi.with(|it| it.no_polisi.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.no_polisi = bukan_kosong(v)))}
                    {isian(id("no-bpkb"), "Nomor BPKB", "text", true,
                        move || isi.with(|it| it.no_bpkb.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.no_bpkb = bukan_kosong(v)))}
                    {isian(id("no-stnk"), "Nomor STNK", "text", false,
                        move || isi.with(|it| it.no_stnk.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.no_stnk = bukan_kosong(v)))}
                    {isian(id("no-rangka"), "Nomor Rangka", "text", false,
                        move || isi.with(|it| it.no_rangka.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.no_rangka = bukan_kosong(v)))}
                    {isian(id("no-mesin"), "Nomor Mesin", "text", false,
                        move || isi.with(|it| it.no_mesin.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.no_mesin = bukan_kosong(v)))}
                </div>
            </Show>
            <Show when=move || jenis.get() == "RUMAH_NEGARA">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="md:col-span-2">
                        {isian(id("alamat"), "Alamat", "text", true,
                            move || isi.with(|it| it.alamat.clone().unwrap_or_default()),
                            move |v| isi.update(|it| it.alamat = bukan_kosong(v)))}
                    </div>
                    {isian(id("luas-tanah"), "Luas Tanah (m²)", "number", true,
                        move || isi.with(|it| it.luas_tanah.map(|n| n.to_string()).unwrap_or_default()),
                        move |v| isi.update(|it| it.luas_tanah = v.trim().parse().ok()))}
                    {isian(id("luas-bangunan"), "Luas Bangunan (m²)", "number", true,
                        move || isi.with(|it| it.luas_bangunan.map(|n| n.to_string()).unwrap_or_default()),
                        move |v| isi.update(|it| it.luas_bangunan = v.trim().parse().ok()))}
                </div>
            </Show>
            <Show when=move || jenis.get() == "LAPTOP">
                {isian(id("serial-number"), "Serial Number", "text", true,
                    move || isi.with(|it| it.serial_number.clone().unwrap_or_default()),
                    move |v| isi.update(|it| it.serial_number = bukan_kosong(v)))}
            </Show>

            <Show when=sudah_dipilih>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="md:col-span-3">
                        {isian(id("lokasi"), "Lokasi Pemakaian", "text", false,
                            move || isi.with(|it| it.lokasi_pemakaian.clone().unwrap_or_default()),
                            move |v| isi.update(|it| it.lokasi_pemakaian = bukan_kosong(v)))}
                    </div>
                    // Kosong = mengikuti periode izin. Diisi bila BMN ini
                    // dipakai dalam jangka waktu berbeda dari yang lain.
                    {isian(id("mulai"), "Mulai (jika berbeda)", "date", false,
                        move || isi.with(|it| it.tanggal_mulai.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.tanggal_mulai = bukan_kosong(v)))}
                    {isian(id("selesai"), "Selesai (jika berbeda)", "date", false,
                        move || isi.with(|it| it.tanggal_selesai.clone().unwrap_or_default()),
                        move |v| isi.update(|it| it.tanggal_selesai = bukan_kosong(v)))}
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn PemakaianBmnForm() -> impl IntoView {
    // Satu-satunya sumber identitas pegawai di formulir ini: apa pun yang
    // dipilih di `PegawaiPicker` mengisi sinyal-sinyal di bawah, dan tidak ada
    // kotak untuk mengetiknya sendiri.
    //
    // `golongan` sengaja dibiarkan kosong. Sumbernya satu kolom, `golpang`,
    // yang menggabungkan pangkat dan golongan — dan bentuknya tidak konsisten:
    // dari 21.328 baris di staging, 21.141 berpola "Pangkat / (Gol)" sementara
    // 187 sisanya justru KEBALIKANNYA ("III/d (Sena Wira)"). Memecahnya berarti
    // menebak, dan tebakan yang salah menaruh golongan di kolom pangkat pada
    // ratusan pegawai. Nilainya dikirim utuh sebagai pangkat.
    let (pegawai_nip, set_pegawai_nip) = signal(String::new());
    let (pegawai_nama, set_pegawai_nama) = signal(String::new());
    let (pegawai_pangkat, set_pegawai_pangkat) = signal(String::new());
    let (pegawai_unit_kerja, set_pegawai_unit_kerja) = signal(String::new());
    let (foto_pegawai, set_foto_pegawai) = signal(String::new());

    let on_pick_pegawai = Callback::new(move |picked: Option<CekPegawaiResponse>| {
        let p = picked.map(|r| r.pegawai);
        set_pegawai_nip.set(p.as_ref().map(|p| p.nip.clone()).unwrap_or_default());
        set_pegawai_nama.set(p.as_ref().and_then(|p| p.nama.clone()).unwrap_or_default());
        set_pegawai_pangkat.set(p.as_ref().and_then(|p| p.pangkat.clone()).unwrap_or_default());
        set_pegawai_unit_kerja.set(
            p.as_ref()
                .and_then(|p| p.nama_satker.clone())
                .unwrap_or_default(),
        );
        set_foto_pegawai.set(p.as_ref().and_then(|p| p.foto.clone()).unwrap_or_default());
    });

    // Daftar BMN. Dimulai dengan SATU baris: permohonan tanpa BMN bukan
    // permohonan. Kunci dihitung maju dan tak pernah dipakai ulang.
    let daftar = RwSignal::new(vec![BarisBmn::baru(0)]);
    let kunci_berikutnya = StoredValue::new(1usize);

    let (tanggal_mulai, set_tanggal_mulai) = signal(String::new());
    let (tanggal_selesai, set_tanggal_selesai) = signal(String::new());
    let (keperluan, set_keperluan) = signal(String::new());

    let (error, set_error) = signal(None::<String>);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);
    let navigate = use_navigate();

    let hapus = Callback::new(move |kunci: usize| {
        daftar.update(|d| {
            if d.len() > 1 {
                d.retain(|b| b.kunci != kunci);
            }
        });
    });

    // Tombol kirim hanya hidup bila SETIAP BMN sudah dipilih dan dinyatakan
    // tersedia. Satu BMN yang sedang dipakai orang lain menggagalkan seluruh
    // permohonan di server, jadi menahannya di sini lebih jujur.
    let semua_siap = move || {
        daftar.with(|d| {
            d.iter().all(|b| {
                b.ketersediaan
                    .with(|k| k.as_ref().is_some_and(|k| k.is_available))
            })
        })
    };

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_success.set(false);

        let baris: Vec<BarisBmn> = daftar.get();
        let mut items = Vec::with_capacity(baris.len());
        let mut terlihat = std::collections::HashSet::new();
        for (i, b) in baris.iter().enumerate() {
            let item = b.isi.get();
            let nomor = i + 1;
            if item.bmn_nup.is_empty() {
                set_error.set(Some(format!("BMN {nomor} belum dipilih")));
                return;
            }
            match b.ketersediaan.get() {
                Some(k) if k.is_available => {}
                Some(k) => {
                    set_error.set(Some(format!(
                        "BMN {nomor} sedang digunakan oleh {} hingga {}",
                        k.active_permit_holder.unwrap_or_default(),
                        k.active_permit_expires.unwrap_or_default()
                    )));
                    return;
                }
                None => {
                    set_error.set(Some(format!(
                        "Ketersediaan BMN {nomor} belum diperiksa"
                    )));
                    return;
                }
            }
            if !terlihat.insert((item.bmn_nup.clone(), item.bmn_kode_barang.clone())) {
                set_error.set(Some(format!(
                    "BMN {nomor} sudah tercantum di baris lain pada permohonan ini"
                )));
                return;
            }
            items.push(item);
        }

        // Pull satker context from the JWT-backed session. The operator MUST
        // be attached to a satker on the authenc side for the permit to be
        // valid — surface a clear error if the mapping hasn't synced yet.
        let session = AuthService::load_session();
        let satker_id = session.as_ref().and_then(|s| s.satker_id.clone());
        let satker_nama = session.as_ref().and_then(|s| s.satker_nama.clone());
        let Some(satker_id) = satker_id else {
            set_error.set(Some(
                "Anda belum terdaftar di satker manapun (satker_id kosong). \
                 Hubungi admin perlengkapan untuk pemetaan satker."
                    .to_string(),
            ));
            return;
        };

        let request = CreateIzinPemakaianRequest {
            pegawai_nip: pegawai_nip.get(),
            pegawai_nama: pegawai_nama.get(),
            pegawai_satker_id: satker_id,
            pegawai_satker_nama: satker_nama.unwrap_or_default(),
            pegawai_jabatan: None,
            pegawai_golongan: None,
            pegawai_pangkat: bukan_kosong(pegawai_pangkat.get()),
            pegawai_unit_kerja: bukan_kosong(pegawai_unit_kerja.get()),
            foto_pegawai: bukan_kosong(foto_pegawai.get()),
            tanggal_mulai: tanggal_mulai.get(),
            tanggal_selesai: tanggal_selesai.get(),
            keperluan: keperluan.get(),
            file_pendukung: None,
            is_renewal: None,
            previous_permit_id: None,
            items,
        };

        set_loading.set(true);
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match create_pemakaian_bmn(request).await {
                Ok(_response) => {
                    set_success.set(true);
                    set_loading.set(false);
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
        <div class="max-w-4xl mx-auto p-6 bg-surface-panel rounded-xl shadow-sm border border-white/[0.06]">
            <h2 class="text-2xl font-bold text-slate-100 mb-6">"Permohonan Izin Pemakaian BMN"</h2>

            <Show when=move || success.get()>
                <div class="mb-4 p-4 bg-success-500/10 text-success-300 rounded-lg border border-success-500/20 flex items-center gap-2">
                    <AppIcon icon=CHECK_CIRCLE />
                    "Permohonan izin berhasil diajukan!"
                </div>
            </Show>

            <Show when=move || error.get().is_some()>
                <div
                    data-testid="form-error"
                    class="mb-4 p-4 bg-danger-500/10 text-danger-300 rounded-lg border border-danger-500/20 flex items-center gap-2"
                >
                    <AppIcon icon=WARNING_CIRCLE />
                    {error.get()}
                </div>
            </Show>

            <form on:submit=on_submit class="space-y-6">
                // Pegawai — identitasnya datang dari data kepegawaian
                // (`cek-pegawai/{nip}`), tidak diketik ulang.
                <div>
                    <h3 class="text-lg font-semibold text-slate-100 mb-4">
                        "Pegawai yang Akan Menggunakan BMN"
                    </h3>
                    <PegawaiPicker on_pick=on_pick_pegawai />
                </div>

                <div class="border-t border-white/[0.06] pt-6">
                    <h3 class="text-lg font-semibold text-slate-100 mb-1">"Periode Izin"</h3>
                    <p class="text-sm text-slate-400 mb-4">
                        "Berlaku untuk setiap BMN di bawah, kecuali yang diberi jangka waktu sendiri."
                    </p>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {isian("periode-mulai".to_string(), "Tanggal Mulai", "date", true,
                            move || tanggal_mulai.get(), move |v| set_tanggal_mulai.set(v))}
                        {isian("periode-selesai".to_string(), "Tanggal Selesai", "date", true,
                            move || tanggal_selesai.get(), move |v| set_tanggal_selesai.set(v))}
                    </div>
                </div>

                <div>
                    <label for="keperluan" class="block text-sm font-medium text-slate-200 mb-1">"Keperluan"</label>
                    <textarea
                        id="keperluan"
                        class=KELAS_ISIAN
                        rows="4"
                        placeholder="Jelaskan keperluan penggunaan BMN (minimal 10 karakter)"
                        prop:value=move || keperluan.get()
                        on:input=move |ev| set_keperluan.set(event_target_value(&ev))
                        required
                    ></textarea>
                </div>

                // Daftar BMN — semua setara. Halaman 2 SK mencetak baris-baris
                // ini dengan nomor urut yang sama dengan nomor di kartu.
                <div class="border-t border-white/[0.06] pt-6">
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="text-lg font-semibold text-slate-100">
                            "Daftar BMN"
                            <span class="text-sm font-normal text-slate-500 ml-2">
                                {move || format!("({} BMN)", daftar.with(Vec::len))}
                            </span>
                        </h3>
                        <button
                            type="button"
                            data-testid="tambah-bmn"
                            class="px-3 py-1.5 bg-green-600 text-white text-sm rounded-lg hover:bg-green-700 flex items-center gap-1"
                            on:click=move |_| {
                                let kunci = kunci_berikutnya.get_value();
                                kunci_berikutnya.set_value(kunci + 1);
                                daftar.update(|d| d.push(BarisBmn::baru(kunci)));
                            }
                        >
                            <AppIcon icon=PLUS />
                            "Tambah BMN"
                        </button>
                    </div>

                    <div class="space-y-3">
                        <For
                            each=move || daftar.get()
                            key=|baris| baris.kunci
                            children=move |baris| {
                                let kunci = baris.kunci;
                                let nomor = Signal::derive(move || {
                                    daftar
                                        .with(|d| d.iter().position(|b| b.kunci == kunci))
                                        .map_or(0, |i| i + 1)
                                });
                                let bisa_dihapus = Signal::derive(move || daftar.with(|d| d.len() > 1));
                                view! {
                                    <KartuBmn
                                        baris=baris
                                        nomor=nomor
                                        bisa_dihapus=bisa_dihapus
                                        on_hapus=hapus
                                    />
                                }
                            }
                        />
                    </div>
                </div>

                <div class="pt-6 flex justify-end gap-3 border-t border-white/[0.06]">
                    <a
                        href=routes::path::PENGELOLAAN_PEMAKAIAN
                        class="px-6 py-2 text-slate-200 bg-white/[0.05] rounded-lg hover:bg-white/[0.09] transition-colors"
                    >
                        "Batal"
                    </a>
                    <button
                        type="submit"
                        class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 flex items-center gap-2"
                        prop:disabled=move || loading.get() || !semua_siap()
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

/// Tahun dari `tgl_perolehan` SIMAN, yang bertipe TEXT dan tidak selalu
/// tanggal. Empat digit pertama diambil hanya bila memang empat digit —
/// selebihnya dikosongkan, karena tahun karangan lebih buruk daripada kolom
/// kosong.
fn tahun_perolehan(item: &BankAsetItem) -> String {
    item.tgl_perolehan
        .as_deref()
        .map(str::trim)
        .filter(|t| t.len() >= 4 && t[..4].chars().all(|c| c.is_ascii_digit()))
        .map(|t| t[..4].to_string())
        .unwrap_or_default()
}
