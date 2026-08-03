use super::PakaianDinasRepository;
use crate::pakaian_dinas::models::*;
use crate::shared::error::{AppResult, bad_request};

impl PakaianDinasRepository {
    pub async fn get_pegawai_pakaian_dinas(
        &self,
        nip: &str,
    ) -> AppResult<Option<PegawaiPakaianDinas>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let row = client
            .query_opt(
                "SELECT * FROM perlengkapan.pegawai_pakaian_dinas WHERE nip = $1",
                &[&nip],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(row.map(|r| PegawaiPakaianDinas::from_row(&r)))
    }

    pub async fn upsert_pegawai_pakaian_dinas(
        &self,
        request: &UpdatePersonalUkuranRequest,
        nip: &str,
        nama: Option<&str>,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO perlengkapan.pegawai_pakaian_dinas
                    (nip, nama, ukuran_baju, ukuran_celana, ukuran_sepatu, with_hijab, updated_at)
                VALUES ($1, $2, $3, $4, $5, COALESCE($6, false), $7)
                ON CONFLICT (nip) DO UPDATE SET
                    ukuran_baju = EXCLUDED.ukuran_baju,
                    ukuran_celana = EXCLUDED.ukuran_celana,
                    ukuran_sepatu = EXCLUDED.ukuran_sepatu,
                    -- $6, not EXCLUDED: the INSERT list already collapsed a
                    -- missing flag to `false`, so EXCLUDED cannot tell "the
                    -- caller said false" from "the caller said nothing" and
                    -- would clear a flag this endpoint does not own.
                    with_hijab = COALESCE($6, pegawai_pakaian_dinas.with_hijab),
                    updated_at = EXCLUDED.updated_at
                "#,
                &[
                    &nip,
                    &nama,
                    &request.ukuran_baju,
                    &request.ukuran_celana,
                    &request.ukuran_sepatu,
                    &request.with_hijab,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_pegawai_pakaian_dinas(nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan data ukuran"))
    }

    /// Full profile upsert — writes sizes + reporting fields in one shot.
    /// Used by the wizard and admin bulk-import.
    pub async fn upsert_pegawai_profile(
        &self,
        request: &crate::pakaian_dinas::models::UpsertPegawaiProfileRequest,
    ) -> AppResult<PegawaiPakaianDinas> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        client
            .execute(
                r#"
                INSERT INTO perlengkapan.pegawai_pakaian_dinas
                    (nip, nama, pangkat, jabatan, eselon, jenis_kelamin,
                     jenis_pegawai, with_hijab, mapped_unit_kerja, kode_satker,
                     ukuran_baju, ukuran_celana, ukuran_sepatu, updated_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
                ON CONFLICT (nip) DO UPDATE SET
                    nama = COALESCE(EXCLUDED.nama, pegawai_pakaian_dinas.nama),
                    pangkat = COALESCE(EXCLUDED.pangkat, pegawai_pakaian_dinas.pangkat),
                    jabatan = COALESCE(EXCLUDED.jabatan, pegawai_pakaian_dinas.jabatan),
                    eselon = COALESCE(EXCLUDED.eselon, pegawai_pakaian_dinas.eselon),
                    jenis_kelamin = COALESCE(EXCLUDED.jenis_kelamin, pegawai_pakaian_dinas.jenis_kelamin),
                    jenis_pegawai = COALESCE(EXCLUDED.jenis_pegawai, pegawai_pakaian_dinas.jenis_pegawai),
                    with_hijab = EXCLUDED.with_hijab,
                    mapped_unit_kerja = COALESCE(EXCLUDED.mapped_unit_kerja, pegawai_pakaian_dinas.mapped_unit_kerja),
                    kode_satker = COALESCE(EXCLUDED.kode_satker, pegawai_pakaian_dinas.kode_satker),
                    ukuran_baju = COALESCE(EXCLUDED.ukuran_baju, pegawai_pakaian_dinas.ukuran_baju),
                    ukuran_celana = COALESCE(EXCLUDED.ukuran_celana, pegawai_pakaian_dinas.ukuran_celana),
                    ukuran_sepatu = COALESCE(EXCLUDED.ukuran_sepatu, pegawai_pakaian_dinas.ukuran_sepatu),
                    updated_at = EXCLUDED.updated_at
                "#,
                &[
                    &request.nip,
                    &request.nama,
                    &request.pangkat,
                    &request.jabatan,
                    &request.eselon,
                    &request.jenis_kelamin,
                    &request.jenis_pegawai,
                    &request.with_hijab,
                    &request.mapped_unit_kerja,
                    &request.kode_satker,
                    &request.ukuran_baju,
                    &request.ukuran_celana,
                    &request.ukuran_sepatu,
                    &now,
                ],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        self.get_pegawai_pakaian_dinas(&request.nip)
            .await?
            .ok_or_else(|| bad_request("Gagal menyimpan profil pegawai"))
    }

    /// Bulk upsert profiles — used by the pakaian dinas wizard when
    /// submitting a full satker roster at once.
    pub async fn bulk_upsert_pegawai_profiles(
        &self,
        requests: &[crate::pakaian_dinas::models::UpsertPegawaiProfileRequest],
    ) -> AppResult<usize> {
        if requests.is_empty() {
            return Ok(0);
        }
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;
        let now = chrono::Utc::now();

        let mut count = 0usize;
        for req in requests {
            client
                .execute(
                    r#"
                    INSERT INTO perlengkapan.pegawai_pakaian_dinas
                        (nip, nama, pangkat, jabatan, eselon, jenis_kelamin,
                         jenis_pegawai, with_hijab, mapped_unit_kerja, kode_satker,
                         ukuran_baju, ukuran_celana, ukuran_sepatu, updated_at)
                    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
                    ON CONFLICT (nip) DO UPDATE SET
                        nama = COALESCE(EXCLUDED.nama, pegawai_pakaian_dinas.nama),
                        pangkat = COALESCE(EXCLUDED.pangkat, pegawai_pakaian_dinas.pangkat),
                        jabatan = COALESCE(EXCLUDED.jabatan, pegawai_pakaian_dinas.jabatan),
                        eselon = COALESCE(EXCLUDED.eselon, pegawai_pakaian_dinas.eselon),
                        jenis_kelamin = COALESCE(EXCLUDED.jenis_kelamin, pegawai_pakaian_dinas.jenis_kelamin),
                        jenis_pegawai = COALESCE(EXCLUDED.jenis_pegawai, pegawai_pakaian_dinas.jenis_pegawai),
                        with_hijab = EXCLUDED.with_hijab,
                        mapped_unit_kerja = COALESCE(EXCLUDED.mapped_unit_kerja, pegawai_pakaian_dinas.mapped_unit_kerja),
                        kode_satker = COALESCE(EXCLUDED.kode_satker, pegawai_pakaian_dinas.kode_satker),
                        ukuran_baju = COALESCE(EXCLUDED.ukuran_baju, pegawai_pakaian_dinas.ukuran_baju),
                        ukuran_celana = COALESCE(EXCLUDED.ukuran_celana, pegawai_pakaian_dinas.ukuran_celana),
                        ukuran_sepatu = COALESCE(EXCLUDED.ukuran_sepatu, pegawai_pakaian_dinas.ukuran_sepatu),
                        updated_at = EXCLUDED.updated_at
                    "#,
                    &[
                        &req.nip,
                        &req.nama,
                        &req.pangkat,
                        &req.jabatan,
                        &req.eselon,
                        &req.jenis_kelamin,
                        &req.jenis_pegawai,
                        &req.with_hijab,
                        &req.mapped_unit_kerja,
                        &req.kode_satker,
                        &req.ukuran_baju,
                        &req.ukuran_celana,
                        &req.ukuran_sepatu,
                        &now,
                    ],
                )
                .await
                .map_err(|e| bad_request(&e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    // ============ MySIMKARI Integration ============

    /// `integrasi.mysimkari_pegawai.satker_id` is **TEXT** holding the MySIMKARI
    /// `kode_satker` — integrasi's own child tables key satkers by the business
    /// code, never by the bigint surrogate. This used to take a `Uuid`, which
    /// tokio-postgres refuses to bind to a text column, so the employee lookup
    /// that the whole pakaian-dinas flow depends on failed at runtime (#94).
    pub async fn get_mysimkari_pegawai_by_satker(
        &self,
        satker_code: &str,
    ) -> AppResult<Vec<MysimkariPegawai>> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        let rows = client
            .query(
                r#"
                SELECT * FROM integrasi.mysimkari_pegawai
                WHERE satker_id = $1
                ORDER BY nama ASC
                "#,
                &[&satker_code],
            )
            .await
            .map_err(|e| bad_request(&e.to_string()))?;

        Ok(rows.iter().map(MysimkariPegawai::from_row).collect())
    }

    // ============ Reports ============
}
