-- ============================================================================
-- Migration: Database Optimization and Query Analysis
-- Description: Additional optimizations, query analysis tools, and performance tuning
-- Author: SIMPelv2 Team
-- Created: 2026-02-10
-- Requirements: NFR-P001, NFR-P002
-- ============================================================================

-- ============================================================================
-- SECTION 1: QUERY ANALYSIS HELPER FUNCTIONS
-- ============================================================================

-- Function: Analyze slow queries and suggest indexes
CREATE OR REPLACE FUNCTION perlengkapan.analyze_slow_queries()
RETURNS TABLE (
    query_text TEXT,
    calls BIGINT,
    total_time_ms NUMERIC,
    mean_time_ms NUMERIC,
    max_time_ms NUMERIC,
    suggested_action TEXT
) AS $
BEGIN
    RETURN QUERY
    SELECT
        SUBSTRING(query, 1, 100) as query_text,
        calls,
        ROUND((total_exec_time)::numeric, 2) as total_time_ms,
        ROUND((mean_exec_time)::numeric, 2) as mean_time_ms,
        ROUND((max_exec_time)::numeric, 2) as max_time_ms,
        CASE
            WHEN mean_exec_time > 1000 THEN 'CRITICAL: Review query and add indexes'
            WHEN mean_exec_time > 500 THEN 'WARNING: Consider optimization'
            WHEN mean_exec_time > 100 THEN 'INFO: Monitor performance'
            ELSE 'OK'
        END as suggested_action
    FROM pg_stat_statements
    WHERE query NOT LIKE '%pg_stat_statements%'
      AND query NOT LIKE '%ANALYZE%'
      AND dbid = (SELECT oid FROM pg_database WHERE datname = current_database())
    ORDER BY mean_exec_time DESC
    LIMIT 20;
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION perlengkapan.analyze_slow_queries IS 'Analyze slow queries and suggest optimizations (requires pg_stat_statements extension)';

-- Function: Check missing indexes on foreign keys
CREATE OR REPLACE FUNCTION perlengkapan.check_missing_fk_indexes()
RETURNS TABLE (
    table_name TEXT,
    column_name TEXT,
    constraint_name TEXT,
    suggested_index TEXT
) AS $
BEGIN
    RETURN QUERY
    SELECT
        tc.table_name::TEXT,
        kcu.column_name::TEXT,
        tc.constraint_name::TEXT,
        'CREATE INDEX idx_' || tc.table_name || '_' || kcu.column_name ||
        ' ON ' || tc.table_schema || '.' || tc.table_name || '(' || kcu.column_name || ');' as suggested_index
    FROM information_schema.table_constraints tc
    JOIN information_schema.key_column_usage kcu
        ON tc.constraint_name = kcu.constraint_name
        AND tc.table_schema = kcu.table_schema
    WHERE tc.constraint_type = 'FOREIGN KEY'
      AND tc.table_schema = 'perlengkapan'
      AND NOT EXISTS (
          SELECT 1
          FROM pg_indexes
          WHERE schemaname = tc.table_schema
            AND tablename = tc.table_name
            AND indexdef LIKE '%' || kcu.column_name || '%'
      )
    ORDER BY tc.table_name, kcu.column_name;
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION perlengkapan.check_missing_fk_indexes IS 'Identify foreign keys without indexes';

-- Function: Get table bloat statistics
CREATE OR REPLACE FUNCTION perlengkapan.check_table_bloat()
RETURNS TABLE (
    table_name TEXT,
    table_size TEXT,
    bloat_size TEXT,
    bloat_ratio NUMERIC,
    suggested_action TEXT
) AS $
BEGIN
    RETURN QUERY
    SELECT
        schemaname || '.' || tablename as table_name,
        pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) as table_size,
        pg_size_pretty((pg_total_relation_size(schemaname || '.' || tablename) *
            GREATEST(0, (n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0) - 0.1)))::bigint) as bloat_size,
        ROUND((n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0) * 100), 2) as bloat_ratio,
        CASE
            WHEN n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0) > 0.2 THEN 'VACUUM FULL recommended'
            WHEN n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0) > 0.1 THEN 'VACUUM recommended'
            ELSE 'OK'
        END as suggested_action
    FROM pg_stat_user_tables
    WHERE schemaname = 'perlengkapan'
      AND n_live_tup + n_dead_tup > 0
    ORDER BY n_dead_tup::numeric / NULLIF(n_live_tup + n_dead_tup, 0) DESC;
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION perlengkapan.check_table_bloat IS 'Check table bloat and suggest VACUUM operations';

-- Function: Get index usage statistics
CREATE OR REPLACE FUNCTION perlengkapan.check_index_usage()
RETURNS TABLE (
    table_name TEXT,
    index_name TEXT,
    index_size TEXT,
    index_scans BIGINT,
    rows_read BIGINT,
    rows_fetched BIGINT,
    usage_ratio NUMERIC,
    suggested_action TEXT
) AS $
BEGIN
    RETURN QUERY
    SELECT
        schemaname || '.' || tablename as table_name,
        indexrelname as index_name,
        pg_size_pretty(pg_relation_size(indexrelid)) as index_size,
        idx_scan as index_scans,
        idx_tup_read as rows_read,
        idx_tup_fetch as rows_fetched,
        CASE
            WHEN idx_scan = 0 THEN 0
            ELSE ROUND((idx_tup_fetch::numeric / NULLIF(idx_tup_read, 0) * 100), 2)
        END as usage_ratio,
        CASE
            WHEN idx_scan = 0 AND pg_relation_size(indexrelid) > 1048576 THEN 'Consider dropping (unused, >1MB)'
            WHEN idx_scan < 10 AND pg_relation_size(indexrelid) > 1048576 THEN 'Low usage, review necessity'
            ELSE 'OK'
        END as suggested_action
    FROM pg_stat_user_indexes
    WHERE schemaname = 'perlengkapan'
    ORDER BY idx_scan ASC, pg_relation_size(indexrelid) DESC;
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION perlengkapan.check_index_usage IS 'Analyze index usage and identify unused indexes';

-- ============================================================================
-- SECTION 2: ADDITIONAL COMPOSITE INDEXES FOR COMPLEX QUERIES
-- ============================================================================

-- Dashboard queries optimization
CREATE INDEX IF NOT EXISTS idx_pkb_dashboard_metrics
ON perlengkapan.pengajuan_kebutuhan_bmn(tahun, status_kode)
INCLUDE (nama, created_at, updated_at)
WHERE status_kode NOT IN (2007, 2009);  -- Exclude CANCELLED and ARCHIVED

CREATE INDEX IF NOT EXISTS idx_ppd_dashboard_metrics
ON perlengkapan.pengajuan_pakaian_dinas(tahun, status_kode)
INCLUDE (nama, created_at)
WHERE status_kode NOT IN (2007, 2009);

-- Gap analysis optimization
CREATE INDEX IF NOT EXISTS idx_pkb_barang_gap_analysis
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_barang(kode_barang, pengajuan_satker_id)
INCLUDE (nama, jumlah, jml_setuju);

-- Workflow monitoring optimization
CREATE INDEX IF NOT EXISTS idx_pkb_aktivitas_workflow
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(aktivitas_id, created_at DESC)
INCLUDE (pengajuan_satker_id, catatan);

CREATE INDEX IF NOT EXISTS idx_ppd_aktivitas_workflow
ON perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas(aktivitas_id, created_at DESC)
INCLUDE (pengajuan_satker_id, catatan);

-- Export queries optimization
CREATE INDEX IF NOT EXISTS idx_pkb_export_query
ON perlengkapan.pengajuan_kebutuhan_bmn(tahun, status_kode, created_at DESC)
INCLUDE (id, nama);

CREATE INDEX IF NOT EXISTS idx_ppd_export_query
ON perlengkapan.pengajuan_pakaian_dinas(tahun, status_kode, created_at DESC)
INCLUDE (id, nama);

-- Batch operations optimization
CREATE INDEX IF NOT EXISTS idx_pkb_satker_batch_ops
ON perlengkapan.pengajuan_kebutuhan_bmn_satker(status_kode, pengajuan_id)
INCLUDE (id, ms_satker_id, nm_satker);

CREATE INDEX IF NOT EXISTS idx_ppd_satker_batch_ops
ON perlengkapan.pengajuan_pakaian_dinas_satker(status_kode, pengajuan_id)
INCLUDE (id, satker_id);

-- ============================================================================
-- SECTION 3: INTEGRATION SCHEMA OPTIMIZATIONS
-- ============================================================================

-- Composite indexes for gap analysis queries
CREATE INDEX IF NOT EXISTS idx_siman_tanah_satker_kode_kondisi
ON integrasi.siman_aset_tanah(satker_code, kode_barang, kondisi)
INCLUDE (nama_barang, nilai_perolehan);

CREATE INDEX IF NOT EXISTS idx_siman_gedung_satker_kode_kondisi
ON integrasi.siman_aset_gedung_bangunan(satker_code, kode_barang, kondisi)
INCLUDE (nama_barang, nilai_perolehan);

CREATE INDEX IF NOT EXISTS idx_siman_alat_satker_kode_kondisi
ON integrasi.siman_aset_alat_besar(satker_code, kode_barang, kondisi)
INCLUDE (nama_barang, nilai_perolehan);

CREATE INDEX IF NOT EXISTS idx_siman_kendaraan_satker_kode_kondisi
ON integrasi.siman_aset_angkutan_bermotor(satker_code, kode_barang, kondisi)
INCLUDE (nama_barang, nilai_perolehan, no_polisi);

-- MySIMKARI composite indexes for pakaian dinas queries
CREATE INDEX IF NOT EXISTS idx_mysimkari_satker_status_jk
ON integrasi.mysimkari_pegawai(satker_code, status_pegawai, jenis_kelamin)
INCLUDE (nip, nama, golongan);

-- API call log optimization for monitoring
CREATE INDEX IF NOT EXISTS idx_api_log_service_status_time
ON integrasi.api_call_log(service_name, status_code, called_at DESC)
INCLUDE (endpoint, duration_ms);

-- Sync status optimization
CREATE INDEX IF NOT EXISTS idx_sync_status_service_entity_time
ON integrasi.sync_status(service_name, entity_type, started_at DESC)
INCLUDE (status, success_records, failed_records, duration_seconds);

-- ============================================================================
-- SECTION 4: QUERY RESULT CACHING SETUP
-- ============================================================================

-- Create schema for cached query results
CREATE SCHEMA IF NOT EXISTS cache;

COMMENT ON SCHEMA cache IS 'Schema for cached query results and materialized views';

-- Materialized view for dashboard metrics (refresh every 5 minutes)
CREATE MATERIALIZED VIEW IF NOT EXISTS cache.mv_dashboard_metrics AS
SELECT
    tahun,
    status_kode,
    COUNT(*) as total_pengajuan,
    COUNT(DISTINCT pengajuan_id) as unique_pengajuan,
    SUM(CASE WHEN status_kode = 2004 THEN 1 ELSE 0 END) as approved_count,
    SUM(CASE WHEN status_kode = 2005 THEN 1 ELSE 0 END) as rejected_count,
    SUM(CASE WHEN status_kode IN (2002, 2003) THEN 1 ELSE 0 END) as pending_count,
    NOW() as last_refreshed
FROM perlengkapan.pengajuan_kebutuhan_bmn_satker
GROUP BY tahun, status_kode;

CREATE UNIQUE INDEX idx_mv_dashboard_metrics_pk ON cache.mv_dashboard_metrics(tahun, status_kode);

COMMENT ON MATERIALIZED VIEW cache.mv_dashboard_metrics IS 'Cached dashboard metrics (refresh every 5 minutes)';

-- Materialized view for gap analysis (refresh every 1 hour)
CREATE MATERIALIZED VIEW IF NOT EXISTS cache.mv_gap_analysis AS
SELECT
    pkb.kode_barang,
    pkb.nama as nama_barang,
    pks.ms_satker_id as satker_id,
    pks.nm_satker as satker_nama,
    SUM(pkb.jumlah) as jumlah_kebutuhan,
    COALESCE(
        (SELECT COUNT(*)
         FROM integrasi.siman_aset_tanah sa
         WHERE sa.satker_code = pks.kode_satker
           AND sa.kode_barang = pkb.kode_barang
           AND sa.kondisi = 'BAIK'),
        0
    ) +
    COALESCE(
        (SELECT COUNT(*)
         FROM integrasi.siman_aset_gedung_bangunan sa
         WHERE sa.satker_code = pks.kode_satker
           AND sa.kode_barang = pkb.kode_barang
           AND sa.kondisi = 'BAIK'),
        0
    ) +
    COALESCE(
        (SELECT COUNT(*)
         FROM integrasi.siman_aset_alat_besar sa
         WHERE sa.satker_code = pks.kode_satker
           AND sa.kode_barang = pkb.kode_barang
           AND sa.kondisi = 'BAIK'),
        0
    ) +
    COALESCE(
        (SELECT COUNT(*)
         FROM integrasi.siman_aset_angkutan_bermotor sa
         WHERE sa.satker_code = pks.kode_satker
           AND sa.kode_barang = pkb.kode_barang
           AND sa.kondisi = 'BAIK'),
        0
    ) as existing_good_quantity,
    SUM(pkb.jumlah) - (
        COALESCE(
            (SELECT COUNT(*)
             FROM integrasi.siman_aset_tanah sa
             WHERE sa.satker_code = pks.kode_satker
               AND sa.kode_barang = pkb.kode_barang
               AND sa.kondisi = 'BAIK'),
            0
        ) +
        COALESCE(
            (SELECT COUNT(*)
             FROM integrasi.siman_aset_gedung_bangunan sa
             WHERE sa.satker_code = pks.kode_satker
               AND sa.kode_barang = pkb.kode_barang
               AND sa.kondisi = 'BAIK'),
            0
        ) +
        COALESCE(
            (SELECT COUNT(*)
             FROM integrasi.siman_aset_alat_besar sa
             WHERE sa.satker_code = pks.kode_satker
               AND sa.kode_barang = pkb.kode_barang
               AND sa.kondisi = 'BAIK'),
            0
        ) +
        COALESCE(
            (SELECT COUNT(*)
             FROM integrasi.siman_aset_angkutan_bermotor sa
             WHERE sa.satker_code = pks.kode_satker
               AND sa.kode_barang = pkb.kode_barang
               AND sa.kondisi = 'BAIK'),
            0
        )
    ) as gap,
    NOW() as last_refreshed
FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang pkb
JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker pks ON pkb.pengajuan_satker_id = pks.id
WHERE pks.status_kode NOT IN (2007, 2009)  -- Exclude CANCELLED and ARCHIVED
GROUP BY pkb.kode_barang, pkb.nama, pks.ms_satker_id, pks.nm_satker, pks.kode_satker;

CREATE INDEX idx_mv_gap_analysis_satker ON cache.mv_gap_analysis(satker_id);
CREATE INDEX idx_mv_gap_analysis_kode ON cache.mv_gap_analysis(kode_barang);
CREATE INDEX idx_mv_gap_analysis_gap ON cache.mv_gap_analysis(gap DESC) WHERE gap > 0;

COMMENT ON MATERIALIZED VIEW cache.mv_gap_analysis IS 'Cached gap analysis results (refresh every 1 hour)';

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION cache.refresh_materialized_views()
RETURNS TEXT AS $
DECLARE
    v_start_time TIMESTAMPTZ;
    v_end_time TIMESTAMPTZ;
    v_duration INTERVAL;
BEGIN
    v_start_time := clock_timestamp();

    -- Refresh dashboard metrics (fast)
    REFRESH MATERIALIZED VIEW CONCURRENTLY cache.mv_dashboard_metrics;

    -- Refresh gap analysis (slower)
    REFRESH MATERIALIZED VIEW CONCURRENTLY cache.mv_gap_analysis;

    v_end_time := clock_timestamp();
    v_duration := v_end_time - v_start_time;

    RETURN 'Materialized views refreshed in ' || v_duration::TEXT;
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cache.refresh_materialized_views IS 'Refresh all materialized views (call every 5 minutes for dashboard, every 1 hour for gap analysis)';

-- ============================================================================
-- SECTION 5: QUERY OPTIMIZATION SETTINGS
-- ============================================================================

-- Set optimal work_mem for complex queries (per connection)
-- This should be set in postgresql.conf or per-session
-- ALTER DATABASE simpelv2 SET work_mem = '64MB';

-- Set optimal shared_buffers (25% of RAM, set in postgresql.conf)
-- shared_buffers = 2GB  (for 8GB RAM server)

-- Set optimal effective_cache_size (50-75% of RAM)
-- effective_cache_size = 6GB  (for 8GB RAM server)

-- Enable parallel query execution
-- ALTER DATABASE simpelv2 SET max_parallel_workers_per_gather = 4;
-- ALTER DATABASE simpelv2 SET max_parallel_workers = 8;

-- Set optimal random_page_cost for SSD
-- ALTER DATABASE simpelv2 SET random_page_cost = 1.1;

-- ============================================================================
-- SECTION 6: AUTOVACUUM TUNING
-- ============================================================================

-- Tune autovacuum for high-traffic tables
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02,
    autovacuum_vacuum_cost_delay = 10
);

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02
);

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02
);

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas SET (
    autovacuum_vacuum_scale_factor = 0.05,
    autovacuum_analyze_scale_factor = 0.02
);

ALTER TABLE integrasi.api_call_log SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_scale_factor = 0.05,
    autovacuum_vacuum_cost_delay = 5
);

-- ============================================================================
-- SECTION 7: STATISTICS TARGETS
-- ============================================================================

-- Increase statistics target for frequently queried columns
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn
    ALTER COLUMN status_kode SET STATISTICS 1000,
    ALTER COLUMN tahun SET STATISTICS 1000;

ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang
    ALTER COLUMN kode_barang SET STATISTICS 1000,
    ALTER COLUMN prioritas SET STATISTICS 500;

ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
    ALTER COLUMN status_kode SET STATISTICS 1000,
    ALTER COLUMN tahun SET STATISTICS 1000;

ALTER TABLE integrasi.siman_aset_tanah
    ALTER COLUMN kode_barang SET STATISTICS 1000,
    ALTER COLUMN kondisi SET STATISTICS 500;

-- ============================================================================
-- SECTION 8: ANALYZE ALL TABLES
-- ============================================================================

-- Update statistics for query planner
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker;
ANALYZE perlengkapan.pengajuan_kebutuhan_bmn_satker_barang;
ANALYZE perlengkapan.pengajuan_pakaian_dinas;
ANALYZE perlengkapan.pengajuan_pakaian_dinas_satker;
ANALYZE perlengkapan.roadmap_sarpras;
ANALYZE perlengkapan.mapping_kodefikasi;
ANALYZE perlengkapan.riwayat_pemenuhan;
ANALYZE perlengkapan.izin_pemakaian_bmn;
ANALYZE integrasi.siman_aset_tanah;
ANALYZE integrasi.siman_aset_gedung_bangunan;
ANALYZE integrasi.siman_aset_alat_besar;
ANALYZE integrasi.siman_aset_angkutan_bermotor;
ANALYZE integrasi.mysimkari_pegawai;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $
DECLARE
    v_total_indexes INTEGER;
    v_total_tables INTEGER;
    v_total_size TEXT;
BEGIN
    -- Count indexes
    SELECT COUNT(*) INTO v_total_indexes
    FROM pg_indexes
    WHERE schemaname IN ('perlengkapan', 'integrasi', 'cache');

    -- Count tables
    SELECT COUNT(*) INTO v_total_tables
    FROM pg_tables
    WHERE schemaname IN ('perlengkapan', 'integrasi', 'cache');

    -- Get total database size
    SELECT pg_size_pretty(pg_database_size(current_database())) INTO v_total_size;

    RAISE NOTICE '✅ Database optimization completed successfully';
    RAISE NOTICE '📊 Statistics:';
    RAISE NOTICE '  - Total tables: %', v_total_tables;
    RAISE NOTICE '  - Total indexes: %', v_total_indexes;
    RAISE NOTICE '  - Database size: %', v_total_size;
    RAISE NOTICE '🔧 Optimizations applied:';
    RAISE NOTICE '  - Query analysis functions created';
    RAISE NOTICE '  - Additional composite indexes added';
    RAISE NOTICE '  - Materialized views for caching created';
    RAISE NOTICE '  - Autovacuum tuning applied';
    RAISE NOTICE '  - Statistics targets increased';
    RAISE NOTICE '📝 Next steps:';
    RAISE NOTICE '  1. Run: SELECT * FROM perlengkapan.analyze_slow_queries();';
    RAISE NOTICE '  2. Run: SELECT * FROM perlengkapan.check_missing_fk_indexes();';
    RAISE NOTICE '  3. Run: SELECT * FROM perlengkapan.check_table_bloat();';
    RAISE NOTICE '  4. Run: SELECT * FROM perlengkapan.check_index_usage();';
    RAISE NOTICE '  5. Schedule: SELECT cache.refresh_materialized_views(); (every 5 minutes)';
    RAISE NOTICE '⚡ Query performance should be significantly improved';
END $;
