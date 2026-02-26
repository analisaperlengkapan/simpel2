-- ============================================================================
-- Migration: Add Indonesian Full-Text Search Support
-- Description: Configure PostgreSQL for Indonesian language full-text search
-- Author: SIMPEL Team
-- Created: 2026-02-10
-- Requirements: REQ-K005
-- ============================================================================

-- ============================================================================
-- SECTION 1: INDONESIAN TEXT SEARCH CONFIGURATION
-- ============================================================================

-- Create Indonesian text search configuration (if not exists)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_ts_config WHERE cfgname = 'indonesian'
    ) THEN
        -- Create Indonesian configuration based on simple (no stemming)
        CREATE TEXT SEARCH CONFIGURATION indonesian (COPY = simple);

        RAISE NOTICE '✅ Created Indonesian text search configuration';
    ELSE
        RAISE NOTICE 'ℹ️  Indonesian text search configuration already exists';
    END IF;
END $$;

-- ============================================================================
-- SECTION 2: INDONESIAN STOP WORDS
-- ============================================================================

-- Create Indonesian stop words dictionary
CREATE TEXT SEARCH DICTIONARY indonesian_stem (
    TEMPLATE = snowball,
    Language = indonesian
);

-- Add Indonesian stop words
-- Common Indonesian words that should be ignored in search
CREATE TEXT SEARCH DICTIONARY indonesian_stopwords (
    TEMPLATE = simple,
    STOPWORDS = indonesian
);

-- Configure the Indonesian text search to use stop words
ALTER TEXT SEARCH CONFIGURATION indonesian
    ALTER MAPPING FOR asciiword, asciihword, hword_asciipart,
                      word, hword, hword_part
    WITH indonesian_stopwords, indonesian_stem;

-- ============================================================================
-- SECTION 3: FULL-TEXT SEARCH INDEXES WITH INDONESIAN SUPPORT
-- ============================================================================

-- Drop existing simple tsvector indexes and recreate with Indonesian

-- Kebutuhan BMN - Full-text search with Indonesian
CREATE INDEX IF NOT EXISTS idx_pkb_nama_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn
USING gin(to_tsvector('indonesian', nama));

CREATE INDEX IF NOT EXISTS idx_pkb_deskripsi_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn
USING gin(to_tsvector('indonesian', COALESCE(deskripsi, '')));

CREATE INDEX IF NOT EXISTS idx_pkb_barang_nama_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
USING gin(to_tsvector('indonesian', nama));

CREATE INDEX IF NOT EXISTS idx_pkb_barang_keterangan_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
USING gin(to_tsvector('indonesian', COALESCE(keterangan, '')));

-- Pakaian Dinas - Full-text search with Indonesian
CREATE INDEX IF NOT EXISTS idx_ppd_nama_fts_indonesian
ON perlengkapan.pengajuan_pakaian_dinas
USING gin(to_tsvector('indonesian', nama));

CREATE INDEX IF NOT EXISTS idx_ppd_jenis_nama_fts_indonesian
ON perlengkapan.ms_jenis_pakaian_dinas
USING gin(to_tsvector('indonesian', nama));

CREATE INDEX IF NOT EXISTS idx_ppd_spesifikasi_nama_fts_indonesian
ON perlengkapan.ms_spesifikasi_pakaian_dinas
USING gin(to_tsvector('indonesian', nama));

-- Roadmap - Full-text search with Indonesian
CREATE INDEX IF NOT EXISTS idx_roadmap_nama_barang_fts_indonesian
ON perlengkapan.roadmap_sarpras
USING gin(to_tsvector('indonesian', nama_barang));

-- Mapping Kodefikasi - Full-text search with Indonesian
CREATE INDEX IF NOT EXISTS idx_mapping_nama_lama_fts_indonesian
ON perlengkapan.mapping_kodefikasi
USING gin(to_tsvector('indonesian', nama_barang_lama));

CREATE INDEX IF NOT EXISTS idx_mapping_nama_baru_fts_indonesian
ON perlengkapan.mapping_kodefikasi
USING gin(to_tsvector('indonesian', COALESCE(nama_barang_baru, '')));

-- ============================================================================
-- SECTION 4: COMBINED SEARCH INDEXES (Multiple fields)
-- ============================================================================

-- Kebutuhan BMN - Combined search on nama + deskripsi
CREATE INDEX IF NOT EXISTS idx_pkb_combined_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn
USING gin(to_tsvector('indonesian', nama || ' ' || COALESCE(deskripsi, '')));

-- Kebutuhan BMN Barang - Combined search on nama + keterangan
CREATE INDEX IF NOT EXISTS idx_pkb_barang_combined_fts_indonesian
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
USING gin(to_tsvector('indonesian', nama || ' ' || COALESCE(keterangan, '')));

-- ============================================================================
-- SECTION 5: ANALYZE TABLES
-- ============================================================================

ANALYZE perlengkapan.pengajuan_kebutuhan_bmn;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang;
ANALYZE perlengkapan.pengajuan_pakaian_dinas;
ANALYZE perlengkapan.ms_jenis_pakaian_dinas;
ANALYZE perlengkapan.ms_spesifikasi_pakaian_dinas;
ANALYZE perlengkapan.roadmap_sarpras;
ANALYZE perlengkapan.mapping_kodefikasi;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '✅ Indonesian full-text search support added successfully';
    RAISE NOTICE '🇮🇩 Text search configuration: indonesian';
    RAISE NOTICE '📚 Stop words: Indonesian common words';
    RAISE NOTICE '🔍 Full-text search indexes created with Indonesian language support';
    RAISE NOTICE '⚡ Search queries can now use Indonesian language features';
    RAISE NOTICE '';
    RAISE NOTICE 'Example usage:';
    RAISE NOTICE '  SELECT * FROM perlengkapan.pengajuan_kebutuhan_bmn';
    RAISE NOTICE '  WHERE to_tsvector(''indonesian'', nama) @@ plainto_tsquery(''indonesian'', ''meja kerja'');';
END $$;
