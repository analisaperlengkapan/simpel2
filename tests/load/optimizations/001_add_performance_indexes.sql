-- Performance Optimization: Add Missing Indexes
-- Based on load test analysis
-- Requirements: NFR-P001, NFR-P002

-- ============================================
-- Foreign Key Indexes
-- ============================================

-- Kebutuhan BMN indexes
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_satker
ON perlengkapan.kebutuhan_bmn(satker_id);

CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_tahun
ON perlengkapan.kebutuhan_bmn(tahun_anggaran);

CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_status
ON perlengkapan.kebutuhan_bmn(status);

CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_kode
ON perlengkapan.kebutuhan_bmn(kode_barang);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_satker_tahun
ON perlengkapan.kebutuhan_bmn(satker_id, tahun_anggaran);

CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_tahun_status
ON perlengkapan.kebutuhan_bmn(tahun_anggaran, status);

-- Pakaian Dinas indexes
CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_tahun
ON perlengkapan.kebutuhan_pakaian_dinas(tahun_anggaran);

CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_nip
ON perlengkapan.kebutuhan_pakaian_dinas(pegawai_nip);

-- Roadmap indexes
CREATE INDEX IF NOT EXISTS idx_roadmap_satker
ON perlengkapan.roadmap_sarpras(satker_id);

CREATE INDEX IF NOT EXISTS idx_roadmap_tahun
ON perlengkapan.roadmap_sarpras(tahun_rencana);

CREATE INDEX IF NOT EXISTS idx_roadmap_periode
ON perlengkapan.roadmap_sarpras(periode_mulai, periode_akhir);

-- Workflow activity indexes
CREATE INDEX IF NOT EXISTS idx_aktivitas_pengajuan
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_aktivitas_user
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(user_id);

CREATE INDEX IF NOT EXISTS idx_aktivitas_created
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(created_at DESC);

-- ============================================
-- Full-Text Search Indexes
-- ============================================

-- Enable pg_trgm extension for fuzzy search
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Full-text search on nama_barang
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_nama_trgm
ON perlengkapan.kebutuhan_bmn
USING GIN(nama_barang gin_trgm_ops);

-- Full-text search with Indonesian language
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_nama_fts
ON perlengkapan.kebutuhan_bmn
USING GIN(to_tsvector('indonesian', nama_barang));

-- Kode barang search
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_kode_trgm
ON perlengkapan.kebutuhan_bmn
USING GIN(kode_barang gin_trgm_ops);

-- ============================================
-- JSONB Indexes (if applicable)
-- ============================================

-- If there are JSONB columns, add GIN indexes
-- Example:
-- CREATE INDEX IF NOT EXISTS idx_kebutuhan_metadata
-- ON perlengkapan.kebutuhan_bmn
-- USING GIN(metadata);

-- ============================================
-- Partial Indexes for Common Filters
-- ============================================

-- Active kebutuhan only
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_active
ON perlengkapan.kebutuhan_bmn(tahun_anggaran, satker_id)
WHERE status NOT IN ('CANCELLED', 'ARCHIVED');

-- Pending approval
CREATE INDEX IF NOT EXISTS idx_kebutuhan_bmn_pending
ON perlengkapan.kebutuhan_bmn(created_at DESC)
WHERE status IN ('SUBMITTED', 'REVIEWED');

-- ============================================
-- Analyze Tables
-- ============================================

-- Update statistics for query planner
ANALYZE perlengkapan.kebutuhan_bmn;
ANALYZE perlengkapan.kebutuhan_pakaian_dinas;
ANALYZE perlengkapan.roadmap_sarpras;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas;

-- ============================================
-- Verification
-- ============================================

-- Check index sizes
SELECT
    schemaname,
    tablename,
    indexname,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_indexes
JOIN pg_class ON pg_indexes.indexname = pg_class.relname
WHERE schemaname = 'perlengkapan'
ORDER BY pg_relation_size(indexrelid) DESC;

-- Check index usage
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as index_scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched
FROM pg_stat_user_indexes
WHERE schemaname = 'perlengkapan'
ORDER BY idx_scan DESC;
