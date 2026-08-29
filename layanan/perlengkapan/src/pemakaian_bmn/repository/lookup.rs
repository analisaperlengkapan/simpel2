//! Lookup + ketersediaan BMN untuk alur pemakaian.
//!
//! # NUP saja BUKAN identitas aset
//!
//! Sebuah aset dikenali oleh **kode satker + kode barang + NUP**. NUP adalah
//! "nomor urut pendaftaran" — nomor urut DI DALAM satu satker untuk satu kode
//! barang, jadi ia berulang di seluruh negeri.
//!
//! Diukur pada snapshot SIMAN staging (624.533 baris, 2026-06-17):
//!
//! | | |
//! |---|---|
//! | baris aset | 624.533 |
//! | nilai NUP berbeda | 14.142 |
//! | (satker, kode barang, NUP) berbeda | 620.676 |
//! | rata-rata aset per NUP | 44,2 |
//! | **aset yang memakai NUP `1`** | **44.017, tersebar di 553 satker** |
//!
//! Cek tabrakan yang ber-key NUP saja karena itu memperlakukan 44.017 aset
//! berbeda sebagai satu benda: satu izin aktif atas NUP `1` di mana pun akan
//! memblokir 553 satker lain memakai aset mereka SENDIRI yang kebetulan juga
//! bernomor 1 — sambil menyebutkan nama pegawai satker lain sebagai
//! pemegangnya. Setiap query di berkas ini yang menjawab "apakah aset ini
//! sedang dipakai" WAJIB ber-key pada ketiganya.
//!
//! # Yang BELUM ber-key lengkap (sengaja, di luar cakupan perubahan ini)
//!
//! Dua permukaan BACA masih ber-key NUP saja, dan keduanya butuh keputusan UI
//! lebih dulu — bagaimana peran lintas-satker menyebut aset yang dimaksud —
//! jadi dikerjakan terpisah, bukan diselundupkan ke sini:
//!
//! * `repository::history::get_bmn_usage_history` — riwayat izin diagregasi
//!   per NUP, sehingga riwayat beberapa aset berbeda tercampur.
//! * `bank_aset::repository::find_lookup_by_nup` — `LIMIT 1` tanpa `ORDER BY`
//!   atas kandidat yang, di dalam satu satker saja, rata-rata berjumlah 3,2
//!   dan bisa mencapai 425. Ia mengisi otomatis kode barang di formulir
//!   pemakaian dan menjadi dasar `verify_asset_siman` di penghapusan, yang
//!   lalu melaporkan "kode_barang SIMAN berbeda dari usulan" untuk aset yang
//!   sebenarnya cocok.

use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use crate::shared::repo::PoolExt;
use crate::shared::satker_scope::{BoxedParam, SatkerScope, as_refs};

impl PemakaianBmnRepository {
    /// Lookup pegawai dari cache MySIMKARI by NIP, dibatasi cakupan pemanggil.
    ///
    /// An employee row is named personal data — NIP, name, rank, posting, photo
    /// — so the caller's scope belongs in this signature, applied as a
    /// predicate INSIDE the query. A check the caller performs afterwards is a
    /// check the next caller forgets; that is how #870 leaked another satker's
    /// roster. The predicate is built on the RESOLVED `kode_satker`
    /// ([`crate::shared::pegawai_ref`]), never on `p.satker_id`.
    pub async fn find_pegawai_by_nip(
        &self,
        nip: &str,
        scope: &SatkerScope,
    ) -> AppResult<Option<PegawaiInfo>> {
        let client = self.pool.client().await?;
        let mut params: Vec<BoxedParam> = vec![Box::new(nip.to_string())];
        let scope_sql = scope
            .push_condition(
                crate::shared::pegawai_ref::PEGAWAI_KODE_SATKER_SQL,
                &mut params,
            )
            .map(|c| format!(" AND {c}"))
            .unwrap_or_default();
        let row = client
            .query_opt(
                &format!(
                    r#"
                    SELECT p.nip, p.nama, p.jabatan, p.golpang AS pangkat,
                           {kode_satker} AS satker_id, p.nama_satker, p.foto
                    FROM integrasi.mysimkari_pegawai p
                    {join}
                    WHERE p.nip = $1{scope_sql}
                    "#,
                    join = crate::shared::pegawai_ref::PEGAWAI_SATKER_JOIN_SQL,
                    kode_satker = crate::shared::pegawai_ref::PEGAWAI_KODE_SATKER_SQL,
                ),
                &as_refs(&params),
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.map(|r| PegawaiInfo {
            nip: r.get("nip"),
            nama: r.try_get("nama").ok().flatten(),
            jabatan: r.try_get("jabatan").ok().flatten(),
            pangkat: r.try_get("pangkat").ok().flatten(),
            satker_id: r.try_get("satker_id").ok().flatten(),
            nama_satker: r.try_get("nama_satker").ok().flatten(),
            foto: r.try_get("foto").ok().flatten(),
        }))
    }

    /// Pemakaian BMN yg saat ini aktif utk pegawai.
    pub async fn list_pemakaian_aktif_by_pegawai(
        &self,
        nip: &str,
    ) -> AppResult<Vec<PemakaianAktifEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1 AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&nip],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianAktifEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Histori pemakaian BMN pegawai (status non-aktif: Expired, Revoked,
    /// Rejected, Cancelled). Limit utk avoid blow-up.
    pub async fn list_pemakaian_histori_by_pegawai(
        &self,
        nip: &str,
        limit: i64,
    ) -> AppResult<Vec<PemakaianHistoriEntry>> {
        let client = self.pool.client().await?;
        let rows = client
            .query(
                r#"
                SELECT id, nomor_izin, bmn_nup, bmn_nama_barang, status,
                       tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE pegawai_nip = $1
                  AND status IN ('EXPIRED', 'REVOKED', 'REJECTED', 'CANCELLED')
                ORDER BY tanggal_selesai DESC
                LIMIT $2
                "#,
                &[&nip, &limit],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows
            .iter()
            .map(|r| PemakaianHistoriEntry {
                permit_id: r.get("id"),
                nomor_izin: r.try_get("nomor_izin").ok().flatten(),
                bmn_nup: r.get("bmn_nup"),
                bmn_nama_barang: r.get("bmn_nama_barang"),
                status: r.get("status"),
                tanggal_mulai: r.get("tanggal_mulai"),
                tanggal_selesai: r.get("tanggal_selesai"),
            })
            .collect())
    }

    /// Cek ketersediaan BMN utk periode tertentu (Fase 1.11). Mengembalikan
    /// `Available`, `PemakaianBerurutan` (existing berakhir sebelum
    /// usulan mulai → boleh), atau `Overlap` (tolak).
    ///
    /// Ber-key pada IDENTITAS aset — kode satker + kode barang + NUP — bukan
    /// NUP saja; alasannya ada di header modul ini.
    pub async fn check_bmn_availability_for_period(
        &self,
        bmn_nup: &str,
        bmn_kode_barang: &str,
        satker_code: &str,
        tgl_mulai: chrono::NaiveDate,
        tgl_selesai: chrono::NaiveDate,
    ) -> AppResult<BmnCheckStatus> {
        let client = self.pool.client().await?;
        // Cari izin ACTIVE utk ASET ini, urutkan tanggal_selesai DESC agar
        // izin paling baru di atas. Kita evaluasi overlap thd usulan
        // periode operator.
        let rows = client
            .query(
                r#"
                SELECT pegawai_nama, tanggal_mulai, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE bmn_nup = $1
                  AND bmn_kode_barang = $2
                  AND satker_code = $3
                  AND status = 'ACTIVE'
                ORDER BY tanggal_selesai DESC
                "#,
                &[&bmn_nup, &bmn_kode_barang, &satker_code],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Iterasi: jika ada existing yg overlap dgn (tgl_mulai..tgl_selesai)
        // → Overlap. Jika semua existing berakhir sebelum tgl_mulai dan ada
        // ≥1 existing → PemakaianBerurutan. Jika kosong → Available.
        let mut latest_existing: Option<(String, chrono::NaiveDate)> = None;
        for r in &rows {
            let ex_holder: String = r.get("pegawai_nama");
            let ex_start: chrono::NaiveDate = r.get("tanggal_mulai");
            let ex_end: chrono::NaiveDate = r.get("tanggal_selesai");

            // Overlap = NOT (ex_end < tgl_mulai OR ex_start > tgl_selesai)
            let overlap = !(ex_end < tgl_mulai || ex_start > tgl_selesai);
            if overlap {
                return Ok(BmnCheckStatus::Overlap {
                    existing_holder: ex_holder,
                    existing_sampai_tgl: ex_end,
                });
            }
            if latest_existing
                .as_ref()
                .map(|(_, prev_end)| ex_end > *prev_end)
                .unwrap_or(true)
            {
                latest_existing = Some((ex_holder, ex_end));
            }
        }

        match latest_existing {
            Some((holder, end_tgl)) => Ok(BmnCheckStatus::PemakaianBerurutan {
                existing_holder: holder,
                existing_sampai_tgl: end_tgl,
            }),
            None => Ok(BmnCheckStatus::Available),
        }
    }

    /// Apakah aset ini sedang dipakai?
    ///
    /// Ber-key pada identitas aset penuh (kode satker + kode barang + NUP —
    /// lihat header modul). Karena satker sudah ikut jadi kunci, izin yang
    /// ditemukan PASTI milik satker yang ditanyakan, jadi tak ada lagi
    /// kebocoran nama pemegang lintas satker yang perlu diredaksi: pemanggil
    /// hanya boleh menanyakan satker yang ada dalam scope-nya, dan itu
    /// ditegakkan oleh [`Self::satker_code_in_scope`] di lapisan service.
    ///
    /// Sebelumnya query ini ber-key `bmn_nup` saja dan memakai scope sebagai
    /// kolom terproyeksi (`in_scope`) untuk memutuskan apakah nama pemegang
    /// disebut. Redaksi itu obat untuk gejala: penyebabnya adalah kuncinya
    /// yang salah, dan `is_available = false` dari satker lain tetap SALAH
    /// walau namanya disembunyikan.
    /// Requirements: REQ-P002, REQ-P003
    pub async fn check_bmn_availability(
        &self,
        bmn_nup: &str,
        bmn_kode_barang: &str,
        satker_code: &str,
    ) -> AppResult<BmnAvailabilityResponse> {
        let conflict = self
            .find_booking_conflict(bmn_nup, bmn_kode_barang, satker_code, None)
            .await?;

        Ok(match conflict {
            Some(c) => BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: false,
                active_permit_id: Some(c.permit_id),
                active_permit_holder: Some(c.holder),
                active_permit_expires: Some(c.expires),
            },
            None => BmnAvailabilityResponse {
                bmn_nup: bmn_nup.to_string(),
                is_available: true,
                active_permit_id: None,
                active_permit_holder: None,
                active_permit_expires: None,
            },
        })
    }

    /// Cek tabrakan untuk jalur TULIS (create / renew).
    ///
    /// Mengembalikan `None` bila aset bebas, atau tabrakannya bila sudah ada
    /// izin `ACTIVE` atas aset yang SAMA. `exclude_permit_id` dipakai saat
    /// perpanjangan: izin yang sedang diperpanjang tidak boleh menabrak
    /// dirinya sendiri. Pengecualian itu dilakukan DI SQL, bukan dengan
    /// membandingkan id yang dikembalikan — id yang diredaksi akan membuat
    /// perbandingan itu diam-diam menolak perpanjangan yang sah.
    pub async fn find_booking_conflict(
        &self,
        bmn_nup: &str,
        bmn_kode_barang: &str,
        satker_code: &str,
        exclude_permit_id: Option<uuid::Uuid>,
    ) -> AppResult<Option<BookingConflict>> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                SELECT id, pegawai_nama, tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn
                WHERE bmn_nup = $1
                  AND bmn_kode_barang = $2
                  AND satker_code = $3
                  AND status = 'ACTIVE'
                  AND ($4::uuid IS NULL OR id <> $4::uuid)
                ORDER BY tanggal_selesai DESC
                LIMIT 1
                "#,
                &[&bmn_nup, &bmn_kode_barang, &satker_code, &exclude_permit_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row_opt.map(|row| BookingConflict {
            permit_id: row.get("id"),
            holder: row.get("pegawai_nama"),
            expires: row.get("tanggal_selesai"),
        }))
    }

    /// Tabrakan untuk PERPANJANGAN.
    ///
    /// Asetnya diidentifikasi dari izin yang sedang diperpanjang itu sendiri
    /// (kode satker + kode barang + NUP milik baris itu), jadi satker-nya tak
    /// perlu ditebak dari sesi pemanggil dan izin milik satker lain tak bisa
    /// ikut terhitung. `IS NOT DISTINCT FROM` dipakai agar dua baris legacy
    /// yang sama-sama ber-`satker_code` NULL tetap dianggap bertabrakan —
    /// arah yang aman.
    pub async fn find_renewal_conflict(
        &self,
        permit_id: uuid::Uuid,
    ) -> AppResult<Option<BookingConflict>> {
        let client = self.pool.client().await?;
        let row_opt = client
            .query_opt(
                r#"
                SELECT lain.id, lain.pegawai_nama, lain.tanggal_selesai
                FROM perlengkapan.izin_pemakaian_bmn ini
                JOIN perlengkapan.izin_pemakaian_bmn lain
                  ON  lain.bmn_nup = ini.bmn_nup
                  AND lain.bmn_kode_barang = ini.bmn_kode_barang
                  AND lain.satker_code IS NOT DISTINCT FROM ini.satker_code
                  AND lain.id <> ini.id
                  AND lain.status = 'ACTIVE'
                WHERE ini.id = $1
                ORDER BY lain.tanggal_selesai DESC
                LIMIT 1
                "#,
                &[&permit_id],
            )
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(row_opt.map(|row| BookingConflict {
            permit_id: row.get("id"),
            holder: row.get("pegawai_nama"),
            expires: row.get("tanggal_selesai"),
        }))
    }

    /// Apakah `code` boleh dilihat oleh pemanggil dgn scope ini?
    ///
    /// Kembar dgn `KebutuhanBmnRepository::satker_code_in_scope`: tier murni
    /// (All/Denied/Satker) diputuskan tanpa DB oleh `SatkerScope` sendiri agar
    /// aturannya tak punya salinan kedua; hanya Wilayah yang perlu
    /// `integrasi.v_satker_wilayah`, lewat
    /// `SatkerScope::WILAYAH_MEMBERSHIP_SQL` — satu definisi, bukan satu
    /// salinan per repository.
    pub async fn satker_code_in_scope(
        &self,
        scope: &crate::shared::satker_scope::SatkerScope,
        code: &str,
    ) -> AppResult<bool> {
        use crate::shared::satker_scope::SatkerScope;
        let SatkerScope::Wilayah(caller) = scope else {
            return Ok(scope.contains_code_local(code).unwrap_or(false));
        };

        let client = self.pool.client().await?;
        let row = client
            .query_one(SatkerScope::WILAYAH_MEMBERSHIP_SQL, &[&code, &caller])
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(row.get::<_, bool>("in_scope"))
    }
}
