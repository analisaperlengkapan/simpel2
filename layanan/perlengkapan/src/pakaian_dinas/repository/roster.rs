//! Pengisian ukuran pakaian dinas per satker.
//!
//! Sampai modul ini ada, TIDAK ADA satu pun jalur di aplikasi yang menulis
//! `pengajuan_pakaian_dinas_satker_pegawai` atau tabel ukurannya — keduanya
//! hanya pernah dibaca (laporan, dasbor) dan dihapus (cascade saat pengajuan
//! dibuang). Baris yang ada di staging datang dari seed e2e, bukan dari
//! aplikasi. Jadi alur pakaian dinas punya ekor persetujuan yang berfungsi
//! tanpa kepala pengisian: tidak ada yang bisa diisi untuk disetujui.

use std::collections::HashMap;

use uuid::Uuid;

use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
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
    async fn pengajuan_satker_row(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<(Uuid, i32, Option<String>, String)> {
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
                SELECT ps.id, ps.aktivitas_id, s.nama_satker, p.nama AS pengajuan_nama
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
        Ok((
            row.get("id"),
            row.get("aktivitas_id"),
            row.try_get("nama_satker").ok().flatten(),
            row.get("pengajuan_nama"),
        ))
    }

    /// Daftar pengisian: kolom kampanye + seluruh pegawai satker + ukuran yang
    /// sudah tersimpan.
    pub async fn get_roster_pengisian(
        &self,
        pengajuan_id: Uuid,
        satker_code: &str,
        scope: &SatkerScope,
    ) -> AppResult<RosterPengisian> {
        let (pengajuan_satker_id, aktivitas_id, satker_nama, pengajuan_nama) = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
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

        // Orangnya sendiri datang dari kepegawaian, bukan dari tabel pengajuan:
        // pegawai yang baru masuk satker harus muncul tanpa perlu ditambahkan
        // manual, dan yang sudah pindah tidak lagi ditawarkan.
        let pegawai_rows = client
            .query(
                &format!(
                    r#"
                    SELECT p.nip, p.nama, p.jabatan, p.golpang, p.gol_kd, p.eselon,
                           p.jk, p.jenis_jabatan_terakhir, p.foto
                    FROM integrasi.mysimkari_pegawai p
                    {join}
                    WHERE {kode_satker} = $1
                    ORDER BY p.nama ASC
                    "#,
                    join = PEGAWAI_SATKER_JOIN_SQL,
                    kode_satker = PEGAWAI_KODE_SATKER_SQL,
                ),
                &[&satker_code],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let pegawai = pegawai_rows
            .iter()
            .map(|row| {
                let nip: String = row.get("nip");
                let (with_hijab, ukuran) =
                    tersimpan.get(&nip).cloned().unwrap_or((false, Vec::new()));
                RosterPegawai {
                    sudah_diisi: tersimpan.contains_key(&nip),
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
            pengajuan_nama,
            satker_kode: satker_code.to_string(),
            satker_nama,
            aktivitas_id,
            dapat_diubah: STATUS_DAPAT_DIUBAH.contains(&aktivitas_id),
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
        let (pengajuan_satker_id, aktivitas_id, _, _) = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        if !STATUS_DAPAT_DIUBAH.contains(&aktivitas_id) {
            return Err(bad_request(
                "Pengajuan satker ini sudah diajukan, ukurannya tidak dapat diubah lagi",
            ));
        }

        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        // Identitas dibekukan dari SoT, bukan dari badan permintaan.
        let identitas = client
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

        for item in &request.ukuran {
            if item.ukuran.trim().is_empty() {
                continue;
            }
            // Kolom harus milik pengajuan ini. Tanpa penjagaan ini sebuah
            // `pakaian_id` dari kampanye lain akan tersimpan dan muncul di
            // laporan kampanye yang salah.
            let milik = tx
                .query_one(
                    "SELECT EXISTS(
                        SELECT 1 FROM perlengkapan.pengajuan_pakaian_dinas_pakaian
                        WHERE id = $1 AND pengajuan_id = $2
                     ) AS ada",
                    &[&item.pakaian_id, &pengajuan_id],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            if !milik.get::<_, bool>("ada") {
                return Err(bad_request(
                    "Ada ukuran yang menunjuk jenis pakaian di luar pengajuan ini",
                ));
            }
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
                    &item.ukuran.trim(),
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
        let (pengajuan_satker_id, aktivitas_id, _, _) = self
            .pengajuan_satker_row(pengajuan_id, satker_code, scope)
            .await?;
        if !STATUS_DAPAT_DIUBAH.contains(&aktivitas_id) {
            return Err(bad_request(
                "Pengajuan satker ini sudah diajukan, isinya tidak dapat diubah lagi",
            ));
        }
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let tx = client
            .transaction()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
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
        tx.commit().await.map_err(|e| bad_request(&e.to_string()))?;
        Ok(())
    }
}
