//! The asset list of a permit — the only place a BMN lives since V014.
//!
//! These are associated functions taking the caller's `Transaction` rather
//! than methods taking `&self`, because a permit and its assets have to be
//! written together. A permit whose list is half-written is not a smaller
//! permit, it is a wrong one: the SK prints the assets that made it and the
//! conflict check leaves the ones that did not looking free.

use super::PemakaianBmnRepository;
use crate::pemakaian_bmn::models::*;
use crate::shared::error::{AppError, AppResult};
use chrono::NaiveDate;
use tokio_postgres::{Row, Transaction};
use uuid::Uuid;

impl PemakaianBmnRepository {
    /// Write the permit's assets, numbered in the order given.
    ///
    /// `urutan` is stored rather than derived from `created_at`, because it is
    /// what orders the table on page 2 of the SK — re-saving a permit must not
    /// shuffle the letter. The period falls back to the permit's default, so a
    /// stored row always states its own dates and no reader has to know about
    /// the fallback.
    pub(crate) async fn insert_items(
        tx: &Transaction<'_>,
        izin_pemakaian_id: Uuid,
        items: &[CreateBmnItemRequest],
        default_mulai: NaiveDate,
        default_selesai: NaiveDate,
    ) -> AppResult<Vec<PemakaianBmnItem>> {
        let query = r#"
            INSERT INTO perlengkapan.pemakaian_bmn_items (
                izin_pemakaian_id, urutan, jenis_bmn,
                bmn_nup, bmn_kode_barang, bmn_nama_barang, bmn_merk, bmn_tipe,
                bmn_tahun_perolehan, bmn_kondisi,
                no_polisi, no_bpkb, no_stnk, no_rangka, no_mesin,
                alamat, luas_tanah, luas_bangunan,
                serial_number, spesifikasi, lokasi_pemakaian,
                tanggal_mulai, tanggal_selesai,
                detail_bmn, keterangan, created_at, updated_at
            ) VALUES (
                $1, $2, $3,
                $4, $5, $6, $7, $8,
                $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17, $18,
                $19, $20, $21,
                $22, $23,
                COALESCE($24, '{}'::jsonb), $25, NOW(), NOW()
            )
            RETURNING *
        "#;

        let mut hasil = Vec::with_capacity(items.len());
        for (i, item) in items.iter().enumerate() {
            let urutan = (i + 1) as i32;
            let mulai = item.tanggal_mulai.unwrap_or(default_mulai);
            let selesai = item.tanggal_selesai.unwrap_or(default_selesai);
            let row = tx
                .query_one(
                    query,
                    &[
                        &izin_pemakaian_id,
                        &urutan,
                        &item.jenis_bmn,
                        &item.bmn_nup,
                        &item.bmn_kode_barang,
                        &item.bmn_nama_barang,
                        &item.bmn_merk,
                        &item.bmn_tipe,
                        &item.bmn_tahun_perolehan,
                        &item.bmn_kondisi,
                        &item.no_polisi,
                        &item.no_bpkb,
                        &item.no_stnk,
                        &item.no_rangka,
                        &item.no_mesin,
                        &item.alamat,
                        &item.luas_tanah,
                        &item.luas_bangunan,
                        &item.serial_number,
                        &item.spesifikasi,
                        &item.lokasi_pemakaian,
                        &mulai,
                        &selesai,
                        &item.detail_bmn,
                        &item.keterangan,
                    ],
                )
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            hasil.push(PemakaianBmnItem::from_row(&row));
        }
        Ok(hasil)
    }

    /// Replace a permit's whole asset list. Delete-then-insert rather than a
    /// diff: `urutan` is unique per permit, so an in-place reorder would have
    /// to dodge its own uniqueness constraint mid-update. The permit is in
    /// DRAFT when this runs — nothing has been signed over the old list.
    pub(crate) async fn replace_items(
        tx: &Transaction<'_>,
        izin_pemakaian_id: Uuid,
        items: &[CreateBmnItemRequest],
        default_mulai: NaiveDate,
        default_selesai: NaiveDate,
    ) -> AppResult<Vec<PemakaianBmnItem>> {
        tx.execute(
            "DELETE FROM perlengkapan.pemakaian_bmn_items WHERE izin_pemakaian_id = $1",
            &[&izin_pemakaian_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Self::insert_items(tx, izin_pemakaian_id, items, default_mulai, default_selesai).await
    }

    /// Recompute the permit's overall window from its assets.
    ///
    /// The header's `tanggal_mulai`/`tanggal_selesai` are DERIVED — MIN/MAX
    /// over the items — and this is their only writer. They stay on the header
    /// because the expiry scheduler, two dozen indexes and every monitoring
    /// query read them; a derived column with one writer and a test pinning
    /// the derivation is a smaller risk than rewriting all of those. The test
    /// is `pemakaian_header_detail_test::the_permit_window_tracks_its_items`.
    ///
    /// Returns the updated header row, or `None` when the permit has no items
    /// (then there is no window to derive and the caller keeps what it had).
    pub(crate) async fn sync_permit_window(
        tx: &Transaction<'_>,
        izin_pemakaian_id: Uuid,
    ) -> AppResult<Option<Row>> {
        tx.query_opt(
            r#"
            UPDATE perlengkapan.izin_pemakaian_bmn p
            SET tanggal_mulai   = w.mulai,
                tanggal_selesai = w.selesai,
                updated_at      = NOW()
            FROM (
                SELECT MIN(tanggal_mulai) AS mulai, MAX(tanggal_selesai) AS selesai
                FROM perlengkapan.pemakaian_bmn_items
                WHERE izin_pemakaian_id = $1
            ) w
            WHERE p.id = $1 AND w.mulai IS NOT NULL
            RETURNING p.*
            "#,
            &[&izin_pemakaian_id],
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))
    }
}
