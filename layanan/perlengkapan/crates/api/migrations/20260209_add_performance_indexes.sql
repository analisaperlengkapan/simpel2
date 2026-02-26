-- ============================================================================
-- Migration: Add Performance Indexes
-- Description: Comprehensive indexing strategy for optimal query performance
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- Requirements: NFR-P001, NFR-P002
-- ============================================================================

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS pg_trgm;  -- For full-text search
CREATE EXTENSION IF NOT EXISTS btree_gin;  -- For composite GIN indexes

-- ============================================================================
-- SECTION 1: FOREIGN KEY INDEXES (Critical for JOIN performance)
-- ============================================================================

-- Kebutuhan BMN Foreign Key Indexes
CREATE INDEX IF NOT EXISTS idx_pkb_asset_pengajuan_fk
ON perlengkapan.pengajuan_kebutuhan_bmn_asset(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_pkb_satker_pengajuan_fk
ON perlengkapan.pengajuan_kebutuhan_bmn_satker(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_satker_fk
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(pengajuan_satker_id);

CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_satker_fk
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(pengajuan_satker_id);

-- Pakaian Dinas Foreign Key Indexes
CREATE INDEX IF NOT EXISTS idx_ppd_spesifikasi_jenis_fk
ON perlengkapan.ms_spesifikasi_pakaian_dinas(jenis_pakaian_dinas_id);

CREATE INDEX IF NOT EXISTS idx_ppd_subspesifikasi_spec_fk
ON perlengkapan.ms_subspesifikasi_pakaian_dinas(spesifikasi_id);

CREATE INDEX IF NOT EXISTS idx_ppd_foto_spec_fk
ON perlengkapan.ms_spesifikasi_pakaian_dinas_foto(spesifikasi_id);

CREATE INDEX IF NOT EXISTS idx_ppd_satker_terpilih_pengajuan_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_terpilih(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_ppd_pakaian_pengajuan_fk
ON perlengkapan.pengajuan_pakaian_dinas_pakaian(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_ppd_satker_pengajuan_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_ppd_pegawai_satker_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai(pengajuan_satker_id);

CREATE INDEX IF NOT EXISTS idx_ppd_ukuran_satker_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran(pengajuan_satker_id);

CREATE INDEX IF NOT EXISTS idx_ppd_ukuran_pegawai_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran(pegawai_id);

CREATE INDEX IF NOT EXISTS idx_ppd_ukuran_pakaian_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran(pakaian_id);

CREATE INDEX IF NOT EXISTS idx_ppd_aktivitas_satker_fk
ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas(pengajuan_satker_id);

-- ============================================================================
-- SECTION 2: FULL-TEXT SEARCH INDEXES (pg_trgm for fuzzy search)
-- ============================================================================

-- Kebutuhan BMN Full-Text Search
CREATE INDEX IF NOT EXISTS idx_pkb_nama_trgm
ON perlengkapan.pengajuan_kebutuhan_bmn USING gin(nama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_pkb_deskripsi_trgm
ON perlengkapan.pengajuan_kebutuhan_bmn USING gin(deskripsi gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_pkb_satker_nama_trgm
ON perlengkapan.pengajuan_kebutuhan_bmn_satker USING gin(nm_satker gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_nama_trgm
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin(nama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_keterangan_trgm
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin(keterangan gin_trgm_ops);

-- Pakaian Dinas Full-Text Search
CREATE INDEX IF NOT EXISTS idx_ppd_nama_trgm
ON perlengkapan.pengajuan_pakaian_dinas USING gin(nama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_ppd_jenis_nama_trgm
ON perlengkapan.ms_jenis_pakaian_dinas USING gin(nama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_ppd_spesifikasi_nama_trgm
ON perlengkapan.ms_spesifikasi_pakaian_dinas USING gin(nama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_ppd_pegawai_nama_trgm
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai USING gin(nama gin_trgm_ops);

-- Roadmap Full-Text Search
CREATE INDEX IF NOT EXISTS idx_roadmap_nama_barang_trgm
ON perlengkapan.roadmap_sarpras USING gin(nama_barang gin_trgm_ops);

-- Mapping Kodefikasi Full-Text Search
CREATE INDEX IF NOT EXISTS idx_mapping_nama_lama_trgm
ON perlengkapan.mapping_kodefikasi USING gin(nama_barang_lama gin_trgm_ops);

CREATE INDEX IF NOT EXISTS idx_mapping_nama_baru_trgm
ON perlengkapan.mapping_kodefikasi USING gin(nama_barang_baru gin_trgm_ops);

-- ============================================================================
-- SECTION 3: COMPOSITE INDEXES (For common query patterns)
-- ============================================================================

-- Kebutuhan BMN Composite Indexes
CREATE INDEX IF NOT EXISTS idx_pkb_tahun_status
ON perlengkapan.pengajuan_kebutuhan_bmn(tahun, status_kode);

CREATE INDEX IF NOT EXISTS idx_pkb_tahun_created
ON perlengkapan.pengajuan_kebutuhan_bmn(tahun, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_pkb_status_created
ON perlengkapan.pengajuan_kebutuhan_bmn(status_kode, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_pkb_satker_pengajuan_status
ON perlengkapan.pengajuan_kebutuhan_bmn_satker(pengajuan_id, status_kode);

CREATE INDEX IF NOT EXISTS idx_pkb_satker_id_status
ON perlengkapan.pengajuan_kebutuhan_bmn_satker(ms_satker_id, status_kode);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_kode_jumlah
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(kode_barang, jumlah);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_prioritas_skor
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(prioritas DESC, skor DESC);

-- Pakaian Dinas Composite Indexes
CREATE INDEX IF NOT EXISTS idx_ppd_tahun_status
ON perlengkapan.pengajuan_pakaian_dinas(tahun, status_kode);

CREATE INDEX IF NOT EXISTS idx_ppd_tahun_created
ON perlengkapan.pengajuan_pakaian_dinas(tahun, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_ppd_satker_pengajuan_status
ON perlengkapan.pengajuan_pakaian_dinas_satker(pengajuan_id, status_kode);

CREATE INDEX IF NOT EXISTS idx_ppd_satker_id_status
ON perlengkapan.pengajuan_pakaian_dinas_satker(satker_id, status_kode);

CREATE INDEX IF NOT EXISTS idx_ppd_pegawai_nip_jk
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai(nip, jenis_kelamin);

-- Roadmap Composite Indexes
CREATE INDEX IF NOT EXISTS idx_roadmap_satker_tahun_status
ON perlengkapan.roadmap_sarpras(satker_id, tahun_rencana, status_pemenuhan);

CREATE INDEX IF NOT EXISTS idx_roadmap_periode_tahun
ON perlengkapan.roadmap_sarpras(periode_mulai, periode_akhir, tahun_rencana);

CREATE INDEX IF NOT EXISTS idx_roadmap_kode_status
ON perlengkapan.roadmap_sarpras(kode_barang, status_pemenuhan);

-- Riwayat Pemenuhan Composite Indexes
CREATE INDEX IF NOT EXISTS idx_riwayat_satker_tahun_kode
ON perlengkapan.riwayat_pemenuhan(satker_id, tahun_anggaran, kode_barang);

CREATE INDEX IF NOT EXISTS idx_riwayat_tahun_sumber
ON perlengkapan.riwayat_pemenuhan(tahun_anggaran, sumber_data);

CREATE INDEX IF NOT EXISTS idx_riwayat_tanggal_sumber
ON perlengkapan.riwayat_pemenuhan(tanggal_pemenuhan DESC, sumber_data);

-- Izin Pemakaian Composite Indexes
CREATE INDEX IF NOT EXISTS idx_izin_jenis_status
ON perlengkapan.izin_pemakaian_bmn(jenis_bmn, status);

CREATE INDEX IF NOT EXISTS idx_izin_pegawai_jenis_status
ON perlengkapan.izin_pemakaian_bmn(pegawai_nip, jenis_bmn, status);

CREATE INDEX IF NOT EXISTS idx_izin_satker_status
ON perlengkapan.izin_pemakaian_bmn(pegawai_satker_id, status);

CREATE INDEX IF NOT EXISTS idx_izin_nup_jenis
ON perlengkapan.izin_pemakaian_bmn(bmn_nup, jenis_bmn);

CREATE INDEX IF NOT EXISTS idx_izin_status_tanggal_selesai
ON perlengkapan.izin_pemakaian_bmn(status, tanggal_selesai)
WHERE status IN ('ACTIVE', 'APPROVED');

-- Parallel Approvals Composite Indexes
CREATE INDEX IF NOT EXISTS idx_parallel_entity_status
ON perlengkapan.parallel_approvals(entity_type, entity_id, status);

CREATE INDEX IF NOT EXISTS idx_parallel_status_expires
ON perlengkapan.parallel_approvals(status, expires_at)
WHERE status = 'PENDING';

-- ============================================================================
-- SECTION 4: JSONB GIN INDEXES (For JSON queries)
-- ============================================================================

-- Kebutuhan BMN JSONB Indexes
CREATE INDEX IF NOT EXISTS idx_pkb_id_jenis_asset_gin
ON perlengkapan.pengajuan_kebutuhan_bmn USING gin(id_jenis_asset);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_file_pendukung_gin
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang USING gin(file_pendukung);

-- Riwayat Pemenuhan JSONB Indexes
CREATE INDEX IF NOT EXISTS idx_riwayat_file_dokumen_gin
ON perlengkapan.riwayat_pemenuhan USING gin(file_dokumen);

-- Izin Pemakaian JSONB Indexes
CREATE INDEX IF NOT EXISTS idx_izin_file_pendukung_gin
ON perlengkapan.izin_pemakaian_bmn USING gin(file_pendukung);

-- ============================================================================
-- SECTION 5: PARTIAL INDEXES (For specific query patterns)
-- ============================================================================

-- Active/Pending Records Only
CREATE INDEX IF NOT EXISTS idx_pkb_active_status
ON perlengkapan.pengajuan_kebutuhan_bmn(status_kode, created_at DESC)
WHERE status_kode NOT IN (2006, 2007, 2008, 2009);

CREATE INDEX IF NOT EXISTS idx_ppd_active_status
ON perlengkapan.pengajuan_pakaian_dinas(status_kode, created_at DESC)
WHERE status_kode NOT IN (2006, 2007, 2008, 2009);

CREATE INDEX IF NOT EXISTS idx_roadmap_active
ON perlengkapan.roadmap_sarpras(satker_id, tahun_rencana)
WHERE status_pemenuhan IN ('PLANNED', 'IN_PROGRESS');

CREATE INDEX IF NOT EXISTS idx_izin_active_expiring
ON perlengkapan.izin_pemakaian_bmn(tanggal_selesai, pegawai_satker_id)
WHERE status = 'ACTIVE' AND tanggal_selesai < CURRENT_DATE + INTERVAL '60 days';

CREATE INDEX IF NOT EXISTS idx_mapping_pending
ON perlengkapan.mapping_kodefikasi(satker_id, created_at DESC)
WHERE status_mapping IN ('PROPOSED', 'VERIFIED');

-- High Priority Items
CREATE INDEX IF NOT EXISTS idx_pkb_barang_high_priority
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(prioritas DESC, skor DESC)
WHERE prioritas >= 7;

CREATE INDEX IF NOT EXISTS idx_roadmap_high_priority
ON perlengkapan.roadmap_sarpras(prioritas DESC, tahun_rencana)
WHERE prioritas >= 7;

-- Recent Activity (Last 30 days)
CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_recent
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(created_at DESC)
WHERE created_at > CURRENT_DATE - INTERVAL '30 days';

CREATE INDEX IF NOT EXISTS idx_ppd_aktivitas_recent
ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas(created_at DESC)
WHERE created_at > CURRENT_DATE - INTERVAL '30 days';

CREATE INDEX IF NOT EXISTS idx_izin_aktivitas_recent
ON perlengkapan.izin_pemakaian_bmn_aktivitas(created_at DESC)
WHERE created_at > CURRENT_DATE - INTERVAL '30 days';

-- ============================================================================
-- SECTION 6: COVERING INDEXES (Include frequently accessed columns)
-- ============================================================================

-- Kebutuhan BMN Covering Indexes
CREATE INDEX IF NOT EXISTS idx_pkb_tahun_status_covering
ON perlengkapan.pengajuan_kebutuhan_bmn(tahun, status_kode)
INCLUDE (nama, created_at, updated_at);

CREATE INDEX IF NOT EXISTS idx_pkb_satker_covering
ON perlengkapan.pengajuan_kebutuhan_bmn_satker(ms_satker_id, status_kode)
INCLUDE (nm_satker, prioritas, created_at);

CREATE INDEX IF NOT EXISTS idx_pkb_barang_covering
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(kode_barang)
INCLUDE (nama, jumlah, jml_setuju, prioritas, skor);

-- Pakaian Dinas Covering Indexes
CREATE INDEX IF NOT EXISTS idx_ppd_tahun_covering
ON perlengkapan.pengajuan_pakaian_dinas(tahun, status_kode)
INCLUDE (nama, created_at);

CREATE INDEX IF NOT EXISTS idx_ppd_pegawai_covering
ON perlengkapan.pengajuan_pakaian_dinas_satker_pegawai(nip)
INCLUDE (nama, jenis_kelamin, jabatan);

-- Izin Pemakaian Covering Indexes
CREATE INDEX IF NOT EXISTS idx_izin_nup_covering
ON perlengkapan.izin_pemakaian_bmn(bmn_nup)
INCLUDE (nomor_izin, jenis_bmn, status, pegawai_nip, tanggal_selesai);

CREATE INDEX IF NOT EXISTS idx_izin_pegawai_covering
ON perlengkapan.izin_pemakaian_bmn(pegawai_nip, status)
INCLUDE (nomor_izin, jenis_bmn, bmn_nama_barang, tanggal_mulai, tanggal_selesai);

-- ============================================================================
-- SECTION 7: BTREE INDEXES FOR SORTING AND RANGE QUERIES
-- ============================================================================

-- Date Range Queries
CREATE INDEX IF NOT EXISTS idx_pkb_created_at_desc
ON perlengkapan.pengajuan_kebutuhan_bmn(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_pkb_updated_at_desc
ON perlengkapan.pengajuan_kebutuhan_bmn(updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_ppd_created_at_desc
ON perlengkapan.pengajuan_pakaian_dinas(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_roadmap_created_at_desc
ON perlengkapan.roadmap_sarpras(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_riwayat_tanggal_pemenuhan_desc
ON perlengkapan.riwayat_pemenuhan(tanggal_pemenuhan DESC);

CREATE INDEX IF NOT EXISTS idx_izin_tanggal_mulai
ON perlengkapan.izin_pemakaian_bmn(tanggal_mulai DESC);

CREATE INDEX IF NOT EXISTS idx_izin_tanggal_selesai_asc
ON perlengkapan.izin_pemakaian_bmn(tanggal_selesai ASC)
WHERE status = 'ACTIVE';

-- Numeric Range Queries
CREATE INDEX IF NOT EXISTS idx_pkb_barang_jumlah
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(jumlah DESC);

CREATE INDEX IF NOT EXISTS idx_roadmap_jumlah_kebutuhan
ON perlengkapan.roadmap_sarpras(jumlah_kebutuhan DESC);

CREATE INDEX IF NOT EXISTS idx_riwayat_nilai_perolehan
ON perlengkapan.riwayat_pemenuhan(nilai_perolehan DESC)
WHERE nilai_perolehan IS NOT NULL;

-- ============================================================================
-- SECTION 8: ANALYZE TABLES FOR QUERY PLANNER
-- ============================================================================

-- Update statistics for query planner optimization
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_asset;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas;

ANALYZE perlengkapan.ms_jenis_pakaian_dinas;
ANALYZE perlengkapan.ms_spesifikasi_pakaian_dinas;
ANALYZE perlengkapan.pengajuan_pakaian_dinas;
ANALYZE perlengkapan.pengajuan_pakaian_dinas_satker;
ANALYZE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai;

ANALYZE perlengkapan.roadmap_sarpras;
ANALYZE perlengkapan.mapping_kodefikasi;
ANALYZE perlengkapan.riwayat_pemenuhan;
ANALYZE perlengkapan.parallel_approvals;
ANALYZE perlengkapan.parallel_approval_votes;
ANALYZE perlengkapan.izin_pemakaian_bmn;
ANALYZE perlengkapan.izin_pemakaian_bmn_aktivitas;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $
DECLARE
    index_count INTEGER;
BEGIN
    -- Count total indexes in perlengkapan schema
    SELECT COUNT(*) INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'perlengkapan';

    RAISE NOTICE '✅ Performance indexes added successfully';
    RAISE NOTICE '📊 Total indexes in perlengkapan schema: %', index_count;
    RAISE NOTICE '📈 Index types added:';
    RAISE NOTICE '  - Foreign Key indexes (critical for JOINs)';
    RAISE NOTICE '  - Full-text search indexes (pg_trgm for fuzzy search)';
    RAISE NOTICE '  - Composite indexes (common query patterns)';
    RAISE NOTICE '  - JSONB GIN indexes (JSON queries)';
    RAISE NOTICE '  - Partial indexes (specific conditions)';
    RAISE NOTICE '  - Covering indexes (include columns)';
    RAISE NOTICE '  - BTREE indexes (sorting and ranges)';
    RAISE NOTICE '🔍 Schema: perlengkapan';
    RAISE NOTICE '⚡ Query performance should be significantly improved';
    RAISE NOTICE '📝 Run EXPLAIN ANALYZE on slow queries to verify index usage';
END $;
