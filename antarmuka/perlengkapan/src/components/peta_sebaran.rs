//! Peta sebaran BMN bertingkat: seluruh Indonesia, satu wilayah (Kejati), satu satker.
//!
//! Tingkat nasional menggambar satu titik per wilayah dengan ringkasan seluruh BMN. Memilih sebuah
//! wilayah memperbesar peta sampai wilayah itu saja, dengan satu titik per satker dan ringkasan
//! wilayahnya. Memilih satker memperbesar lagi dan menampilkan rincian BMN satker itu. Setiap tingkat
//! menggambar jumlah titik yang wajar (puluhan wilayah, puluhan satker), bukan ratusan ribu aset.
//!
//! Peta digambar dari data yang ditanam di aplikasi (`peta_indonesia_data`), bukan dari server peta:
//! jaringan pemerintah dan CSP aplikasi tidak mengizinkan permintaan keluar. Koordinat satker datang
//! dari MySIMKARI lewat `/bank-aset/sebaran`; satker yang belum punya koordinat tetap dihitung dan
//! tampil di tabel, hanya tidak bisa dititik. Wilayah memakai definisi yang sama dengan penyaring
//! wilayah dan tier RBAC (digit 6-9 kode SIMAN), bernama Kejati-nya.
//!
//! Ukuran titik menunjukkan jumlah aset, warna menunjukkan porsi aset rusak.

use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use lib_ui::components::charts::format_rupiah_short;

use super::peta_indonesia_data::{INDONESIA, LAT_MAKS, LEBAR, LON_MIN, SKALA, TETANGGA, TINGGI};
use crate::api::bank_aset::{self, AsetFilterQuery, BankAsetDashboard, SebaranSatker};
use crate::routes::path;

/// `viewBox`: x, y, lebar, tinggi dalam satuan SVG.
pub type Kotak = (f64, f64, f64, f64);

const SELURUH: Kotak = (0.0, 0.0, LEBAR, TINGGI);

/// Nama kelompok yang tak berada di bawah Kejati mana pun (Kejaksaan Agung dan unit pusat).
pub const NAMA_PUSAT: &str = "Kejaksaan Agung dan unit pusat";

/// Porsi aset rusak yang mewarnai titik. Batasnya sengaja kasar: peta dipakai untuk melihat di mana
/// masalah menumpuk, angka persisnya ada di kartu dan tabel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kelas {
    Baik,
    Perhatian,
    Rusak,
}

impl Kelas {
    pub fn warna(self) -> &'static str {
        match self {
            Kelas::Baik => "#34d399",
            Kelas::Perhatian => "#fbbf24",
            Kelas::Rusak => "#f87171",
        }
    }
}

/// Kurang dari 5% rusak: baik. Kurang dari 20%: perlu perhatian. Selebihnya: rusak.
pub fn kelas_kondisi(total: i64, rusak: i64) -> Kelas {
    if total <= 0 {
        return Kelas::Baik;
    }
    let porsi = rusak as f64 / total as f64;
    if porsi < 0.05 {
        Kelas::Baik
    } else if porsi < 0.20 {
        Kelas::Perhatian
    } else {
        Kelas::Rusak
    }
}

/// Jumlah BMN sekelompok satker.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Ringkasan {
    pub satker: usize,
    pub aset: i64,
    pub nilai: f64,
    pub baik: i64,
    pub rusak: i64,
    pub bergps: i64,
}

impl Ringkasan {
    pub fn dari<'a>(satker: impl IntoIterator<Item = &'a SebaranSatker>) -> Self {
        let mut r = Ringkasan::default();
        for s in satker {
            r.satker += 1;
            r.aset += s.total_aset;
            r.nilai += s.nilai_perolehan;
            r.baik += s.aset_baik;
            r.rusak += s.aset_rusak;
            r.bergps += s.aset_bergps;
        }
        r
    }

    pub fn kelas(&self) -> Kelas {
        kelas_kondisi(self.aset, self.rusak)
    }
}

/// Lintang/bujur menjadi koordinat SVG (proyeksi equirectangular, sama dengan pembuat peta).
pub fn ke_svg(lintang: f64, bujur: f64) -> (f64, f64) {
    ((bujur - LON_MIN) * SKALA, (LAT_MAKS - lintang) * SKALA)
}

/// Satu titik satker di peta. `idx` menunjuk satker asal.
#[derive(Debug, Clone, PartialEq)]
pub struct Titik {
    pub idx: usize,
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub kelas: Kelas,
}

/// Titik untuk setiap satker yang punya koordinat, dan indeks satker yang tidak punya.
///
/// Satker diterima berurut dari yang terbanyak asetnya; yang terbanyak tetap di titik kota dan
/// yang lain di kota yang sama disebar spiral di sekitarnya (sudut emas), jaraknya tumbuh pelan
/// sehingga belasan titik tetap terpisah.
pub fn susun_titik(satker: &[SebaranSatker]) -> (Vec<Titik>, Vec<usize>) {
    let maks = satker
        .iter()
        .map(|s| s.total_aset)
        .max()
        .unwrap_or(1)
        .max(1) as f64;
    let mut per_kota: std::collections::HashMap<(i64, i64), usize> =
        std::collections::HashMap::new();
    let mut titik = Vec::new();
    let mut tanpa = Vec::new();
    for (idx, s) in satker.iter().enumerate() {
        let (Some(lintang), Some(bujur)) = (s.lintang, s.bujur) else {
            tanpa.push(idx);
            continue;
        };
        let (mut x, mut y) = ke_svg(lintang, bujur);
        let kunci = (
            (lintang * 100.0).round() as i64,
            (bujur * 100.0).round() as i64,
        );
        let urutan = per_kota.entry(kunci).or_insert(0);
        if *urutan > 0 {
            let k = *urutan as f64;
            let sudut = k * 2.399_963;
            let jarak = 2.4 * k.sqrt();
            x += jarak * sudut.cos();
            y += jarak * sudut.sin();
        }
        *urutan += 1;
        let r = 1.6 + 4.4 * (s.total_aset.max(0) as f64 / maks).sqrt();
        titik.push(Titik {
            idx,
            x,
            y,
            r,
            kelas: kelas_kondisi(s.total_aset, s.aset_rusak),
        });
    }
    (titik, tanpa)
}

/// Satu wilayah (Kejati) beserta anggotanya.
#[derive(Debug, Clone, PartialEq)]
pub struct Wilayah {
    pub kode: String,
    pub nama: String,
    pub ringkasan: Ringkasan,
    /// Tempat titik wilayah di peta; `None` bila tak satu pun anggotanya punya koordinat.
    pub titik: Option<(f64, f64)>,
    /// Kotak yang memuat titik anggotanya yang punya koordinat.
    pub batas: Option<Kotak>,
    /// Indeks satker anggota.
    pub anggota: Vec<usize>,
}

/// Nama yang dibaca orang: nama Kejati, atau nama kelompok pusat bila tak ada Kejati di atasnya.
pub fn nama_wilayah(nama: Option<&str>) -> String {
    nama.map(str::trim)
        .filter(|n| !n.is_empty())
        .map_or_else(|| NAMA_PUSAT.to_string(), str::to_string)
}

/// Kelompokkan satker per wilayah, terbesar asetnya lebih dulu.
///
/// Titik wilayah adalah tempat Kejati-nya sendiri bila satker Kejati itu ada di daftar dan
/// berkoordinat; bila tidak, titik berat anggotanya berbobot jumlah aset.
pub fn susun_wilayah(satker: &[SebaranSatker]) -> Vec<Wilayah> {
    let mut urut: Vec<String> = Vec::new();
    let mut per: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    for (i, s) in satker.iter().enumerate() {
        let kode = s.wilayah_kode.clone().unwrap_or_default();
        if !per.contains_key(&kode) {
            urut.push(kode.clone());
        }
        per.entry(kode).or_default().push(i);
    }
    let mut hasil: Vec<Wilayah> = urut
        .into_iter()
        .map(|kode| {
            let anggota = per.remove(&kode).unwrap_or_default();
            let nama = nama_wilayah(
                anggota
                    .iter()
                    .find_map(|&i| satker[i].wilayah_nama.as_deref()),
            );
            let ringkasan = Ringkasan::dari(anggota.iter().map(|&i| &satker[i]));
            let berkoordinat: Vec<&SebaranSatker> = anggota
                .iter()
                .map(|&i| &satker[i])
                .filter(|s| s.lintang.is_some() && s.bujur.is_some())
                .collect();
            let milik_kejati = berkoordinat
                .iter()
                .find(|s| s.nama_satker.trim().eq_ignore_ascii_case(nama.trim()));
            let titik = milik_kejati
                .map(|s| ke_svg(s.lintang.unwrap_or_default(), s.bujur.unwrap_or_default()))
                .or_else(|| {
                    let bobot: f64 = berkoordinat
                        .iter()
                        .map(|s| s.total_aset.max(1) as f64)
                        .sum();
                    if berkoordinat.is_empty() || bobot <= 0.0 {
                        return None;
                    }
                    let lat = berkoordinat
                        .iter()
                        .map(|s| s.lintang.unwrap_or_default() * s.total_aset.max(1) as f64)
                        .sum::<f64>()
                        / bobot;
                    let lon = berkoordinat
                        .iter()
                        .map(|s| s.bujur.unwrap_or_default() * s.total_aset.max(1) as f64)
                        .sum::<f64>()
                        / bobot;
                    Some(ke_svg(lat, lon))
                });
            let batas = berkoordinat.iter().fold(None, |acc: Option<Kotak>, s| {
                let (x, y) = ke_svg(s.lintang.unwrap_or_default(), s.bujur.unwrap_or_default());
                Some(match acc {
                    None => (x, y, x, y),
                    Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                })
            });
            Wilayah {
                kode,
                nama,
                ringkasan,
                titik,
                // Folded as (min x, min y, max x, max y); the box is (x, y, width, height).
                batas: batas.map(|(x0, y0, x1, y1)| (x0, y0, x1 - x0, y1 - y0)),
                anggota,
            }
        })
        .collect();
    hasil.sort_by(|a, b| {
        b.ringkasan
            .aset
            .cmp(&a.ringkasan.aset)
            .then_with(|| a.nama.cmp(&b.nama))
    });
    hasil
}

/// Tingkat yang sedang dilihat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tingkat {
    Nasional,
    /// Indeks di daftar wilayah.
    Wilayah(usize),
    /// Indeks di daftar satker.
    Satker(usize),
}

/// Wilayah yang memuat satker ke-`i`.
pub fn wilayah_dari_satker(wilayah: &[Wilayah], i: usize) -> Option<usize> {
    wilayah.iter().position(|w| w.anggota.contains(&i))
}

/// Wilayah yang sedang dibuka: wilayah itu sendiri, atau wilayah dari satker yang dipilih.
pub fn wilayah_terbuka(tingkat: Tingkat, wilayah: &[Wilayah]) -> Option<usize> {
    match tingkat {
        Tingkat::Nasional => None,
        Tingkat::Wilayah(w) => Some(w),
        Tingkat::Satker(i) => wilayah_dari_satker(wilayah, i),
    }
}

/// Kotak berpusat di (cx, cy) selebar `lebar` dengan bentuk yang sama dengan peta, digeser agar
/// tetap di dalam peta. Bentuk yang sama membuat animasi zoom tak menggepengkan peta.
pub fn kotak_berpusat(cx: f64, cy: f64, lebar: f64) -> Kotak {
    let lebar = lebar.clamp(20.0, LEBAR);
    let tinggi = lebar * TINGGI / LEBAR;
    let x = (cx - lebar / 2.0).clamp(0.0, LEBAR - lebar);
    let y = (cy - tinggi / 2.0).clamp(0.0, TINGGI - tinggi);
    (x, y, lebar, tinggi)
}

/// Kotak yang memuat `batas` dengan ruang di sekelilingnya, tak lebih sempit dari `LEBAR_MIN`
/// supaya wilayah yang satkernya berdekatan tak diperbesar sampai peta tak terbaca.
pub fn kotak_memuat(batas: Kotak) -> Kotak {
    const LEBAR_MIN: f64 = 70.0;
    let (x, y, w, h) = batas;
    let ruang = 1.25;
    let lebar = (w * ruang).max(h * ruang * LEBAR / TINGGI).max(LEBAR_MIN);
    kotak_berpusat(x + w / 2.0, y + h / 2.0, lebar)
}

/// Kotak pandang untuk satu tingkat.
pub fn kotak_tingkat(tingkat: Tingkat, wilayah: &[Wilayah], titik: &[Titik]) -> Kotak {
    match tingkat {
        Tingkat::Nasional => SELURUH,
        Tingkat::Wilayah(w) => wilayah
            .get(w)
            .and_then(|w| w.batas)
            .map_or(SELURUH, kotak_memuat),
        Tingkat::Satker(i) => match titik.iter().find(|t| t.idx == i) {
            Some(t) => kotak_berpusat(t.x, t.y, 36.0),
            // A satker without a place keeps its wilayah in view.
            None => kotak_tingkat(
                wilayah_dari_satker(wilayah, i).map_or(Tingkat::Nasional, Tingkat::Wilayah),
                wilayah,
                titik,
            ),
        },
    }
}

/// Campuran linear dua kotak; `t` 0 = `dari`, 1 = `ke`.
pub fn campur(dari: Kotak, ke: Kotak, t: f64) -> Kotak {
    // The last frame lands exactly on the target, not a rounding error away from it.
    if t >= 1.0 {
        return ke;
    }
    let l = |a: f64, b: f64| a + (b - a) * t;
    (
        l(dari.0, ke.0),
        l(dari.1, ke.1),
        l(dari.2, ke.2),
        l(dari.3, ke.3),
    )
}

/// Tautan ke Daftar Aset yang sudah disaring ke satu satker.
pub fn tautan_daftar(nama_satker: &str, kode_siman: Option<&str>) -> String {
    let nama = urlencoding::encode(nama_satker);
    match kode_siman.map(str::trim).filter(|k| !k.is_empty()) {
        // By code: a name would merge two satkers that share one. The name only labels the notice.
        Some(kode) => format!(
            "{}?kode_siman={}&nama={nama}",
            path::BANK_ASET_DAFTAR,
            urlencoding::encode(kode)
        ),
        None => format!("{}?satker={nama}", path::BANK_ASET_DAFTAR),
    }
}

fn persen(bagian: i64, total: i64) -> String {
    if total <= 0 {
        return "0%".to_string();
    }
    format!("{:.1}%", bagian as f64 / total as f64 * 100.0)
}

/// Angka dengan titik pemisah ribuan, seperti yang dibaca orang Indonesia.
pub fn ribuan(n: i64) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Yang sedang disorot kursor atau fokus papan ketik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sorot {
    Wilayah(usize),
    Satker(usize),
}

impl Sorot {
    fn tingkat(self) -> Tingkat {
        match self {
            Sorot::Wilayah(w) => Tingkat::Wilayah(w),
            Sorot::Satker(i) => Tingkat::Satker(i),
        }
    }
}

const LANGKAH: u32 = 14;

fn animasi(
    vb: RwSignal<Kotak>,
    dari: Kotak,
    ke: Kotak,
    langkah: u32,
    generasi: RwSignal<u64>,
    saya: u64,
) {
    if generasi.get_untracked() != saya {
        return;
    }
    let t = langkah as f64 / LANGKAH as f64;
    let ease = 1.0 - (1.0 - t).powi(3);
    vb.set(campur(dari, ke, ease));
    if langkah < LANGKAH {
        set_timeout(
            move || animasi(vb, dari, ke, langkah + 1, generasi, saya),
            Duration::from_millis(18),
        );
    }
}

/// Satu titik di peta. Ukurannya mengikuti pembesaran (`faktor`) supaya tetap wajar di layar.
#[allow(clippy::too_many_arguments)]
fn bulat(
    vb: RwSignal<Kotak>,
    sorot: RwSignal<Option<Sorot>>,
    tingkat: RwSignal<Tingkat>,
    (x, y): (f64, f64),
    r0: f64,
    warna: &'static str,
    redup: bool,
    terpilih: bool,
    label: String,
    pada: Sorot,
) -> impl IntoView {
    // Dots shrink with the zoom so they do not swallow the map, but less than the map grows:
    // zoomed in there are fewer of them and they are what the eye is looking for.
    let ukuran = move || (vb.get().2 / LEBAR).powf(0.65);
    view! {
        <circle
            data-testid="titik-peta"
            cx=format!("{x:.2}")
            cy=format!("{y:.2}")
            r=move || format!("{:.3}", r0 * ukuran())
            fill=warna
            fill-opacity=if redup { "0.3" } else { "0.85" }
            stroke=if terpilih { "#ffffff" } else { "#06101e" }
            stroke-width=move || format!("{:.3}", if terpilih { 1.4 } else { 0.5 } * ukuran())
            class="cursor-pointer outline-none focus:stroke-white"
            tabindex="0"
            role="button"
            aria-label=label
            on:mouseenter=move |_| sorot.set(Some(pada))
            on:mouseleave=move |_| sorot.set(None)
            on:focus=move |_| sorot.set(Some(pada))
            on:blur=move |_| sorot.set(None)
            on:click=move |_| tingkat.set(pada.tingkat())
            on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                if ev.key() == "Enter" || ev.key() == " " {
                    ev.prevent_default();
                    tingkat.set(pada.tingkat());
                }
            }
        />
    }
}

#[component]
pub fn PetaSebaran(
    satker: Arc<Vec<SebaranSatker>>,
    wilayah: Arc<Vec<Wilayah>>,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView {
    let (titik, tanpa) = susun_titik(&satker);
    let titik = Arc::new(titik);
    let nasional = Ringkasan::dari(satker.iter());
    let jumlah_tanpa = tanpa.len();
    let aset_tanpa: i64 = tanpa.iter().map(|&i| satker[i].total_aset).sum();
    let jumlah_titik = titik.len();
    let maks_wilayah = wilayah
        .iter()
        .map(|w| w.ringkasan.aset)
        .max()
        .unwrap_or(1)
        .max(1) as f64;

    let sorot = RwSignal::<Option<Sorot>>::new(None);
    let vb = RwSignal::new(SELURUH);
    let generasi = RwSignal::new(0_u64);

    // Zoom mengikuti tingkat: peta bergeser ke kotak tujuan dalam beberapa langkah. Setiap tujuan
    // baru membatalkan animasi yang masih berjalan.
    {
        let (wilayah, titik) = (Arc::clone(&wilayah), Arc::clone(&titik));
        Effect::new(move |awal: Option<()>| {
            let ke = kotak_tingkat(tingkat.get(), &wilayah, &titik);
            sorot.set(None);
            if awal.is_none() {
                vb.set(ke);
                return;
            }
            let saya = generasi.get_untracked() + 1;
            generasi.set(saya);
            animasi(vb, vb.get_untracked(), ke, 1, generasi, saya);
        });
    }

    let kotak = move || {
        let (x, y, w, h) = vb.get();
        format!("{x:.2} {y:.2} {w:.2} {h:.2}")
    };

    let lapisan = {
        let (wilayah, satker, titik) = (
            Arc::clone(&wilayah),
            Arc::clone(&satker),
            Arc::clone(&titik),
        );
        move || match tingkat.get() {
            Tingkat::Nasional => wilayah
                .iter()
                .enumerate()
                .filter_map(|(w, wil)| {
                    let tempat = wil.titik?;
                    let r = 2.6 + 6.4 * (wil.ringkasan.aset.max(0) as f64 / maks_wilayah).sqrt();
                    let label = format!(
                        "{}: {} satker, {} aset",
                        wil.nama, wil.ringkasan.satker, wil.ringkasan.aset
                    );
                    Some(
                        bulat(
                            vb,
                            sorot,
                            tingkat,
                            tempat,
                            r,
                            wil.ringkasan.kelas().warna(),
                            false,
                            false,
                            label,
                            Sorot::Wilayah(w),
                        )
                        .into_any(),
                    )
                })
                .collect::<Vec<_>>(),
            sekarang => {
                let terbuka = wilayah_terbuka(sekarang, &wilayah);
                let dipilih = match sekarang {
                    Tingkat::Satker(i) => Some(i),
                    _ => None,
                };
                titik
                    .iter()
                    .filter(|t| terbuka.is_some_and(|w| wilayah[w].anggota.contains(&t.idx)))
                    .map(|t| {
                        let s = &satker[t.idx];
                        let label = format!("{}: {} aset", s.nama_satker, s.total_aset);
                        let aktif = dipilih == Some(t.idx);
                        let redup = dipilih.is_some() && !aktif;
                        bulat(
                            vb,
                            sorot,
                            tingkat,
                            (t.x, t.y),
                            t.r,
                            t.kelas.warna(),
                            redup,
                            aktif,
                            label,
                            Sorot::Satker(t.idx),
                        )
                        .into_any()
                    })
                    .collect::<Vec<_>>()
            }
        }
    };

    // Jalur remah roti: Indonesia > Kejati > satker. Setiap ruas yang sudah dilewati bisa diklik.
    let remah = {
        let (wilayah, satker) = (Arc::clone(&wilayah), Arc::clone(&satker));
        move || {
            let t = tingkat.get();
            let terbuka = wilayah_terbuka(t, &wilayah);
            let tombol = "rounded-md px-2 py-1 text-xs font-medium text-slate-300 hover:bg-white/[0.08] hover:text-slate-100";
            let kini = "px-2 py-1 text-xs font-semibold text-gold-300";
            view! {
                <nav
                    aria-label="Tingkat peta"
                    class="flex flex-wrap items-center gap-1"
                    data-testid="peta-remah"
                >
                    {if t == Tingkat::Nasional {
                        view! { <span class=kini>"Indonesia"</span> }.into_any()
                    } else {
                        view! {
                            <button
                                type="button"
                                class=tombol
                                on:click=move |_| tingkat.set(Tingkat::Nasional)
                            >
                                "Indonesia"
                            </button>
                        }
                            .into_any()
                    }}
                    {terbuka
                        .map(|w| {
                            let nama = wilayah[w].nama.clone();
                            view! {
                                <span class="text-slate-600">"›"</span>
                                {if matches!(t, Tingkat::Wilayah(_)) {
                                    view! { <span class=kini>{nama}</span> }.into_any()
                                } else {
                                    view! {
                                        <button
                                            type="button"
                                            class=tombol
                                            on:click=move |_| tingkat.set(Tingkat::Wilayah(w))
                                        >
                                            {nama}
                                        </button>
                                    }
                                        .into_any()
                                }}
                            }
                        })}
                    {match t {
                        Tingkat::Satker(i) => {
                            let nama = satker[i].nama_satker.clone();
                            Some(
                                view! {
                                    <span class="text-slate-600">"›"</span>
                                    <span class=kini>{nama}</span>
                                },
                            )
                        }
                        _ => None,
                    }}
                </nav>
            }
        }
    };

    let panel = {
        let (wilayah, satker) = (Arc::clone(&wilayah), Arc::clone(&satker));
        move || {
            let kini = tingkat.get();
            let fokus = sorot.get().or(match kini {
                Tingkat::Satker(i) => Some(Sorot::Satker(i)),
                _ => None,
            });
            match (fokus, kini) {
                (Some(Sorot::Wilayah(w)), _) => kartu_wilayah(&wilayah[w]).into_any(),
                (Some(Sorot::Satker(i)), _) => kartu_satker(&satker[i], i, tingkat).into_any(),
                (None, Tingkat::Wilayah(w)) => {
                    panel_wilayah(&wilayah[w], &satker, tingkat).into_any()
                }
                (None, _) => panel_nasional(&nasional, &wilayah, tingkat).into_any(),
            }
        }
    };

    view! {
        <div id="peta-bmn" data-testid="peta-sebaran" class="space-y-4">
            {remah}
            <div class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_20rem]">
                <div
                    class="relative w-full overflow-hidden rounded-xl border border-white/[0.08]"
                    style=format!("background:#0a1b30;aspect-ratio:{LEBAR}/{TINGGI}")
                >
                    <svg
                        viewBox=kotak
                        preserveAspectRatio="xMidYMid meet"
                        class="absolute inset-0 h-full w-full"
                        role="img"
                        aria-label="Peta sebaran BMN per wilayah dan satuan kerja"
                    >
                        <path d=TETANGGA fill="#12294a" stroke="#1d3d66" stroke-width="0.6" vector-effect="non-scaling-stroke" />
                        <path d=INDONESIA fill="#1d4268" stroke="#3a6a9a" stroke-width="0.8" vector-effect="non-scaling-stroke" />
                        {lapisan}
                    </svg>
                </div>
                <div class="space-y-4">
                    <div class="rounded-xl border border-white/[0.08] bg-white/[0.03] p-4">
                        {panel}
                    </div>
                    <div class="rounded-xl border border-white/[0.08] bg-white/[0.03] p-4 text-xs text-slate-300">
                        <p class="mb-2 font-medium text-slate-200">"Warna = porsi aset rusak"</p>
                        <ul class="space-y-1.5">
                            <li class="flex items-center gap-2">
                                <span
                                    class="inline-block h-2.5 w-2.5 rounded-full"
                                    style="background:#34d399"
                                ></span>
                                "Di bawah 5%"
                            </li>
                            <li class="flex items-center gap-2">
                                <span
                                    class="inline-block h-2.5 w-2.5 rounded-full"
                                    style="background:#fbbf24"
                                ></span>
                                "5% sampai 20%"
                            </li>
                            <li class="flex items-center gap-2">
                                <span
                                    class="inline-block h-2.5 w-2.5 rounded-full"
                                    style="background:#f87171"
                                ></span>
                                "20% atau lebih"
                            </li>
                        </ul>
                        <p class="mt-3 text-slate-400">"Ukuran titik = jumlah aset."</p>
                    </div>
                </div>
            </div>
            <p data-testid="peta-cakupan" class="text-xs text-slate-400">
                {format!("{jumlah_titik} satker ada di peta.")}
                {(jumlah_tanpa > 0)
                    .then(|| {
                        format!(
                            " {jumlah_tanpa} satker ({} aset) belum punya koordinat di MySIMKARI, \
                             tetap terhitung dan ada di tabel di bawah.",
                            ribuan(aset_tanpa),
                        )
                    })}
            </p>
        </div>
    }
}

fn baris_ringkasan(r: &Ringkasan) -> impl IntoView + use<> {
    let kelas = r.kelas();
    view! {
        <dl class="grid grid-cols-2 gap-x-3 gap-y-2 text-xs">
            <div>
                <dt class="text-slate-500">"Satker"</dt>
                <dd class="font-semibold text-slate-100">{ribuan(r.satker as i64)}</dd>
            </div>
            <div>
                <dt class="text-slate-500">"Total aset"</dt>
                <dd class="font-semibold text-slate-100">{ribuan(r.aset)}</dd>
            </div>
            <div>
                <dt class="text-slate-500">"Nilai perolehan"</dt>
                <dd class="font-semibold text-slate-100">{format_rupiah_short(r.nilai)}</dd>
            </div>
            <div>
                <dt class="text-slate-500">"Kondisi baik"</dt>
                <dd class="font-semibold text-success-300">{persen(r.baik, r.aset)}</dd>
            </div>
            <div class="col-span-2">
                <dt class="text-slate-500">"Rusak"</dt>
                <dd class="font-semibold" style=format!("color:{}", kelas.warna())>
                    {persen(r.rusak, r.aset)}
                </dd>
            </div>
        </dl>
    }
}

const TOMBOL_BARIS: &str = "flex w-full items-center justify-between gap-2 rounded-lg px-2 py-1.5 text-left text-xs text-slate-200 transition hover:bg-white/[0.06]";
const TOMBOL_AKSI: &str = "focus-ring rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-xs font-medium text-slate-100 transition hover:bg-white/[0.08]";

fn panel_nasional(
    nasional: &Ringkasan,
    wilayah: &Arc<Vec<Wilayah>>,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView + use<> {
    let teratas: Vec<(usize, String, i64)> = wilayah
        .iter()
        .enumerate()
        .filter(|(_, w)| w.titik.is_some())
        .take(6)
        .map(|(i, w)| (i, w.nama.clone(), w.ringkasan.aset))
        .collect();
    let jumlah_wilayah = wilayah.len();
    view! {
        <div data-testid="panel-nasional" class="space-y-3">
            <div>
                <p class="text-sm font-semibold text-slate-100">"Seluruh Indonesia"</p>
                <p class="text-[0.7rem] text-slate-500">
                    {format!("{jumlah_wilayah} wilayah. Pilih titik untuk memperbesar.")}
                </p>
            </div>
            {baris_ringkasan(nasional)}
            <div>
                <p class="mb-1 text-[0.65rem] uppercase tracking-wide text-slate-500">
                    "Wilayah terbesar"
                </p>
                <ul>
                    {teratas
                        .into_iter()
                        .map(|(i, nama, aset)| {
                            view! {
                                <li>
                                    <button
                                        type="button"
                                        class=TOMBOL_BARIS
                                        on:click=move |_| tingkat.set(Tingkat::Wilayah(i))
                                    >
                                        <span class="min-w-0 break-words">{nama}</span>
                                        <span class="shrink-0 text-slate-400">{ribuan(aset)}</span>
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            </div>
        </div>
    }
}

/// The preview of a wilayah while a dot is hovered or focused. It carries no buttons on purpose: it
/// goes away the moment the pointer leaves the dot, so a control in it could never be reached.
/// Clicking the dot (or Enter) opens the wilayah.
fn kartu_wilayah(w: &Wilayah) -> impl IntoView + use<> {
    let nama = w.nama.clone();
    let ringkasan = w.ringkasan.clone();
    view! {
        <div data-testid="kartu-wilayah" class="space-y-3">
            <div>
                <p class="break-words text-sm font-semibold text-slate-100">{nama}</p>
                <p class="text-[0.7rem] text-slate-500">"Klik titik untuk memperbesar wilayah ini."</p>
            </div>
            {baris_ringkasan(&ringkasan)}
        </div>
    }
}

fn panel_wilayah(
    w: &Wilayah,
    satker: &Arc<Vec<SebaranSatker>>,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView + use<> {
    let nama = w.nama.clone();
    let ringkasan = w.ringkasan.clone();
    let daftar: Vec<(usize, String, i64, bool)> = w
        .anggota
        .iter()
        .take(8)
        .map(|&i| {
            let s = &satker[i];
            (
                i,
                s.nama_satker.clone(),
                s.total_aset,
                s.lintang.is_some() && s.bujur.is_some(),
            )
        })
        .collect();
    let lebih = w.anggota.len().saturating_sub(daftar.len());
    view! {
        <div data-testid="panel-wilayah" class="space-y-3">
            <div>
                <p class="break-words text-sm font-semibold text-slate-100">{nama}</p>
                <p class="text-[0.7rem] text-slate-500">
                    "Pilih titik satker untuk melihat rinciannya."
                </p>
            </div>
            {baris_ringkasan(&ringkasan)}
            <div>
                <p class="mb-1 text-[0.65rem] uppercase tracking-wide text-slate-500">
                    "Satker terbesar"
                </p>
                <ul>
                    {daftar
                        .into_iter()
                        .map(|(i, nama, aset, di_peta)| {
                            view! {
                                <li>
                                    <button
                                        type="button"
                                        class=TOMBOL_BARIS
                                        on:click=move |_| tingkat.set(Tingkat::Satker(i))
                                    >
                                        <span class="min-w-0 break-words">
                                            {nama}
                                            {(!di_peta)
                                                .then(|| {
                                                    view! {
                                                        <span class="ml-1 text-slate-600">"· belum di peta"</span>
                                                    }
                                                })}
                                        </span>
                                        <span class="shrink-0 text-slate-400">{ribuan(aset)}</span>
                                    </button>
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
                {(lebih > 0)
                    .then(|| {
                        view! {
                            <p class="mt-1 px-2 text-[0.7rem] text-slate-500">
                                {format!("dan {lebih} satker lain di tabel di bawah")}
                            </p>
                        }
                    })}
            </div>
        </div>
    }
}

fn kartu_satker(
    s: &SebaranSatker,
    indeks: usize,
    tingkat: RwSignal<Tingkat>,
) -> impl IntoView + use<> {
    let kelas = kelas_kondisi(s.total_aset, s.aset_rusak);
    let href = tautan_daftar(&s.nama_satker, s.kode_satker.as_deref());
    let terpilih = tingkat.get_untracked() == Tingkat::Satker(indeks);
    let rincian = terpilih.then(|| {
        view! { <RincianSatker kode=s.kode_satker.clone() /> }
    });
    let gps = if s.aset_bergps > 0 {
        format!(
            "{} dari {} aset punya koordinat GPS di SIMAN.",
            ribuan(s.aset_bergps),
            ribuan(s.total_aset)
        )
    } else {
        "SIMAN belum menyertakan koordinat GPS untuk aset satker ini, jadi titik aset belum bisa digambar."
            .to_string()
    };
    let tanpa_tempat = (s.lintang.is_none() || s.bujur.is_none()).then_some(
        "Satker ini belum punya koordinat di MySIMKARI, jadi tidak ada titiknya di peta.",
    );
    view! {
        <div data-testid="kartu-satker" class="space-y-3">
            <div class="min-w-0">
                <p class="break-words text-sm font-semibold text-slate-100">
                    {s.nama_satker.clone()}
                </p>
                <p class="mt-0.5 break-all text-[0.7rem] text-slate-500">
                    {s.kode_satker.clone().unwrap_or_else(|| "-".to_string())}
                    {s.provinsi.clone().map(|p| format!(" · {p}")).unwrap_or_default()}
                </p>
            </div>
            <dl class="grid grid-cols-2 gap-x-3 gap-y-2 text-xs">
                <div>
                    <dt class="text-slate-500">"Total aset"</dt>
                    <dd class="font-semibold text-slate-100">{ribuan(s.total_aset)}</dd>
                </div>
                <div>
                    <dt class="text-slate-500">"Nilai perolehan"</dt>
                    <dd class="font-semibold text-slate-100">
                        {format_rupiah_short(s.nilai_perolehan)}
                    </dd>
                </div>
                <div>
                    <dt class="text-slate-500">"Kondisi baik"</dt>
                    <dd class="font-semibold text-success-300">
                        {persen(s.aset_baik, s.total_aset)}
                    </dd>
                </div>
                <div>
                    <dt class="text-slate-500">"Rusak"</dt>
                    <dd class="font-semibold" style=format!("color:{}", kelas.warna())>
                        {persen(s.aset_rusak, s.total_aset)}
                    </dd>
                </div>
            </dl>
            {rincian}
            <p data-testid="gps-satker" class="text-[0.7rem] text-slate-500">{gps}</p>
            {tanpa_tempat.map(|t| view! { <p class="text-[0.7rem] text-slate-500">{t}</p> })}
            // Only the opened satker carries a link: a hover preview vanishes with the pointer.
            {terpilih
                .then(|| {
                    view! {
                        <A href=href attr:class=format!("{TOMBOL_AKSI} inline-flex items-center gap-2")>
                            "Lihat aset satker ini"
                        </A>
                    }
                })}
        </div>
    }
}

/// BMN satu satker menurut jenisnya, dari ringkasan dashboard Bank Aset yang disaring ke satker itu.
/// Server membatasi hasil menurut hak akses pemanggil, jadi satker di luar cakupannya menjawab nol.
#[component]
fn RincianSatker(kode: Option<String>) -> impl IntoView {
    // By the SIMAN code, exact: a name would merge two satkers that share one, and the MySIMKARI
    // code would take in every SIMAN code paired with it. No code, no honest filter to ask with.
    let Some(kode) = kode else {
        return view! {
            <div data-testid="rincian-satker" class="space-y-2">
                <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Jenis BMN"</p>
                <p class="text-xs text-slate-500">"Rincian jenis belum tersedia."</p>
            </div>
        }
        .into_any();
    };
    let data = RwSignal::<Option<Result<BankAsetDashboard, String>>>::new(None);
    let filter = AsetFilterQuery {
        kode_siman: Some(kode),
        ..Default::default()
    };
    spawn_local(async move {
        let hasil = bank_aset::fetch_dashboard(&filter)
            .await
            .map_err(|e| e.to_string());
        // The panel may already be gone (another satker was picked); a write to a dropped
        // signal is a no-op rather than a panic.
        let _ = data.try_set(Some(hasil));
    });
    view! {
        <div data-testid="rincian-satker" class="space-y-2">
            <p class="text-[0.65rem] uppercase tracking-wide text-slate-500">"Jenis BMN"</p>
            {move || match data.get() {
                None => {
                    view! { <p class="text-xs text-slate-500">"Memuat rincian..."</p> }.into_any()
                }
                Some(Err(_)) => {
                    view! { <p class="text-xs text-slate-500">"Rincian jenis belum tersedia."</p> }
                        .into_any()
                }
                Some(Ok(d)) => {
                    let total = d.total_aset.max(1) as f64;
                    view! {
                        <ul class="space-y-1.5">
                            {d
                                .kategori_breakdown
                                .into_iter()
                                .take(6)
                                .map(|k| {
                                    let lebar = (k.count as f64 / total * 100.0).clamp(2.0, 100.0);
                                    view! {
                                        <li class="text-xs">
                                            <div class="flex justify-between gap-2 text-slate-300">
                                                <span class="min-w-0 break-words">{k.kategori}</span>
                                                <span class="shrink-0 text-slate-400">
                                                    {ribuan(k.count)}
                                                </span>
                                            </div>
                                            <div class="mt-0.5 h-1 rounded bg-white/[0.06]">
                                                <div
                                                    class="h-1 rounded bg-gold-400/70"
                                                    style=format!("width:{lebar:.0}%")
                                                ></div>
                                            </div>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                        .into_any()
                }
            }}
        </div>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn satker(
        nama: &str,
        total: i64,
        rusak: i64,
        tempat: Option<(f64, f64)>,
        wilayah: Option<(&str, Option<&str>)>,
    ) -> SebaranSatker {
        SebaranSatker {
            kode_satker: Some(format!("K-{nama}")),
            nama_satker: nama.into(),
            total_aset: total,
            nilai_perolehan: total as f64 * 1000.0,
            aset_baik: total - rusak,
            aset_rusak: rusak,
            lintang: tempat.map(|t| t.0),
            bujur: tempat.map(|t| t.1),
            provinsi: None,
            wilayah_kode: wilayah.map(|w| w.0.to_string()),
            wilayah_nama: wilayah.and_then(|w| w.1.map(str::to_string)),
            aset_bergps: 0,
        }
    }

    #[test]
    fn colour_follows_the_share_of_broken_assets() {
        assert_eq!(kelas_kondisi(100, 4), Kelas::Baik);
        assert_eq!(kelas_kondisi(100, 5), Kelas::Perhatian);
        assert_eq!(kelas_kondisi(100, 19), Kelas::Perhatian);
        assert_eq!(kelas_kondisi(100, 20), Kelas::Rusak);
        assert_eq!(kelas_kondisi(0, 0), Kelas::Baik);
    }

    #[test]
    fn jakarta_lands_on_java_and_the_projection_matches_the_map_builder() {
        // 106.8 E, 6.2 S: (106.8 - 94) * 20 = 256, (6.8 + 6.2) * 20 = 260
        let (x, y) = ke_svg(-6.2, 106.8);
        assert!(
            (x - 256.0).abs() < 1e-9 && (y - 260.0).abs() < 1e-9,
            "{x} {y}"
        );
        assert!((0.0..=LEBAR).contains(&x) && (0.0..=TINGGI).contains(&y));
    }

    #[test]
    fn a_satker_without_coordinates_is_reported_not_dropped() {
        let mut sebagian = satker("c", 3, 0, Some((-6.2, 106.8)), None);
        sebagian.bujur = None;
        let (titik, tanpa) = susun_titik(&[
            satker("a", 10, 0, Some((-6.2, 106.8)), None),
            satker("b", 5, 0, None, None),
            sebagian,
        ]);
        assert_eq!(titik.len(), 1);
        assert_eq!(tanpa, vec![1, 2]);
    }

    #[test]
    fn satkers_in_one_city_are_spread_so_each_stays_visible() {
        let lima: Vec<SebaranSatker> = (0..5)
            .map(|i| satker(&format!("s{i}"), 10, 0, Some((-6.2, 106.8)), None))
            .collect();
        let (titik, _) = susun_titik(&lima);
        assert_eq!(titik.len(), 5);
        assert_eq!((titik[0].x, titik[0].y), ke_svg(-6.2, 106.8));
        for a in 0..titik.len() {
            for b in (a + 1)..titik.len() {
                let jarak =
                    ((titik[a].x - titik[b].x).powi(2) + (titik[a].y - titik[b].y).powi(2)).sqrt();
                assert!(jarak > 1.5, "dots {a} and {b} overlap: {jarak}");
            }
        }
    }

    #[test]
    fn satkers_group_into_their_wilayah_largest_first_and_the_sums_add_up() {
        let s = vec![
            satker(
                "kejati a",
                100,
                10,
                Some((3.5, 98.7)),
                Some(("02", Some("KEJATI SUMUT"))),
            ),
            satker(
                "kejari a1",
                50,
                0,
                Some((3.6, 98.8)),
                Some(("02", Some("KEJATI SUMUT"))),
            ),
            satker(
                "kejari b",
                400,
                40,
                Some((-7.2, 112.7)),
                Some(("13", Some("KEJATI JATIM"))),
            ),
            satker("pusat", 10, 0, None, Some(("0199", None))),
        ];
        let w = susun_wilayah(&s);
        assert_eq!(
            w.iter().map(|w| w.nama.as_str()).collect::<Vec<_>>(),
            ["KEJATI JATIM", "KEJATI SUMUT", NAMA_PUSAT]
        );
        let sumut = &w[1];
        assert_eq!(sumut.ringkasan.satker, 2);
        assert_eq!(sumut.ringkasan.aset, 150);
        assert_eq!(sumut.ringkasan.rusak, 10);
        // Every satker belongs to exactly one wilayah and nothing is lost or counted twice.
        let total: i64 = w.iter().map(|w| w.ringkasan.aset).sum();
        assert_eq!(total, Ringkasan::dari(s.iter()).aset);
        assert_eq!(w.iter().map(|w| w.anggota.len()).sum::<usize>(), s.len());
    }

    #[test]
    fn a_wilayah_dot_sits_on_its_kejati_and_falls_back_to_the_weighted_centre() {
        let s = vec![
            satker(
                "KEJATI SUMUT",
                10,
                0,
                Some((3.5, 98.7)),
                Some(("02", Some("KEJATI SUMUT"))),
            ),
            satker(
                "kejari x",
                1000,
                0,
                Some((2.0, 99.0)),
                Some(("02", Some("KEJATI SUMUT"))),
            ),
            satker(
                "kejari y",
                30,
                0,
                Some((-7.0, 112.0)),
                Some(("13", Some("KEJATI JATIM"))),
            ),
            satker(
                "kejari z",
                10,
                0,
                Some((-8.0, 114.0)),
                Some(("13", Some("KEJATI JATIM"))),
            ),
        ];
        let w = susun_wilayah(&s);
        let sumut = w.iter().find(|w| w.nama == "KEJATI SUMUT").unwrap();
        assert_eq!(
            sumut.titik,
            Some(ke_svg(3.5, 98.7)),
            "the Kejati's own place, not the big Kejari's"
        );
        let jatim = w.iter().find(|w| w.nama == "KEJATI JATIM").unwrap();
        let (x, _) = jatim.titik.expect("centroid");
        let (xy, _) = ke_svg(-7.0, 112.0);
        let (xz, _) = ke_svg(-8.0, 114.0);
        assert!(x > xy && x < xz, "between the two: {x}");
        assert!(x < (xy + xz) / 2.0, "nearer the heavier one: {x}");
    }

    #[test]
    fn a_wilayah_whose_satkers_have_no_place_has_no_dot_but_is_still_counted() {
        let w = susun_wilayah(&[satker("pusat", 10, 0, None, Some(("0199", None)))]);
        assert_eq!(w.len(), 1);
        assert!(w[0].titik.is_none() && w[0].batas.is_none());
        assert_eq!(w[0].ringkasan.aset, 10);
    }

    #[test]
    fn zoom_boxes_stay_inside_the_map_and_keep_its_shape() {
        let s = vec![
            satker("a", 1, 0, Some((3.5, 98.7)), Some(("02", Some("N")))),
            satker("b", 1, 0, Some((-3.0, 104.0)), Some(("02", Some("N")))),
            satker("c", 1, 0, Some((-6.2, 106.8)), Some(("09", Some("J")))),
        ];
        let w = susun_wilayah(&s);
        let (t, _) = susun_titik(&s);
        for tingkat in [
            Tingkat::Nasional,
            Tingkat::Wilayah(0),
            Tingkat::Wilayah(1),
            Tingkat::Satker(0),
            Tingkat::Satker(2),
        ] {
            let (x, y, bw, bh) = kotak_tingkat(tingkat, &w, &t);
            assert!(x >= -1e-9 && y >= -1e-9, "{tingkat:?}");
            assert!(
                x + bw <= LEBAR + 1e-9 && y + bh <= TINGGI + 1e-9,
                "{tingkat:?} leaves the map"
            );
            assert!(
                (bw / bh - LEBAR / TINGGI).abs() < 1e-6,
                "{tingkat:?} changes the aspect: {bw}x{bh}"
            );
        }
        // Zooming in really zooms in.
        let n = kotak_tingkat(Tingkat::Nasional, &w, &t).2;
        assert!(kotak_tingkat(Tingkat::Wilayah(0), &w, &t).2 < n);
        assert!(kotak_tingkat(Tingkat::Satker(0), &w, &t).2 < n);
    }

    #[test]
    fn a_satker_without_a_place_keeps_its_wilayah_in_view() {
        let s = vec![
            satker("a", 5, 0, Some((3.5, 98.7)), Some(("02", Some("N")))),
            satker("b", 1, 0, None, Some(("02", Some("N")))),
        ];
        let w = susun_wilayah(&s);
        let (t, _) = susun_titik(&s);
        assert_eq!(
            kotak_tingkat(Tingkat::Satker(1), &w, &t),
            kotak_tingkat(Tingkat::Wilayah(0), &w, &t)
        );
    }

    #[test]
    fn the_wilayah_of_a_picked_satker_is_the_one_that_holds_it() {
        let s = vec![
            satker("a", 1, 0, Some((3.5, 98.7)), Some(("02", Some("N")))),
            satker("b", 1, 0, Some((-6.2, 106.8)), Some(("09", Some("J")))),
        ];
        let w = susun_wilayah(&s);
        let i = wilayah_dari_satker(&w, 1).unwrap();
        assert_eq!(w[i].nama, "J");
        assert_eq!(wilayah_terbuka(Tingkat::Satker(1), &w), Some(i));
        assert_eq!(wilayah_terbuka(Tingkat::Nasional, &w), None);
    }

    #[test]
    fn blending_two_boxes_starts_and_ends_exactly_on_them() {
        let (a, b) = ((0.0, 0.0, 960.0, 372.0), (100.0, 50.0, 70.0, 27.1));
        assert_eq!(campur(a, b, 0.0), a);
        assert_eq!(campur(a, b, 1.0), b);
    }

    #[test]
    fn counts_read_with_thousands_separators() {
        assert_eq!(ribuan(0), "0");
        assert_eq!(ribuan(999), "999");
        assert_eq!(ribuan(1000), "1.000");
        assert_eq!(ribuan(624_533), "624.533");
        assert_eq!(ribuan(-1500), "-1.500");
    }

    #[test]
    fn the_link_to_the_list_names_the_satker_by_code_and_only_falls_back_to_the_name() {
        // Two satkers can share a name: the code is what tells them apart.
        let url = tautan_daftar("KEJARI KEMBAR", Some("006015555010001KD"));
        assert!(
            url.ends_with("?kode_siman=006015555010001KD&nama=KEJARI%20KEMBAR"),
            "{url}"
        );
        // A satker with no code can only be asked for by its (encoded) name.
        let url = tautan_daftar("KEJAKSAAN NEGERI A & B", None);
        assert!(
            url.ends_with("?satker=KEJAKSAAN%20NEGERI%20A%20%26%20B"),
            "{url}"
        );
        assert!(
            tautan_daftar("X", Some("  ")).contains("?satker="),
            "a blank code is no code"
        );
    }
}
