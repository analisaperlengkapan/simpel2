-- =====================================================
-- Migration: SIMAN Helper Functions & Stored Procedures
-- Description: Functions untuk manage sync, upsert, cleanup, dan query
-- Version: 3.0 (Updated for Unified Table)
-- Created: 2025-11-06
-- =====================================================

SET search_path TO integrasi, public;

-- =====================================================
-- FUNCTION: start_sync
-- Purpose: Mulai tracking sinkronisasi untuk kategori tertentu
-- =====================================================
CREATE OR REPLACE FUNCTION start_sync(
    p_kategori_aset VARCHAR(100),
    p_ba_key VARCHAR(50) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_sync_id UUID;
BEGIN
    INSERT INTO siman_sync_log (
        kategori_aset,
        ba_key,
        sync_start_time,
        status
    ) VALUES (
        p_kategori_aset,
        p_ba_key,
        CURRENT_TIMESTAMP,
        'running'
    )
    RETURNING id INTO v_sync_id;

    RETURN v_sync_id;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: complete_sync
-- Purpose: Mark sync sebagai completed dengan statistik
-- =====================================================
CREATE OR REPLACE FUNCTION complete_sync(
    p_sync_id UUID,
    p_total_records INTEGER DEFAULT 0,
    p_success_records INTEGER DEFAULT 0,
    p_failed_records INTEGER DEFAULT 0
)
RETURNS VOID AS $$
BEGIN
    UPDATE siman_sync_log
    SET
        sync_end_time = CURRENT_TIMESTAMP,
        total_records = p_total_records,
        success_records = p_success_records,
        failed_records = p_failed_records,
        status = 'completed',
        updated_at = CURRENT_TIMESTAMP
    WHERE id = p_sync_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Sync log dengan ID % tidak ditemukan', p_sync_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: fail_sync
-- Purpose: Mark sync sebagai failed dengan error message
-- =====================================================
CREATE OR REPLACE FUNCTION fail_sync(
    p_sync_id UUID,
    p_error_message TEXT
)
RETURNS VOID AS $$
BEGIN
    UPDATE siman_sync_log
    SET
        sync_end_time = CURRENT_TIMESTAMP,
        status = 'failed',
        error_message = p_error_message,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = p_sync_id;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Sync log dengan ID % tidak ditemukan', p_sync_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: get_latest_sync_status
-- Purpose: Get status sync terakhir untuk kategori tertentu
-- =====================================================
CREATE OR REPLACE FUNCTION get_latest_sync_status(
    p_kategori_aset VARCHAR(100) DEFAULT NULL
)
RETURNS TABLE (
    sync_id UUID,
    kategori_aset VARCHAR(100),
    status VARCHAR(20),
    sync_start_time TIMESTAMP,
    sync_end_time TIMESTAMP,
    total_records INTEGER,
    success_records INTEGER,
    failed_records INTEGER,
    error_message TEXT,
    duration_seconds INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        sl.id,
        sl.kategori_aset,
        sl.status,
        sl.sync_start_time,
        sl.sync_end_time,
        sl.total_records,
        sl.success_records,
        sl.failed_records,
        sl.error_message,
        EXTRACT(EPOCH FROM (sl.sync_end_time - sl.sync_start_time))::INTEGER as duration_seconds
    FROM siman_sync_log sl
    WHERE p_kategori_aset IS NULL OR sl.kategori_aset = p_kategori_aset
    ORDER BY sl.sync_start_time DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: upsert_siman_aset
-- Purpose: Insert or update aset data (simplified untuk unified table)
-- =====================================================
CREATE OR REPLACE FUNCTION upsert_siman_aset(
    p_kategori_aset VARCHAR(50),
    p_no_aset BIGINT,
    p_kode_satker VARCHAR(20),
    p_aset_data JSONB,
    p_sync_id UUID
)
RETURNS UUID AS $$
DECLARE
    v_aset_id UUID;
BEGIN
    INSERT INTO siman_aset (
        kategori_aset,
        no_aset,
        kode_satker,
        kd_jns_bmn,
        kd_brg,
        tercatat,
        merk,
        tipe,
        rph_aset,
        rph_susut,
        rph_mutasi,
        nama_satker,
        alamat,
        ur_kondisi,
        raw_data,
        sync_id
    ) VALUES (
        p_kategori_aset,
        p_no_aset,
        p_kode_satker,
        (p_aset_data->>'kd_jns_bmn')::INTEGER,
        p_aset_data->>'kd_brg',
        p_aset_data->>'tercatat',
        p_aset_data->>'merk',
        p_aset_data->>'tipe',
        (p_aset_data->>'rph_aset')::BIGINT,
        (p_aset_data->>'rph_susut')::BIGINT,
        (p_aset_data->>'rph_mutasi')::BIGINT,
        p_aset_data->>'nama_satker',
        p_aset_data->>'alamat',
        p_aset_data->>'ur_kondisi',
        p_aset_data,
        p_sync_id
    )
    ON CONFLICT (no_aset, kode_satker, kategori_aset)
    DO UPDATE SET
        kd_jns_bmn = EXCLUDED.kd_jns_bmn,
        kd_brg = EXCLUDED.kd_brg,
        tercatat = EXCLUDED.tercatat,
        merk = EXCLUDED.merk,
        tipe = EXCLUDED.tipe,
        rph_aset = EXCLUDED.rph_aset,
        rph_susut = EXCLUDED.rph_susut,
        rph_mutasi = EXCLUDED.rph_mutasi,
        nama_satker = EXCLUDED.nama_satker,
        alamat = EXCLUDED.alamat,
        ur_kondisi = EXCLUDED.ur_kondisi,
        raw_data = EXCLUDED.raw_data,
        sync_id = EXCLUDED.sync_id,
        updated_at = CURRENT_TIMESTAMP
    RETURNING id INTO v_aset_id;

    RETURN v_aset_id;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: cleanup_old_sync_logs
-- Purpose: Hapus sync logs yang lebih lama dari N hari
-- =====================================================
CREATE OR REPLACE FUNCTION cleanup_old_sync_logs(
    p_retention_days INTEGER DEFAULT 90
)
RETURNS INTEGER AS $$
DECLARE
    v_deleted_count INTEGER;
BEGIN
    DELETE FROM siman_sync_log
    WHERE created_at < CURRENT_TIMESTAMP - INTERVAL '1 day' * p_retention_days
    AND status = 'completed';

    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;

    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: get_aset_summary_by_satker
-- Purpose: Get summary aset untuk satker tertentu
-- =====================================================
CREATE OR REPLACE FUNCTION get_aset_summary_by_satker(
    p_kode_satker VARCHAR(20)
)
RETURNS TABLE (
    kategori_aset VARCHAR(50),
    total_aset BIGINT,
    total_nilai_perolehan NUMERIC,
    total_nilai_buku NUMERIC,
    aset_baik BIGINT,
    aset_rusak BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        sa.kategori_aset,
        COUNT(*) as total_aset,
        SUM(sa.rph_aset) as total_nilai_perolehan,
        SUM(sa.rph_aset - COALESCE(sa.rph_susut, 0)) as total_nilai_buku,
        COUNT(*) FILTER (WHERE sa.ur_kondisi = 'Baik') as aset_baik,
        COUNT(*) FILTER (WHERE sa.ur_kondisi LIKE 'Rusak%') as aset_rusak
    FROM siman_aset sa
    WHERE sa.kode_satker = p_kode_satker
    GROUP BY sa.kategori_aset
    ORDER BY total_aset DESC;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: search_aset
-- Purpose: Full-text search untuk aset
-- =====================================================
CREATE OR REPLACE FUNCTION search_aset(
    p_search_query TEXT,
    p_kategori_aset VARCHAR(50) DEFAULT NULL,
    p_limit INTEGER DEFAULT 100
)
RETURNS TABLE (
    id UUID,
    kategori_aset VARCHAR(50),
    no_aset BIGINT,
    nama TEXT,
    merk VARCHAR(500),
    tipe VARCHAR(500),
    kode_satker VARCHAR(20),
    nama_satker VARCHAR(100),
    alamat VARCHAR(200),
    ur_kondisi VARCHAR(50),
    rph_aset BIGINT,
    rank REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        sa.id,
        sa.kategori_aset,
        sa.no_aset,
        sa.nama,
        sa.merk,
        sa.tipe,
        sa.kode_satker,
        sa.nama_satker,
        sa.alamat,
        sa.ur_kondisi,
        sa.rph_aset,
        ts_rank(
            to_tsvector('indonesian',
                COALESCE(sa.alamat, '') || ' ' ||
                COALESCE(sa.nama, '') || ' ' ||
                COALESCE(sa.nama_satker, '')
            ),
            plainto_tsquery('indonesian', p_search_query)
        ) as rank
    FROM siman_aset sa
    WHERE
        to_tsvector('indonesian',
            COALESCE(sa.alamat, '') || ' ' ||
            COALESCE(sa.nama, '') || ' ' ||
            COALESCE(sa.nama_satker, '')
        ) @@ plainto_tsquery('indonesian', p_search_query)
        AND (p_kategori_aset IS NULL OR sa.kategori_aset = p_kategori_aset)
    ORDER BY rank DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- FUNCTION: get_aset_by_condition
-- Purpose: Get aset berdasarkan kondisi tertentu
-- =====================================================
CREATE OR REPLACE FUNCTION get_aset_by_condition(
    p_kondisi VARCHAR(50)
)
RETURNS TABLE (
    id UUID,
    kategori_aset VARCHAR(50),
    no_aset BIGINT,
    nama TEXT,
    kode_satker VARCHAR(20),
    nama_satker VARCHAR(100),
    rph_aset BIGINT,
    ur_kondisi VARCHAR(50)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        sa.id,
        sa.kategori_aset,
        sa.no_aset,
        sa.nama,
        sa.kode_satker,
        sa.nama_satker,
        sa.rph_aset,
        sa.ur_kondisi
    FROM siman_aset sa
    WHERE sa.ur_kondisi = p_kondisi
    ORDER BY sa.rph_aset DESC;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- COMMENTS
-- =====================================================
COMMENT ON FUNCTION start_sync IS 'Mulai tracking sync untuk kategori aset';
COMMENT ON FUNCTION complete_sync IS 'Mark sync sebagai completed dengan statistik';
COMMENT ON FUNCTION fail_sync IS 'Mark sync sebagai failed dengan error message';
COMMENT ON FUNCTION get_latest_sync_status IS 'Get status sync terakhir';
COMMENT ON FUNCTION upsert_siman_aset IS 'Insert or update aset data (UPSERT pattern)';
COMMENT ON FUNCTION cleanup_old_sync_logs IS 'Cleanup sync logs lama (retention policy)';
COMMENT ON FUNCTION get_aset_summary_by_satker IS 'Get summary aset per satker';
COMMENT ON FUNCTION search_aset IS 'Full-text search untuk aset';
COMMENT ON FUNCTION get_aset_by_condition IS 'Get aset berdasarkan kondisi (Baik/Rusak)';

-- =====================================================
-- SUCCESS MESSAGE
-- =====================================================
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'SIMAN HELPER FUNCTIONS CREATED';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Functions: 9 helper functions';
    RAISE NOTICE '- start_sync()';
    RAISE NOTICE '- complete_sync()';
    RAISE NOTICE '- fail_sync()';
    RAISE NOTICE '- get_latest_sync_status()';
    RAISE NOTICE '- upsert_siman_aset()';
    RAISE NOTICE '- cleanup_old_sync_logs()';
    RAISE NOTICE '- get_aset_summary_by_satker()';
    RAISE NOTICE '- search_aset()';
    RAISE NOTICE '- get_aset_by_condition()';
    RAISE NOTICE '========================================';
END $$;
