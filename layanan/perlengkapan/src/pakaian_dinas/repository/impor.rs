//! Impor profil ukuran awal (lihat `impor_profil.rs`): validasi per baris, lalu penulisan massal.

use super::PakaianDinasRepository;
use super::pegawai::{aturan_grup, kunci_grup_bersama};
use crate::pakaian_dinas::impor_profil::{
    ImporBaris, ImporDitolak, ImporProfilHasil, identitas_bersih, nip_tersamar,
};
use crate::pakaian_dinas::validasi_ukuran::validasi_ukuran;
use crate::shared::error::{AppResult, bad_request};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Sumber yang tertulis pada ukuran hasil impor: impor yang diulang boleh menimpanya, ukuran yang
/// diisi orangnya sendiri (`APLIKASI`) tidak.
const SUMBER_IMPOR: &str = "IMPOR";

struct BarisSah {
    nip: String,
    nama: String,
    jenis_kelamin: Option<String>,
    with_hijab: bool,
    ukuran: Vec<(Uuid, String)>,
}

impl PakaianDinasRepository {
    /// Baris yang gagal validasi ditolak SENDIRI-SENDIRI (dengan alasannya); yang lolos ditulis
    /// dalam satu transaksi, atau tak satu pun bila basis data menolak. `dry_run` menjalankan
    /// semuanya kecuali penulisan.
    ///
    /// Aturan terhadap yang sudah ada: profil yang sudah ada tidak diubah (nama dan hijab milik
    /// orangnya atau sinkronisasi), dan ukuran yang diisi lewat aplikasi tidak ditimpa. Hanya
    /// ukuran hasil impor sebelumnya yang diganti, jadi mengulang impor aman.
    pub async fn impor_profil(
        &self,
        baris: &[ImporBaris],
        dry_run: bool,
    ) -> AppResult<ImporProfilHasil> {
        let mut hasil = ImporProfilHasil {
            dry_run,
            diterima: baris.len(),
            ..Default::default()
        };
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Spesifikasi aktif menurut NAMA jenis dan NAMA spesifikasinya (huruf besar, dipangkas).
        let spek_rows = tx
            .query(
                "SELECT s.id, upper(btrim(j.nama)) AS jenis, upper(btrim(s.nama)) AS spek, s.ukuran_group
                 FROM perlengkapan.ms_spesifikasi_pakaian_dinas s
                 JOIN perlengkapan.ms_jenis_pakaian_dinas j ON j.id = s.jenis_pakaian_dinas_id
                 WHERE s.is_active AND j.is_active
                 ORDER BY s.created_at, s.id",
                &[],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        // `None` = pasangan nama yang menunjuk LEBIH DARI SATU spesifikasi aktif: pembuatan
        // spesifikasi tidak melarang nama ganda, dan ukuran profil disimpan per ID. Memilih yang
        // tertua akan menulis ke kolom yang bisa jadi bukan yang dipakai kampanye, tanpa
        // memberi tahu pengirim; baris yang menyebutnya ditolak sebagai ambigu.
        let mut spesifikasi: HashMap<(String, String), Option<(Uuid, String)>> = HashMap::new();
        for r in &spek_rows {
            spesifikasi
                .entry((r.get("jenis"), r.get("spek")))
                .and_modify(|v| *v = None)
                .or_insert_with(|| Some((r.get("id"), r.get("ukuran_group"))));
        }
        // Daftar ukuran dibaca di bawah kunci bersama grupnya: ukuran tak bisa dihapus admin di
        // antara validasi dan penulisan (protokol yang sama dengan penyimpanan profil).
        let mut kode_grup: Vec<String> = spesifikasi
            .values()
            .flatten()
            .map(|(_, g)| g.clone())
            .collect();
        kode_grup.sort();
        kode_grup.dedup();
        kunci_grup_bersama(&tx, &kode_grup).await?;
        let aturan = aturan_grup(&tx, &kode_grup).await?;

        // ── validasi per baris ───────────────────────────────────────────────────────────
        let mut sah: Vec<BarisSah> = Vec::new();
        let mut terlihat: HashSet<String> = HashSet::new();
        for b in baris {
            let tolak = |alasan: String| ImporDitolak {
                rujukan: b.rujukan.chars().take(60).collect(),
                nip_tersamar: nip_tersamar(&b.nip),
                alasan,
            };
            let (nip, nama, jenis_kelamin) = match identitas_bersih(b) {
                Ok(v) => v,
                Err(e) => {
                    hasil.ditolak.push(tolak(e));
                    continue;
                }
            };
            if !terlihat.insert(nip.clone()) {
                hasil
                    .ditolak
                    .push(tolak("NIP ganda dalam permintaan ini".to_string()));
                continue;
            }
            let mut ukuran: Vec<(Uuid, String)> = Vec::new();
            let mut dipakai: HashSet<Uuid> = HashSet::new();
            let mut galat: Option<String> = None;
            for u in &b.ukuran {
                let nama_kolom = format!("{} {}", u.jenis.trim(), u.spesifikasi.trim());
                let kunci = (
                    u.jenis.trim().to_uppercase(),
                    u.spesifikasi.trim().to_uppercase(),
                );
                let (id, grup_kode) = match spesifikasi.get(&kunci) {
                    Some(Some(s)) => s,
                    Some(None) => {
                        galat = Some(format!(
                            "Spesifikasi ambigu: ada lebih dari satu {nama_kolom} aktif"
                        ));
                        break;
                    }
                    None => {
                        galat = Some(format!("Spesifikasi tidak dikenal: {nama_kolom}"));
                        break;
                    }
                };
                if !dipakai.insert(*id) {
                    galat = Some(format!("Kolom {nama_kolom} muncul dua kali"));
                    break;
                }
                let Some((grup, daftar)) = aturan.get(grup_kode) else {
                    galat = Some(format!("Grup ukuran \"{grup_kode}\" tidak ada"));
                    break;
                };
                match validasi_ukuran(grup, daftar, &u.ukuran) {
                    Ok(nilai) => ukuran.push((*id, nilai)),
                    Err(e) => {
                        galat = Some(format!("{nama_kolom}: {e}"));
                        break;
                    }
                }
            }
            if let Some(g) = galat {
                hasil.ditolak.push(tolak(g));
                continue;
            }
            if ukuran.is_empty() {
                hasil.ditolak.push(tolak("Tidak ada ukuran".to_string()));
                continue;
            }
            sah.push(BarisSah {
                nip,
                nama,
                jenis_kelamin,
                with_hijab: b.with_hijab.unwrap_or(false),
                ukuran,
            });
        }
        hasil.baris_dimuat = sah.len();
        if sah.is_empty() {
            return Ok(hasil);
        }

        // ── yang sudah ada ───────────────────────────────────────────────────────────────
        let nips: Vec<String> = sah.iter().map(|b| b.nip.clone()).collect();
        let ada: HashSet<String> = tx
            .query(
                "SELECT nip FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = ANY($1)",
                &[&nips],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .iter()
            .map(|r| r.get::<_, String>("nip"))
            .collect();
        let sumber_ada: HashMap<(String, Uuid), String> = tx
            .query(
                "SELECT nip, spesifikasi_id, sumber
                 FROM perlengkapan.pegawai_pakaian_dinas_ukuran WHERE nip = ANY($1)",
                &[&nips],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .iter()
            .map(|r| ((r.get("nip"), r.get("spesifikasi_id")), r.get("sumber")))
            .collect();

        let mut profil_nip: Vec<String> = Vec::new();
        let mut profil_nama: Vec<String> = Vec::new();
        let mut profil_jk: Vec<Option<String>> = Vec::new();
        let mut profil_hijab: Vec<bool> = Vec::new();
        let (mut tulis_nip, mut tulis_spek, mut tulis_ukuran): (
            Vec<String>,
            Vec<Uuid>,
            Vec<String>,
        ) = (Vec::new(), Vec::new(), Vec::new());
        for b in &sah {
            if ada.contains(&b.nip) {
                hasil.profil_sudah_ada += 1;
            } else {
                hasil.profil_baru += 1;
                profil_nip.push(b.nip.clone());
                profil_nama.push(b.nama.clone());
                profil_jk.push(b.jenis_kelamin.clone());
                profil_hijab.push(b.with_hijab);
            }
            for (spek, ukuran) in &b.ukuran {
                match sumber_ada.get(&(b.nip.clone(), *spek)).map(String::as_str) {
                    None | Some(SUMBER_IMPOR) => {
                        hasil.ukuran_ditulis += 1;
                        tulis_nip.push(b.nip.clone());
                        tulis_spek.push(*spek);
                        tulis_ukuran.push(ukuran.clone());
                    }
                    Some(_) => hasil.ukuran_dilewati_sudah_diisi += 1,
                }
            }
        }
        if dry_run {
            return Ok(hasil); // transaksi dibuang tanpa commit
        }

        // ── penulisan massal ─────────────────────────────────────────────────────────────
        if !profil_nip.is_empty() {
            tx.execute(
                "INSERT INTO perlengkapan.pegawai_pakaian_dinas (nip, nama, jenis_kelamin, with_hijab, updated_at)
                 SELECT t.nip, t.nama, t.jk, t.hijab, now()
                 FROM unnest($1::text[], $2::text[], $3::text[], $4::bool[]) AS t(nip, nama, jk, hijab)
                 ON CONFLICT (nip) DO NOTHING",
                &[&profil_nip, &profil_nama, &profil_jk, &profil_hijab],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        }
        // `WHERE sumber = 'IMPOR'`: bila orangnya mengisi ukuran ini di antara pembacaan di atas
        // dan penulisan ini, isian orangnya yang menang.
        tx.execute(
            "INSERT INTO perlengkapan.pegawai_pakaian_dinas_ukuran (nip, spesifikasi_id, ukuran, sumber)
             SELECT t.nip, t.spek, t.ukuran, 'IMPOR'
             FROM unnest($1::text[], $2::uuid[], $3::text[]) AS t(nip, spek, ukuran)
             ON CONFLICT (nip, spesifikasi_id) DO UPDATE
                SET ukuran = EXCLUDED.ukuran, sumber = 'IMPOR', updated_at = now()
                WHERE perlengkapan.pegawai_pakaian_dinas_ukuran.sumber = 'IMPOR'",
            &[&tulis_nip, &tulis_spek, &tulis_ukuran],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(hasil)
    }
}
