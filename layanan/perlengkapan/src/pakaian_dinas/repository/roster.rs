//! Pengisian ukuran pakaian dinas per satker.
//!
//! Sampai modul ini ada, TIDAK ADA satu pun jalur di aplikasi yang menulis
//! `pengajuan_pakaian_dinas_satker_pegawai` atau tabel ukurannya — keduanya
//! hanya pernah dibaca (laporan, dasbor) dan dihapus (cascade saat pengajuan
//! dibuang). Baris yang ada di staging datang dari seed e2e, bukan dari
//! aplikasi. Jadi alur pakaian dinas punya ekor persetujuan yang berfungsi
//! tanpa kepala pengisian: tidak ada yang bisa diisi untuk disetujui.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::PakaianDinasRepository;
use super::pegawai::{aturan_grup, kunci_grup_bersama};
use crate::pakaian_dinas::models::*;
use crate::pakaian_dinas::validasi_ukuran::validasi_ukuran;
use crate::shared::error::{AppError, AppResult, bad_request};
use crate::shared::pegawai_ref::{PEGAWAI_KODE_SATKER_SQL, PEGAWAI_SATKER_JOIN_SQL};
use crate::shared::satker_scope::SatkerScope;

/// Status satker yang masih boleh diubah operator: Input dan Dikembalikan.
/// Sama persis dengan dua baris `submit` pada tabel transisi di `services.rs`
/// — kalau daftar ini menyimpang, layar akan mengizinkan pengeditan yang lalu
/// ditolak server saat "Ajukan".
pub const STATUS_DAPAT_DIUBAH: [i32; 2] = [1000, 1003];

impl PakaianDinasRepository {
    /// Kolom ukuran yang diminta satu pengajuan.
    pub async fn list_pakaian_pengajuan(
        &self,
        pengajuan_id: Uuid,
    ) -> AppResult<Vec<PengajuanPakaianItem>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let rows = client
            .query(
                r#"
                SELECT id, jenis_pakaian_nama, spesifikasi_id, spesifikasi_nama,
                       spesifikasi_ukuran_group, subspesifikasi_id, subspesifikasi_nama,
                       subspesifikasi_gender
                FROM perlengkapan.pengajuan_pakaian_dinas_pakaian
                WHERE pengajuan_id = $1
                ORDER BY jenis_pakaian_nama, spesifikasi_nama, subspesifikasi_nama NULLS FIRST
                "#,
                &[&pengajuan_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        Ok(rows.iter().map(PengajuanPakaianItem::from_row).collect())
    }

    /// Baris satker pada satu pengajuan, dicari lewat kode satker.
    ///
    /// Di-scope seperti pembacaan satker lain: di luar lingkup menjawab
    /// NotFound, bukan Forbidden, supaya keberadaan baris satker lain tidak
    /// bisa disimpulkan dari kode status (#93).
    ///
    /// Kembarannya `roster_snapshot_at` ikut dibawa karena setiap pemakaian
    /// baris ini menentukan apakah daftar pegawai dibaca dari snapshot atau
    /// live (V018).
    async fn pengajuan_satker_row(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<SatkerRow> {
        if !self.satker_code_in_scope(scope, satker_code).await? {
            return Err(AppError::NotFound(format!(
                "Satker tidak ditemukan: {satker_code}"
            )));
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let row = client
            .query_opt(
                r#"
                SELECT ps.id, ps.aktivitas_id, s.nama_satker, p.nama AS pengajuan_nama,
                       p.roster_snapshot_at
                FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
                JOIN perlengkapan.pengajuan_pakaian_dinas p ON p.id = ps.pengajuan_id
                LEFT JOIN integrasi.mysimkari_satker s ON ps.satker_id = s.kode_satker
                WHERE ps.pengajuan_id = $1 AND ps.satker_id = $2
                "#,
                &[&pengajuan_id, &satker_code],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?
            .ok_or_else(|| {
                AppError::NotFound("Satker tidak terdaftar pada pengajuan ini".to_string())
            })?;
        Ok(SatkerRow {
            id: row.get("id"),
            aktivitas_id: row.get("aktivitas_id"),
            nama_satker: row.try_get("nama_satker").ok().flatten(),
            pengajuan_nama: row.get("pengajuan_nama"),
            roster_snapshot_at: row.try_get("roster_snapshot_at").ok().flatten(),
        })
    }

    /// Daftar pegawai satu satker pada satu pengajuan: dari snapshot beku
    /// (V018) bila kampanye punya `roster_snapshot_at`, selain itu live dari
    /// kepegawaian.
    ///
    /// Bentuk barisnya SENGAJA sama pada kedua cabang supaya pemanggil tak
    /// perlu tahu asalnya; hanya `roster_snapshot_at` yang membedakannya.
    async fn daftar_pegawai_satker(
        &self,
        client: &tokio_postgres::Client,
        pengajuan_id: Uuid,
        satker_code: &str,
        snapshot_at: Option<DateTime<Utc>>,
    ) -> AppResult<Vec<tokio_postgres::Row>> {
        // "Keluarkan dari Pengajuan" deletes the participation row but not the
        // roster member; the exclusion table (V020) is what keeps the person out
        // of every readiness denominator. The row is still RETURNED, flagged, so
        // the operator can put the person back — hiding it here stranded them:
        // the only way to undo an exclusion was to save a size for a form the
        // list no longer offered.
        let rows = if snapshot_at.is_some() {
            // Snapshot beku: orangnya persis seperti saat kampanye dibuat.
            client
                .query(
                    r#"
                    SELECT snap.nip, snap.nama_pegawai AS nama, snap.jabatan,
                           snap.pangkat AS golpang, snap.gol_kd, snap.eselon,
                           snap.jenis_kelamin AS jk, snap.jenis AS jenis_jabatan_terakhir,
                           snap.foto,
                           EXISTS (
                               SELECT 1
                               FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan d
                               WHERE d.pengajuan_id = snap.pengajuan_id
                                 AND d.satker_id = snap.satker_id
                                 AND d.nip = snap.nip
                           ) AS dikeluarkan
                    FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot snap
                    WHERE snap.pengajuan_id = $1 AND snap.satker_id = $2
                    ORDER BY snap.nama_pegawai ASC
                    "#,
                    &[&pengajuan_id, &satker_code],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
        } else {
            client
                .query(
                    &format!(
                        r#"
                        SELECT p.nip, p.nama, p.jabatan, p.golpang, p.gol_kd, p.eselon,
                               p.jk, p.jenis_jabatan_terakhir, p.foto,
                               EXISTS (
                                   SELECT 1
                                   FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan d
                                   WHERE d.pengajuan_id = $1
                                     AND d.satker_id = $2
                                     AND d.nip = p.nip
                               ) AS dikeluarkan
                        FROM integrasi.mysimkari_pegawai p
                        {join}
                        WHERE {kode_satker} = $2
                        ORDER BY p.nama ASC
                        "#,
                        join = PEGAWAI_SATKER_JOIN_SQL,
                        kode_satker = PEGAWAI_KODE_SATKER_SQL,
                    ),
                    &[&pengajuan_id, &satker_code],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
        };
        Ok(rows)
    }

    /// Daftar pengisian: kolom kampanye + seluruh pegawai satker + ukuran yang
    /// sudah tersimpan.
    pub async fn get_roster_pengisian(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<RosterPengisian> {
        let satker = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        let pengajuan_satker_id = satker.id;
        let aktivitas_id = satker.aktivitas_id;
        let pakaian = self.list_pakaian_pengajuan(pengajuan_id).await?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Ukuran yang sudah tersimpan, dikelompokkan per NIP. Diambil sekali
        // lalu dipetakan di memori: satu kueri per pegawai akan menjadi ratusan
        // kueri pada satker besar.
        let rows = client
            .query(
                r#"
                SELECT psp.nip, psp.with_hijab, pu.pakaian_id, pu.ukuran
                FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp
                LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran pu
                       ON pu.pegawai_id = psp.id
                WHERE psp.pengajuan_satker_id = $1
                "#,
                &[&pengajuan_satker_id],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let mut tersimpan: HashMap<String, (bool, Vec<UkuranPegawaiItem>)> = HashMap::new();
        for row in &rows {
            let nip: String = row.get("nip");
            let with_hijab: bool = row.try_get("with_hijab").unwrap_or(false);
            let entry = tersimpan.entry(nip).or_insert((with_hijab, Vec::new()));
            entry.0 = with_hijab;
            if let (Ok(Some(pakaian_id)), Ok(Some(ukuran))) = (
                row.try_get::<_, Option<Uuid>>("pakaian_id"),
                row.try_get::<_, Option<String>>("ukuran"),
            ) {
                entry.1.push(UkuranPegawaiItem { pakaian_id, ukuran });
            }
        }

        // Orangnya: dari snapshot beku bila kampanye punya satu, selain itu
        // dari kepegawaian. Sebelum V018 ini SELALU live, jadi pegawai yang
        // mutasi masuk/keluar mengubah kampanye lama. Pegawai yang baru masuk
        // satker tetap muncul pada kampanye tanpa snapshot.
        let pegawai_rows = self
            .daftar_pegawai_satker(
                &client,
                pengajuan_id,
                satker_code,
                satker.roster_snapshot_at,
            )
            .await?;

        // Ukuran yang sudah tercatat di profil tiap pegawai (halaman Ukuran),
        // diambil sekali untuk seluruh satker. Operator tidak mengetik ulang data
        // yang sudah ada di database; ia memeriksa lalu menyimpan.
        let nips: Vec<String> = pegawai_rows.iter().map(|r| r.get("nip")).collect();
        // Ukuran profil DIBACA PER SPESIFIKASI (V019): sepatu PDH dan sepatu PDL, kaos dan
        // baju, masing-masing punya ukurannya sendiri. Sebelumnya hanya satu ukuran per grup,
        // jadi kolom-kolom dalam grup yang sama mendapat saran yang sama.
        let profil_rows = client
            .query(
                "SELECT nip, with_hijab \
                 FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = ANY($1)",
                &[&nips],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let ukuran_rows = client
            .query(
                "SELECT nip, spesifikasi_id, ukuran \
                 FROM perlengkapan.pegawai_pakaian_dinas_ukuran WHERE nip = ANY($1)",
                &[&nips],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let mut profil: HashMap<String, ProfilUkuran> = profil_rows
            .iter()
            .map(|r| {
                (
                    r.get::<_, String>("nip"),
                    ProfilUkuran {
                        per_spesifikasi: HashMap::new(),
                        with_hijab: r.try_get("with_hijab").unwrap_or(false),
                    },
                )
            })
            .collect();
        for r in &ukuran_rows {
            if let Some(p) = profil.get_mut(r.get::<_, &str>("nip")) {
                p.per_spesifikasi
                    .insert(r.get("spesifikasi_id"), r.get("ukuran"));
            }
        }

        let pegawai = pegawai_rows
            .iter()
            .map(|row| {
                let nip: String = row.get("nip");
                let simpanan = tersimpan.get(&nip);
                // A saved EMPTY form creates the participation row but no sizes.
                // Counting the row as filled hid those people from the operator
                // dialog and from both readiness counts.
                let sudah_diisi = simpanan.is_some_and(|(_, ukuran)| !ukuran.is_empty());
                let (tersimpan_hijab, ukuran) = simpanan.cloned().unwrap_or((false, Vec::new()));
                // Saran hanya untuk yang belum diisi: yang sudah diisi memakai
                // isinya sendiri.
                let saran = profil
                    .get(&nip)
                    .filter(|_| !sudah_diisi)
                    .map(|p| p.saran_untuk(&pakaian))
                    .unwrap_or_default();
                // Hijab: pilihan yang SUDAH tersimpan pada pengajuan menang,
                // termasuk saat formulirnya masih kosong — operator bisa
                // menyimpan centang lebih dulu dan ukurannya menyusul. Profil
                // hanya menjadi nilai awal bila baris partisipasinya belum ada.
                let with_hijab = match simpanan {
                    Some(_) => tersimpan_hijab,
                    None => profil.get(&nip).is_some_and(|p| p.with_hijab),
                };
                RosterPegawai {
                    sudah_diisi,
                    dikeluarkan: row.get("dikeluarkan"),
                    saran,
                    nip,
                    nama: row.get("nama"),
                    jabatan: row.try_get("jabatan").ok().flatten(),
                    pangkat: row.try_get("golpang").ok().flatten(),
                    gol_kd: row.try_get("gol_kd").ok().flatten(),
                    eselon: row.try_get("eselon").ok().flatten(),
                    // Kolom `jk` NOT NULL di sumber, tapi CHECK pada tabel
                    // pengajuan hanya menerima L/P — nilai lain ditolak saat
                    // menyimpan, bukan didiamkan menjadi baris tanpa gender.
                    jenis_kelamin: row.try_get("jk").unwrap_or_else(|_| "L".to_string()),
                    jenis: row.try_get("jenis_jabatan_terakhir").ok().flatten(),
                    foto: row.try_get("foto").ok().flatten(),
                    with_hijab,
                    ukuran,
                }
            })
            .collect();

        Ok(RosterPengisian {
            pengajuan_satker_id,
            pengajuan_nama: satker.pengajuan_nama,
            satker_kode: satker_code.to_string(),
            satker_nama: satker.nama_satker,
            aktivitas_id,
            dapat_diubah: STATUS_DAPAT_DIUBAH.contains(&aktivitas_id),
            roster_snapshot_at: satker.roster_snapshot_at,
            pakaian,
            pegawai,
        })
    }

    /// Simpan ukuran satu pegawai.
    ///
    /// Satu transaksi: baris pegawai di-upsert, ukurannya diganti seluruhnya.
    /// Mengganti alih-alih menambah supaya menghapus pilihan benar-benar
    /// menghapusnya — kalau tidak, ukuran lama akan bertahan di laporan
    /// padahal operator sudah mengosongkannya di layar.
    pub async fn simpan_ukuran_pegawai(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        nip: &str,
        request: &SimpanUkuranPegawaiRequest,
        scope: &SatkerScope,
    ) -> AppResult<()> {
        let satker = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        let pengajuan_satker_id = satker.id;
        let aktivitas_id = satker.aktivitas_id;
        if !STATUS_DAPAT_DIUBAH.contains(&aktivitas_id) {
            return Err(bad_request(
                "Pengajuan satker ini sudah diajukan, ukurannya tidak dapat diubah lagi",
            ));
        }
        self.pastikan_periode(pengajuan_id, true).await?;

        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Identitas dibekukan dari SoT, bukan dari badan permintaan. Bila
        // kampanye punya snapshot (V018), identitasnya datang dari snapshot itu
        // supaya baris pengajuan sejalan dengan daftar yang sudah dibekukan —
        // bukan dari MySIMKARI yang mungkin sudah berubah sejak kampanye dibuat.
        let identitas = if satker.roster_snapshot_at.is_some() {
            client
                .query_opt(
                    r#"
                    SELECT nip, nama_pegawai AS nama, jabatan, pangkat AS golpang, gol_kd, eselon,
                           jenis_kelamin AS jk, jenis AS jenis_jabatan_terakhir
                    FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_snapshot
                    WHERE pengajuan_id = $1 AND satker_id = $2 AND nip = $3
                    "#,
                    &[&pengajuan_id, &satker_code, &nip],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
        } else {
            client
                .query_opt(
                    &format!(
                        r#"
                        SELECT p.nip, p.nama, p.jabatan, p.golpang, p.gol_kd, p.eselon,
                               p.jk, p.jenis_jabatan_terakhir
                        FROM integrasi.mysimkari_pegawai p
                        {join}
                        WHERE {kode_satker} = $1 AND p.nip = $2
                        "#,
                        join = PEGAWAI_SATKER_JOIN_SQL,
                        kode_satker = PEGAWAI_KODE_SATKER_SQL,
                    ),
                    &[&satker_code, &nip],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
        }
        .ok_or_else(|| {
            AppError::NotFound(format!(
                "Pegawai {nip} tidak terdaftar pada satker {satker_code}"
            ))
        })?;

        let jk: String = identitas.try_get("jk").unwrap_or_else(|_| String::new());
        if jk != "L" && jk != "P" {
            return Err(bad_request(&format!(
                "Jenis kelamin pegawai {nip} tidak tercatat di data kepegawaian, \
                 sehingga ukurannya belum dapat disimpan"
            )));
        }

        let nama: String = identitas.get("nama");
        let jabatan: Option<String> = identitas.try_get("jabatan").ok().flatten();
        let pangkat: Option<String> = identitas.try_get("golpang").ok().flatten();
        let gol_kd: Option<String> = identitas.try_get("gol_kd").ok().flatten();
        let eselon: Option<String> = identitas.try_get("eselon").ok().flatten();
        let jenis: Option<String> = identitas.try_get("jenis_jabatan_terakhir").ok().flatten();

        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        kunci_baris_satker(&tx, pengajuan_satker_id).await?;

        let pegawai_row = tx
            .query_opt(
                r#"
                SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
                WHERE pengajuan_satker_id = $1 AND nip = $2
                "#,
                &[&pengajuan_satker_id, &nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let pegawai_id: Uuid = match pegawai_row {
            Some(row) => {
                let id: Uuid = row.get("id");
                tx.execute(
                    r#"
                    UPDATE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
                    SET nama = $2, pangkat = $3, jabatan = $4, eselon = $5,
                        jenis_kelamin = $6, gol_kd = $7, jenis = $8, with_hijab = $9
                    WHERE id = $1
                    "#,
                    &[
                        &id,
                        &nama,
                        &pangkat,
                        &jabatan,
                        &eselon,
                        &jk,
                        &gol_kd,
                        &jenis,
                        &request.with_hijab,
                    ],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
                id
            }
            None => tx
                .query_one(
                    r#"
                    INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
                        (pengajuan_satker_id, nip, nama, pangkat, jabatan, eselon,
                         jenis_kelamin, gol_kd, jenis, with_hijab)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                    RETURNING id
                    "#,
                    &[
                        &pengajuan_satker_id,
                        &nip,
                        &nama,
                        &pangkat,
                        &jabatan,
                        &eselon,
                        &jk,
                        &gol_kd,
                        &jenis,
                        &request.with_hijab,
                    ],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
                .get("id"),
        };

        tx.execute(
            "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
             WHERE pegawai_id = $1",
            &[&pegawai_id],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        // Saving a size is the operator saying "this person IS in the
        // submission", so a previous "Keluarkan" no longer applies.
        tx.execute(
            "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan
             WHERE pengajuan_id = $1 AND satker_id = $2 AND nip = $3",
            &[&pengajuan_id, &satker_code, &nip],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;

        // Aturan grup, dimuat sekali per grup yang muncul di permintaan ini.
        let mut aturan: HashMap<String, (UkuranGrup, Vec<String>)> = HashMap::new();

        for item in &request.ukuran {
            if item.ukuran.trim().is_empty() {
                continue;
            }
            // Kolom harus milik pengajuan ini. Tanpa penjagaan ini sebuah
            // `pakaian_id` dari kampanye lain akan tersimpan dan muncul di
            // laporan kampanye yang salah. Grup ukuran kolom itu (yang DISALIN ke kolom
            // kampanye saat dibuat) ikut dibaca di sini untuk memeriksa nilainya.
            let milik = tx
                .query_opt(
                    "SELECT spesifikasi_nama, spesifikasi_ukuran_group
                     FROM perlengkapan.pengajuan_pakaian_dinas_pakaian
                     WHERE id = $1 AND pengajuan_id = $2",
                    &[&item.pakaian_id, &pengajuan_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?
                .ok_or_else(|| {
                    bad_request("Ada ukuran yang menunjuk jenis pakaian di luar pengajuan ini")
                })?;
            let nama_kolom: String = milik.get("spesifikasi_nama");
            let grup_kolom: String = milik.get("spesifikasi_ukuran_group");
            // Aturan yang sama dengan profil: pilihan harus dari daftar grupnya, teks dibatasi
            // panjangnya. Grup yang sudah tak ada (kolom kampanye lama) dilewati, bukan ditolak:
            // kampanye itu sah ketika dibuat.
            if !aturan.contains_key(&grup_kolom) {
                // Kunci bersama sebelum daftar dibaca: lihat `kunci_grup_bersama`.
                kunci_grup_bersama(&tx, std::slice::from_ref(&grup_kolom)).await?;
                aturan.extend(aturan_grup(&tx, std::slice::from_ref(&grup_kolom)).await?);
            }
            let ukuran_sah = match aturan.get(&grup_kolom) {
                Some((grup, daftar)) => validasi_ukuran(grup, daftar, &item.ukuran)
                    .map_err(|e| bad_request(&format!("{nama_kolom}: {e}")))?,
                None => item.ukuran.trim().to_string(),
            };
            tx.execute(
                r#"
                INSERT INTO perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
                    (pengajuan_satker_id, pegawai_id, pakaian_id, ukuran)
                VALUES ($1, $2, $3, $4)
                "#,
                &[
                    &pengajuan_satker_id,
                    &pegawai_id,
                    &item.pakaian_id,
                    &ukuran_sah,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        }

        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }

    /// Keluarkan satu pegawai dari pengajuan.
    ///
    /// Untuk yang memang tidak memerlukan pakaian dinas. Barisnya dihapus, bukan
    /// ditandai, supaya laporan tidak perlu tahu soal penanda dan tidak ada
    /// pegawai berukuran kosong yang ikut terhitung.
    pub async fn hapus_pegawai_dari_pengajuan(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        nip: &str,
        scope: &SatkerScope,
    ) -> AppResult<()> {
        let satker = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        let pengajuan_satker_id = satker.id;
        let aktivitas_id = satker.aktivitas_id;
        if !STATUS_DAPAT_DIUBAH.contains(&aktivitas_id) {
            return Err(bad_request(
                "Pengajuan satker ini sudah diajukan, isinya tidak dapat diubah lagi",
            ));
        }
        self.pastikan_periode(pengajuan_id, true).await?;
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        kunci_baris_satker(&tx, pengajuan_satker_id).await?;
        tx.execute(
            r#"
            DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran
            WHERE pegawai_id IN (
                SELECT id FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
                WHERE pengajuan_satker_id = $1 AND nip = $2
            )
            "#,
            &[&pengajuan_satker_id, &nip],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        tx.execute(
            "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai
             WHERE pengajuan_satker_id = $1 AND nip = $2",
            &[&pengajuan_satker_id, &nip],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        // The roster (frozen or live) still names this person. Record the
        // exclusion so readiness stops counting them as "belum mengisi" and the
        // roster list stops offering them.
        tx.execute(
            "INSERT INTO perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan
                 (pengajuan_id, satker_id, nip)
             VALUES ($1, $2, $3)
             ON CONFLICT (pengajuan_id, satker_id, nip) DO NOTHING",
            &[&pengajuan_id, &satker_code, &nip],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }

    /// Masukkan kembali pegawai yang sebelumnya dikeluarkan. Hanya pengecualian
    /// yang dihapus: ukurannya tetap kosong sampai operator mengisinya, jadi
    /// orang itu kembali muncul sebagai "belum mengisi" — persis seperti
    /// sebelum ia dikeluarkan.
    pub async fn masukkan_kembali_pegawai(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        nip: &str,
        scope: &SatkerScope,
    ) -> AppResult<()> {
        let satker = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        let pengajuan_satker_id = satker.id;
        if !STATUS_DAPAT_DIUBAH.contains(&satker.aktivitas_id) {
            return Err(bad_request(
                "Pengajuan satker ini sudah diajukan, isinya tidak dapat diubah lagi",
            ));
        }
        self.pastikan_periode(pengajuan_id, true).await?;
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        kunci_baris_satker(&tx, pengajuan_satker_id).await?;
        let kembali = tx
            .execute(
                "DELETE FROM perlengkapan.pengajuan_pakaian_dinas_pegawai_dikeluarkan
                 WHERE pengajuan_id = $1 AND satker_id = $2 AND nip = $3",
                &[&pengajuan_id, &satker_code, &nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        if kembali == 0 {
            return Err(AppError::NotFound(format!(
                "Pegawai {nip} tidak sedang dikeluarkan dari pengajuan ini"
            )));
        }
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }
}

/// Serialise everything that writes one satker's roster (saving a size,
/// excluding an employee, putting them back). Without it a save and an
/// exclusion for the same NIP can both commit — leaving a size row AND an
/// exclusion, so the person is hidden from the roster yet still counted in the
/// procurement recap.
async fn kunci_baris_satker(
    tx: &tokio_postgres::Transaction<'_>,
    pengajuan_satker_id: Uuid,
) -> AppResult<()> {
    let locked = tx
        .query_opt(
            "SELECT 1 FROM perlengkapan.pengajuan_pakaian_dinas_satker WHERE id = $1 FOR UPDATE",
            &[&pengajuan_satker_id],
        )
        .await
        .map_err(|e| bad_request(&e.to_string()))?;
    if locked.is_none() {
        return Err(AppError::NotFound(
            "Pengajuan satker tidak ditemukan".to_string(),
        ));
    }
    Ok(())
}

/// Baris satker satu pengajuan, apa yang dibutuhkan pemanggilnya.
///
/// `roster_snapshot_at` menentukan dari mana daftar pegawai dibaca (V018):
/// `Some` = snapshot beku kampanye, `None` = live dari MySIMKARI.
struct SatkerRow {
    id: Uuid,
    aktivitas_id: i32,
    nama_satker: Option<String>,
    pengajuan_nama: String,
    roster_snapshot_at: Option<DateTime<Utc>>,
}

/// What a pegawai's own profile (the Ukuran page) already says about their sizes: one size
/// per pakaian SPESIFIKASI (V019), not one per size group.
struct ProfilUkuran {
    per_spesifikasi: HashMap<Uuid, String>,
    with_hijab: bool,
}

impl ProfilUkuran {
    /// The profile's size for each campaign column, by the column's spesifikasi. A column whose
    /// spesifikasi the profile has no size for gets no suggestion, and so does an empty size.
    fn saran_untuk(&self, kolom: &[PengajuanPakaianItem]) -> Vec<UkuranPegawaiItem> {
        kolom
            .iter()
            .filter_map(|item| {
                let ukuran = self.per_spesifikasi.get(&item.spesifikasi_id)?.trim();
                (!ukuran.is_empty()).then(|| UkuranPegawaiItem {
                    pakaian_id: item.id,
                    ukuran: ukuran.to_string(),
                })
            })
            .collect()
    }
}
