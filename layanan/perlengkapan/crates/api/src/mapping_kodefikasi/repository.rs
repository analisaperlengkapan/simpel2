//! # Mapping Kodefikasi Repository
//!
//! Database operations for mapping kodefikasi

use crate::errors::AppError;
use crate::mapping_kodefikasi::models::*;
use deadpool_postgres::Pool;
use uuid::Uuid;

/// Repository for mapping kodefikasi operations
pub struct MappingRepository {
    pool: Pool,
}

impl MappingRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Detect non-standard codes from SIMAN data
    pub async fn detect_non_standard_codes(&self) -> Result<Vec<NonStandardCode>, AppError> {
        let query = r#"
            SELECT DISTINCT
                sa.kode_barang as kode_lama,
                sa.nama_barang as nama_lama,
                sa.satker_id,
                COUNT(*) as jumlah_aset
            FROM integrasi.siman_aset_tanah sa
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            WHERE mb.id IS NULL
            GROUP BY sa.kode_barang, sa.nama_barang, sa.satker_id
            ORDER BY jumlah_aset DESC
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let non_standard = rows
            .into_iter()
            .map(|row| NonStandardCode {
                kode_lama: row.get("kode_lama"),
                nama_lama: row.get("nama_lama"),
                satker_id: row.get("satker_id"),
                jumlah_aset: row.get("jumlah_aset"),
                suggested_mapping: None,
            })
            .collect();

        Ok(non_standard)
    }

    /// Suggest mapping using fuzzy matching
    pub async fn suggest_mapping(
        &self,
        nama_lama: &str,
    ) -> Result<Vec<MappingSuggestion>, AppError> {
        let query = r#"
            SELECT
                id,
                kode,
                nama,
                similarity(nama, $1) as score
            FROM perlengkapan.ms_barang
            WHERE similarity(nama, $1) > 0.3
            ORDER BY score DESC
            LIMIT 5
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[&nama_lama]).await?;

        let suggestions = rows
            .into_iter()
            .map(|row| MappingSuggestion {
                barang_id: row.get("id"),
                kode_baru: row.get("kode"),
                nama_baru: row.get("nama"),
                similarity_score: row.get("score"),
            })
            .collect();

        Ok(suggestions)
    }

    /// Create a mapping proposal
    pub async fn create_proposal(
        &self,
        request: &MappingProposalRequest,
    ) -> Result<MappingProposal, AppError> {
        let proposal_id = Uuid::new_v4();

        let query = r#"
            INSERT INTO perlengkapan.mapping_kodefikasi
            (id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
             status_mapping, catatan_mapping, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, 'PROPOSED', $6, NOW(), NOW())
            RETURNING id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
                      status_mapping, catatan_mapping, created_at, updated_at
        "#;

        let client = self.pool.get().await?;
        let row = client
            .query_one(
                query,
                &[
                    &proposal_id,
                    &request.satker_id,
                    &request.kode_lama,
                    &request.nama_lama,
                    &request.kode_baru_id,
                    &request.catatan,
                ],
            )
            .await?;

        Ok(MappingProposal {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang_lama: row.get("kode_barang_lama"),
            nama_barang_lama: row.get("nama_barang_lama"),
            kode_barang_baru_id: row.get("kode_barang_baru_id"),
            status_mapping: row.get("status_mapping"),
            catatan_mapping: row.get("catatan_mapping"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Get all mapping proposals
    pub async fn get_all_proposals(&self) -> Result<Vec<MappingProposal>, AppError> {
        let query = r#"
            SELECT id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
                   status_mapping, catatan_mapping, created_at, updated_at
            FROM perlengkapan.mapping_kodefikasi
            ORDER BY created_at DESC
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let proposals = rows
            .into_iter()
            .map(|row| MappingProposal {
                id: row.get("id"),
                satker_id: row.get("satker_id"),
                kode_barang_lama: row.get("kode_barang_lama"),
                nama_barang_lama: row.get("nama_barang_lama"),
                kode_barang_baru_id: row.get("kode_barang_baru_id"),
                status_mapping: row.get("status_mapping"),
                catatan_mapping: row.get("catatan_mapping"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(proposals)
    }

    /// Get proposal by ID
    pub async fn get_proposal_by_id(&self, id: Uuid) -> Result<MappingProposal, AppError> {
        let query = r#"
            SELECT id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
                   status_mapping, catatan_mapping, created_at, updated_at
            FROM perlengkapan.mapping_kodefikasi
            WHERE id = $1
        "#;

        let client = self.pool.get().await?;
        let row = client.query_one(query, &[&id]).await?;

        Ok(MappingProposal {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang_lama: row.get("kode_barang_lama"),
            nama_barang_lama: row.get("nama_barang_lama"),
            kode_barang_baru_id: row.get("kode_barang_baru_id"),
            status_mapping: row.get("status_mapping"),
            catatan_mapping: row.get("catatan_mapping"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Verify a mapping proposal
    pub async fn verify_proposal(
        &self,
        proposal_id: Uuid,
        approved: bool,
        catatan_verifikasi: Option<String>,
    ) -> Result<MappingProposal, AppError> {
        let new_status = if approved { "VERIFIED" } else { "REJECTED" };

        let query = r#"
            UPDATE perlengkapan.mapping_kodefikasi
            SET status_mapping = $1,
                catatan_mapping = COALESCE($2, catatan_mapping),
                updated_at = NOW()
            WHERE id = $3
            RETURNING id, satker_id, kode_barang_lama, nama_barang_lama, kode_barang_baru_id,
                      status_mapping, catatan_mapping, created_at, updated_at
        "#;

        let client = self.pool.get().await?;
        let row = client
            .query_one(query, &[&new_status, &catatan_verifikasi, &proposal_id])
            .await?;

        Ok(MappingProposal {
            id: row.get("id"),
            satker_id: row.get("satker_id"),
            kode_barang_lama: row.get("kode_barang_lama"),
            nama_barang_lama: row.get("nama_barang_lama"),
            kode_barang_baru_id: row.get("kode_barang_baru_id"),
            status_mapping: row.get("status_mapping"),
            catatan_mapping: row.get("catatan_mapping"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    /// Apply verified mapping to SIMAN assets
    pub async fn apply_mapping(&self, proposal_id: Uuid) -> Result<u64, AppError> {
        let query = r#"
            UPDATE integrasi.siman_aset_tanah sa
            SET kode_barang = mb.kode
            FROM perlengkapan.mapping_kodefikasi mk
            JOIN perlengkapan.ms_barang mb ON mk.kode_barang_baru_id = mb.id
            WHERE mk.id = $1
              AND sa.kode_barang = mk.kode_barang_lama
              AND sa.satker_id = mk.satker_id
        "#;

        let client = self.pool.get().await?;
        let rows_affected = client.execute(query, &[&proposal_id]).await?;

        Ok(rows_affected)
    }

    /// Get mapping progress statistics
    pub async fn get_mapping_progress(&self) -> Result<MappingProgress, AppError> {
        let query = r#"
            WITH non_standard AS (
                SELECT COUNT(DISTINCT sa.kode_barang) as total
                FROM integrasi.siman_aset_tanah sa
                LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
                WHERE mb.id IS NULL
            ),
            mapped AS (
                SELECT COUNT(*) as total
                FROM perlengkapan.mapping_kodefikasi
                WHERE status_mapping = 'VERIFIED'
            ),
            pending AS (
                SELECT COUNT(*) as total
                FROM perlengkapan.mapping_kodefikasi
                WHERE status_mapping = 'PROPOSED'
            )
            SELECT
                (SELECT total FROM non_standard) as total_non_standard,
                (SELECT total FROM mapped) as total_mapped,
                (SELECT total FROM pending) as pending_verification
        "#;

        let client = self.pool.get().await?;
        let row = client.query_one(query, &[]).await?;

        let total_non_standard: i64 = row.get("total_non_standard");
        let total_mapped: i64 = row.get("total_mapped");
        let pending_verification: i64 = row.get("pending_verification");

        let mapping_percentage = if total_non_standard > 0 {
            (total_mapped as f64 / total_non_standard as f64) * 100.0
        } else {
            0.0
        };

        // Get non-standard codes with status
        let codes_query = r#"
            SELECT DISTINCT
                sa.kode_barang as kode_lama,
                sa.nama_barang as nama_lama,
                sa.satker_id,
                s.nama as satker_nama,
                COUNT(*) as jumlah_aset,
                mk.status_mapping,
                mb.kode as kode_baru,
                mb.nama as nama_baru
            FROM integrasi.siman_aset_tanah sa
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            LEFT JOIN perlengkapan.mapping_kodefikasi mk ON sa.kode_barang = mk.kode_barang_lama
                AND sa.satker_id = mk.satker_id
            LEFT JOIN authenc.satkers s ON sa.satker_id = s.id
            WHERE mb.id IS NULL
            GROUP BY sa.kode_barang, sa.nama_barang, sa.satker_id, s.nama,
                     mk.status_mapping, mb.kode, mb.nama
            ORDER BY jumlah_aset DESC
            LIMIT 100
        "#;

        let rows = client.query(codes_query, &[]).await?;

        let non_standard_codes = rows
            .into_iter()
            .map(|row| NonStandardCodeWithStatus {
                kode_lama: row.get("kode_lama"),
                nama_lama: row.get("nama_lama"),
                satker_id: row.get("satker_id"),
                satker_nama: row.get("satker_nama"),
                jumlah_aset: row.get("jumlah_aset"),
                status_mapping: row.get("status_mapping"),
                kode_baru: row.get("kode_baru"),
                nama_baru: row.get("nama_baru"),
            })
            .collect();

        Ok(MappingProgress {
            total_non_standard,
            total_mapped,
            pending_verification,
            mapping_percentage,
            non_standard_codes,
        })
    }

    /// Get mapping progress by satker
    pub async fn get_mapping_progress_by_satker(
        &self,
    ) -> Result<Vec<MappingProgressBySatker>, AppError> {
        let query = r#"
            SELECT
                s.id as satker_id,
                s.nama as satker_nama,
                COUNT(DISTINCT CASE WHEN mb.id IS NULL THEN sa.kode_barang END) as total_non_standard,
                COUNT(DISTINCT CASE WHEN mk.status_mapping = 'VERIFIED' THEN mk.id END) as total_mapped,
                COUNT(DISTINCT CASE WHEN mk.status_mapping = 'PROPOSED' THEN mk.id END) as pending_verification
            FROM authenc.satkers s
            LEFT JOIN integrasi.siman_aset_tanah sa ON s.id = sa.satker_id
            LEFT JOIN perlengkapan.ms_barang mb ON sa.kode_barang = mb.kode
            LEFT JOIN perlengkapan.mapping_kodefikasi mk ON sa.kode_barang = mk.kode_barang_lama
                AND sa.satker_id = mk.satker_id
            GROUP BY s.id, s.nama
            HAVING COUNT(DISTINCT CASE WHEN mb.id IS NULL THEN sa.kode_barang END) > 0
            ORDER BY total_non_standard DESC
        "#;

        let client = self.pool.get().await?;
        let rows = client.query(query, &[]).await?;

        let progress = rows
            .into_iter()
            .map(|row| {
                let total_non_standard: i64 = row.get("total_non_standard");
                let total_mapped: i64 = row.get("total_mapped");
                let mapping_percentage = if total_non_standard > 0 {
                    (total_mapped as f64 / total_non_standard as f64) * 100.0
                } else {
                    0.0
                };

                MappingProgressBySatker {
                    satker_id: row.get("satker_id"),
                    satker_nama: row.get("satker_nama"),
                    total_non_standard,
                    total_mapped,
                    pending_verification: row.get("pending_verification"),
                    mapping_percentage,
                }
            })
            .collect();

        Ok(progress)
    }
}
