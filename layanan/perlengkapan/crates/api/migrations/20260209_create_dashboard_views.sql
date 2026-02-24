-- ============================================================================
-- Migration: Create Dashboard Views
-- Description: Optimized views and materialized views for dashboard queries
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- Requirements: REQ-DB001, REQ-DB013
-- ============================================================================

-- Ensure schema exists
CREATE SCHEMA IF NOT EXISTS perlengkapan;

-- ============================================================================
-- GAP ANALYSIS VIEW
-- Requirement: REQ-DB001
-- ============================================================================

CREATE OR REPLACE VIEW perlengkapan.v_gap_analysis AS
SELECT
    -- Satker Information
    k.satker_id,
    k.satker_nama,

    -- Asset Information
    kb.kode_barang,
    kb.nama_barang,

    -- Standard Quantity (from kebutuhan)
    COALESCE(SUM(kb.jumlah), 0) as standard_quantity,

    -- Existing Good Quantity (from SIMAN)
    COALESCE((
        SELECT COUNT(*)
        FROM integrasi.siman_aset_tanah sa
        WHERE sa.satker_id = k.satker_id::TEXT
        AND sa.kode_barang = kb.kode_barang
        AND sa.kondisi = 'BAIK'
    ), 0) +
    COALESCE((
        SELECT COUNT(*)
        FROM integrasi.siman_aset_gedung_bangunan sa
        WHERE sa.satker_id = k.satker_id::TEXT
        AND sa.kode_barang = kb.kode_barang
        AND sa.kondisi = 'BAIK'
    ), 0) +
    COALESCE((
        SELECT COUNT(*)
        FROM integrasi.siman_aset_alat_besar sa
        WHERE sa.satker_id = k.satker_id::TEXT
        AND sa.kode_barang = kb.kode_barang
        AND sa.kondisi = 'BAIK'
    ), 0) +
    COALESCE((
        SELECT COUNT(*)
        FROM integrasi.siman_aset_angkutan_bermotor sa
        WHERE sa.satker_id = k.satker_id::TEXT
        AND sa.kode_barang = kb.kode_barang
        AND sa.kondisi = 'BAIK'
    ), 0) as existing_good_quantity,

    -- Gap Calculation
    GREATEST(
        COALESCE(SUM(kb.jumlah), 0) - (
            COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_tanah sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
            COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_gedung_bangunan sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
            COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_alat_besar sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
            COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_angkutan_bermotor sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0)
        ),
        0
    ) as gap,

    -- Priority Score (from kebutuhan)
    AVG(kb.prioritas) as avg_priority,
    AVG(kb.skor) as avg_score,

    -- Tahun Anggaran
    p.tahun as tahun_anggaran

FROM perlengkapan.pengajuan_kebutuhan_bmn p
JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker k ON p.id = k.pengajuan_id
JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang kb ON k.id = kb.pengajuan_satker_id
WHERE p.status_kode >= 2004  -- Only approved/analyzed requests
GROUP BY k.satker_id, k.satker_nama, kb.kode_barang, kb.nama_barang, p.tahun
HAVING GREATEST(
    COALESCE(SUM(kb.jumlah), 0) - (
        COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_tanah sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
        COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_gedung_bangunan sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
        COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_alat_besar sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0) +
        COALESCE((SELECT COUNT(*) FROM integrasi.siman_aset_angkutan_bermotor sa WHERE sa.satker_id = k.satker_id::TEXT AND sa.kode_barang = kb.kode_barang AND sa.kondisi = 'BAIK'), 0)
    ),
    0
) > 0  -- Only show items with gap
ORDER BY gap DESC, avg_priority DESC;

COMMENT ON VIEW perlengkapan.v_gap_analysis IS 'Gap analysis: standard quantity vs existing good condition assets from SIMAN';

-- ============================================================================
-- DASHBOARD METRICS MATERIALIZED VIEW
-- Requirement: REQ-DB001, REQ-DB013
-- ============================================================================

CREATE MATERIALIZED VIEW IF NOT EXISTS perlengkapan.mv_dashboard_metrics AS
SELECT
    -- Timestamp
    NOW() as last_refreshed_at,

    -- Kebutuhan BMN Metrics
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn) as total_pengajuan_kebutuhan,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE status_kode = 2000) as draft_count,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE status_kode = 2002) as submitted_count,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE status_kode = 2006) as approved_count,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn WHERE status_kode = 2007) as rejected_count,

    -- Kebutuhan BMN by Year
    (SELECT jsonb_object_agg(tahun, count) FROM (
        SELECT tahun, COUNT(*) as count
        FROM perlengkapan.pengajuan_kebutuhan_bmn
        GROUP BY tahun
        ORDER BY tahun DESC
        LIMIT 5
    ) t) as kebutuhan_by_year,

    -- Kebutuhan BMN by Status
    (SELECT jsonb_object_agg(status_nama, count) FROM (
        SELECT m.nama as status_nama, COUNT(p.id) as count
        FROM perlengkapan.ms_aktivitas_bmn m
        LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn p ON m.kode = p.status_kode
        WHERE m.is_active = TRUE
        GROUP BY m.nama
    ) t) as kebutuhan_by_status,

    -- Total Satker Involved
    (SELECT COUNT(DISTINCT satker_id) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker) as total_satker_kebutuhan,

    -- Total Barang Items
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang) as total_barang_items,
    (SELECT COALESCE(SUM(jumlah), 0) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang) as total_jumlah_diminta,
    (SELECT COALESCE(SUM(jml_setuju), 0) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_barang) as total_jumlah_disetujui,

    -- Pakaian Dinas Metrics
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas) as total_pengajuan_pakaian,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas WHERE status_kode = 2000) as pakaian_draft_count,
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas WHERE status_kode = 2006) as pakaian_approved_count,

    -- Pakaian Dinas by Year
    (SELECT jsonb_object_agg(tahun, count) FROM (
        SELECT tahun, COUNT(*) as count
        FROM perlengkapan.pengajuan_pakaian_dinas
        GROUP BY tahun
        ORDER BY tahun DESC
        LIMIT 5
    ) t) as pakaian_by_year,

    -- Total Pegawai in Pakaian Dinas
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai) as total_pegawai_pakaian,

    -- Roadmap Metrics
    (SELECT COUNT(*) FROM perlengkapan.roadmap_sarpras) as total_roadmap_items,
    (SELECT COUNT(*) FROM perlengkapan.roadmap_sarpras WHERE status_pemenuhan = 'COMPLETED') as roadmap_completed_count,
    (SELECT COUNT(*) FROM perlengkapan.roadmap_sarpras WHERE status_pemenuhan = 'IN_PROGRESS') as roadmap_in_progress_count,

    -- Roadmap by Period
    (SELECT jsonb_object_agg(periode, count) FROM (
        SELECT CONCAT(periode_mulai, '-', periode_akhir) as periode, COUNT(*) as count
        FROM perlengkapan.roadmap_sarpras
        GROUP BY periode_mulai, periode_akhir
        ORDER BY periode_mulai DESC
    ) t) as roadmap_by_period,

    -- Fulfillment Metrics
    (SELECT COUNT(*) FROM perlengkapan.riwayat_pemenuhan) as total_pemenuhan,
    (SELECT COALESCE(SUM(jumlah_terpenuhi), 0) FROM perlengkapan.riwayat_pemenuhan) as total_jumlah_terpenuhi,
    (SELECT COALESCE(SUM(nilai_perolehan), 0) FROM perlengkapan.riwayat_pemenuhan) as total_nilai_perolehan,

    -- Fulfillment by Source
    (SELECT jsonb_object_agg(sumber_data, count) FROM (
        SELECT sumber_data, COUNT(*) as count
        FROM perlengkapan.riwayat_pemenuhan
        GROUP BY sumber_data
    ) t) as pemenuhan_by_source,

    -- Mapping Kodefikasi Metrics
    (SELECT COUNT(*) FROM perlengkapan.mapping_kodefikasi) as total_mapping,
    (SELECT COUNT(*) FROM perlengkapan.mapping_kodefikasi WHERE status_mapping = 'APPROVED') as mapping_approved_count,
    (SELECT COUNT(*) FROM perlengkapan.mapping_kodefikasi WHERE status_mapping = 'PROPOSED') as mapping_proposed_count,

    -- Izin Pemakaian BMN Metrics
    (SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn) as total_izin_pemakaian,
    (SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn WHERE status = 'ACTIVE') as izin_active_count,
    (SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn WHERE status = 'EXPIRED') as izin_expired_count,
    (SELECT COUNT(*) FROM perlengkapan.izin_pemakaian_bmn WHERE status = 'ACTIVE' AND tanggal_selesai < CURRENT_DATE + INTERVAL '30 days') as izin_expiring_soon_count,

    -- Izin by Type
    (SELECT jsonb_object_agg(jenis_bmn, count) FROM (
        SELECT jenis_bmn, COUNT(*) as count
        FROM perlengkapan.izin_pemakaian_bmn
        WHERE status = 'ACTIVE'
        GROUP BY jenis_bmn
    ) t) as izin_by_type,

    -- Integration Metrics (from integrasi schema)
    (SELECT COUNT(*) FROM integrasi.siman_aset_tanah) as siman_tanah_count,
    (SELECT COUNT(*) FROM integrasi.siman_aset_gedung_bangunan) as siman_gedung_count,
    (SELECT COUNT(*) FROM integrasi.siman_aset_alat_besar) as siman_alat_besar_count,
    (SELECT COUNT(*) FROM integrasi.siman_aset_angkutan_bermotor) as siman_angkutan_count,
    (SELECT COUNT(*) FROM integrasi.mysimkari_pegawai) as mysimkari_pegawai_count,
    (SELECT COUNT(*) FROM integrasi.mysimkari_satker) as mysimkari_satker_count,

    -- Workflow Metrics
    (SELECT COUNT(*) FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas WHERE created_at > CURRENT_DATE - INTERVAL '7 days') as workflow_activities_last_7_days,
    (SELECT COUNT(*) FROM perlengkapan.parallel_approvals WHERE status = 'PENDING') as pending_parallel_approvals,

    -- Top 10 Gap Analysis (kode_barang with highest gap)
    (SELECT jsonb_agg(row_to_json(t)) FROM (
        SELECT kode_barang, nama_barang, SUM(gap) as total_gap
        FROM perlengkapan.v_gap_analysis
        GROUP BY kode_barang, nama_barang
        ORDER BY total_gap DESC
        LIMIT 10
    ) t) as top_10_gap_items,

    -- Top 10 Satker by Kebutuhan Count
    (SELECT jsonb_agg(row_to_json(t)) FROM (
        SELECT satker_id, satker_nama, COUNT(*) as kebutuhan_count
        FROM perlengkapan.pengajuan_kebutuhan_bmn_satker
        GROUP BY satker_id, satker_nama
        ORDER BY kebutuhan_count DESC
        LIMIT 10
    ) t) as top_10_satker_by_kebutuhan;

-- Create unique index for concurrent refresh
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_dashboard_metrics_refresh
ON perlengkapan.mv_dashboard_metrics(last_refreshed_at);

COMMENT ON MATERIALIZED VIEW perlengkapan.mv_dashboard_metrics IS 'Materialized view with aggregated dashboard metrics, refreshed every 5 minutes';

-- ============================================================================
-- REFRESH FUNCTION FOR MATERIALIZED VIEW
-- ============================================================================

CREATE OR REPLACE FUNCTION perlengkapan.refresh_dashboard_metrics()
RETURNS void AS $
BEGIN
    -- Refresh materialized view concurrently (non-blocking)
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_dashboard_metrics;

    RAISE NOTICE 'Dashboard metrics refreshed at %', NOW();
END;
$ LANGUAGE plpgsql;

COMMENT ON FUNCTION perlengkapan.refresh_dashboard_metrics() IS 'Refresh dashboard metrics materialized view (call every 5 minutes via cron)';

-- ============================================================================
-- ADDITIONAL DASHBOARD VIEWS
-- ============================================================================

-- View: Kebutuhan BMN Summary by Satker
CREATE OR REPLACE VIEW perlengkapan.v_kebutuhan_summary_by_satker AS
SELECT
    k.satker_id,
    k.satker_nama,
    p.tahun,
    COUNT(DISTINCT k.id) as total_pengajuan,
    COUNT(DISTINCT kb.id) as total_barang_items,
    COALESCE(SUM(kb.jumlah), 0) as total_jumlah_diminta,
    COALESCE(SUM(kb.jml_setuju), 0) as total_jumlah_disetujui,
    COUNT(DISTINCT k.id) FILTER (WHERE k.status_kode = 2006) as approved_count,
    COUNT(DISTINCT k.id) FILTER (WHERE k.status_kode = 2007) as rejected_count,
    MAX(k.updated_at) as last_updated_at
FROM perlengkapan.pengajuan_kebutuhan_bmn p
JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker k ON p.id = k.pengajuan_id
LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang kb ON k.id = kb.pengajuan_satker_id
GROUP BY k.satker_id, k.satker_nama, p.tahun
ORDER BY p.tahun DESC, k.satker_nama;

COMMENT ON VIEW perlengkapan.v_kebutuhan_summary_by_satker IS 'Summary of kebutuhan BMN by satker and year';

-- View: Pakaian Dinas Summary by Satker
CREATE OR REPLACE VIEW perlengkapan.v_pakaian_summary_by_satker AS
SELECT
    ps.satker_id,
    p.tahun,
    COUNT(DISTINCT ps.id) as total_pengajuan,
    COUNT(DISTINCT psp.id) as total_pegawai,
    COUNT(DISTINCT ps.id) FILTER (WHERE ps.status_kode = 2006) as approved_count,
    COUNT(DISTINCT ps.id) FILTER (WHERE ps.status_kode = 2007) as rejected_count,
    MAX(ps.updated_at) as last_updated_at
FROM perlengkapan.pengajuan_pakaian_dinas p
JOIN perlengkapan.pengajuan_pakaian_dinas_satker ps ON p.id = ps.pengajuan_id
LEFT JOIN perlengkapan.pengajuan_pakaian_dinas_satker_pegawai psp ON ps.id = psp.pengajuan_satker_id
GROUP BY ps.satker_id, p.tahun
ORDER BY p.tahun DESC, ps.satker_id;

COMMENT ON VIEW perlengkapan.v_pakaian_summary_by_satker IS 'Summary of pakaian dinas by satker and year';

-- View: Workflow Performance Metrics
CREATE OR REPLACE VIEW perlengkapan.v_workflow_performance AS
SELECT
    'kebutuhan_bmn' as module,
    k.satker_id,
    k.satker_nama,
    m.nama as current_status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (NOW() - k.created_at)) / 86400) as avg_days_in_status,
    MIN(k.created_at) as oldest_created_at,
    MAX(k.updated_at) as latest_updated_at
FROM perlengkapan.pengajuan_kebutuhan_bmn_satker k
JOIN perlengkapan.ms_aktivitas_bmn m ON k.status_kode = m.kode
WHERE k.status_kode NOT IN (2006, 2007, 2008, 2009)  -- Exclude completed statuses
GROUP BY k.satker_id, k.satker_nama, m.nama

UNION ALL

SELECT
    'pakaian_dinas' as module,
    ps.satker_id,
    NULL as satker_nama,
    m.nama as current_status,
    COUNT(*) as count,
    AVG(EXTRACT(EPOCH FROM (NOW() - ps.created_at)) / 86400) as avg_days_in_status,
    MIN(ps.created_at) as oldest_created_at,
    MAX(ps.updated_at) as latest_updated_at
FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
JOIN perlengkapan.ms_aktivitas_bmn m ON ps.status_kode = m.kode
WHERE ps.status_kode NOT IN (2006, 2007, 2008, 2009)
GROUP BY ps.satker_id, m.nama

ORDER BY module, avg_days_in_status DESC;

COMMENT ON VIEW perlengkapan.v_workflow_performance IS 'Workflow performance metrics showing bottlenecks and average processing time';

-- View: Asset Utilization by Satker (from SIMAN)
CREATE OR REPLACE VIEW perlengkapan.v_asset_utilization_by_satker AS
SELECT
    satker_id,
    'tanah' as asset_type,
    COUNT(*) as total_assets,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    ROUND((COUNT(*) FILTER (WHERE kondisi = 'BAIK')::NUMERIC / NULLIF(COUNT(*), 0)::NUMERIC) * 100, 2) as utilization_rate
FROM integrasi.siman_aset_tanah
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'gedung_bangunan' as asset_type,
    COUNT(*) as total_assets,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    ROUND((COUNT(*) FILTER (WHERE kondisi = 'BAIK')::NUMERIC / NULLIF(COUNT(*), 0)::NUMERIC) * 100, 2) as utilization_rate
FROM integrasi.siman_aset_gedung_bangunan
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'alat_besar' as asset_type,
    COUNT(*) as total_assets,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    ROUND((COUNT(*) FILTER (WHERE kondisi = 'BAIK')::NUMERIC / NULLIF(COUNT(*), 0)::NUMERIC) * 100, 2) as utilization_rate
FROM integrasi.siman_aset_alat_besar
WHERE satker_id IS NOT NULL
GROUP BY satker_id

UNION ALL

SELECT
    satker_id,
    'angkutan_bermotor' as asset_type,
    COUNT(*) as total_assets,
    COUNT(*) FILTER (WHERE kondisi = 'BAIK') as baik_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK RINGAN') as rusak_ringan_count,
    COUNT(*) FILTER (WHERE kondisi = 'RUSAK BERAT') as rusak_berat_count,
    ROUND((COUNT(*) FILTER (WHERE kondisi = 'BAIK')::NUMERIC / NULLIF(COUNT(*), 0)::NUMERIC) * 100, 2) as utilization_rate
FROM integrasi.siman_aset_angkutan_bermotor
WHERE satker_id IS NOT NULL
GROUP BY satker_id

ORDER BY satker_id, asset_type;

COMMENT ON VIEW perlengkapan.v_asset_utilization_by_satker IS 'Asset utilization rate by satker based on condition from SIMAN';

-- View: Trend Analysis Year-over-Year
CREATE OR REPLACE VIEW perlengkapan.v_trend_analysis_yoy AS
SELECT
    tahun,
    COUNT(DISTINCT p.id) as total_pengajuan,
    COUNT(DISTINCT k.satker_id) as total_satker,
    COUNT(DISTINCT kb.id) as total_barang_items,
    COALESCE(SUM(kb.jumlah), 0) as total_jumlah_diminta,
    COALESCE(SUM(kb.jml_setuju), 0) as total_jumlah_disetujui,
    ROUND(
        (COALESCE(SUM(kb.jml_setuju), 0)::NUMERIC / NULLIF(COALESCE(SUM(kb.jumlah), 0), 0)::NUMERIC) * 100,
        2
    ) as approval_rate,
    LAG(COUNT(DISTINCT p.id)) OVER (ORDER BY tahun) as prev_year_pengajuan,
    ROUND(
        ((COUNT(DISTINCT p.id)::NUMERIC - LAG(COUNT(DISTINCT p.id)) OVER (ORDER BY tahun)::NUMERIC) /
         NULLIF(LAG(COUNT(DISTINCT p.id)) OVER (ORDER BY tahun), 0)::NUMERIC) * 100,
        2
    ) as yoy_growth_rate
FROM perlengkapan.pengajuan_kebutuhan_bmn p
LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker k ON p.id = k.pengajuan_id
LEFT JOIN perlengkapan.pengajuan_kebutuhan_bmn_satker_barang kb ON k.id = kb.pengajuan_satker_id
GROUP BY tahun
ORDER BY tahun DESC;

COMMENT ON VIEW perlengkapan.v_trend_analysis_yoy IS 'Year-over-year trend analysis for kebutuhan BMN';

-- ============================================================================
-- CRON JOB SETUP INSTRUCTIONS
-- ============================================================================

-- To set up automatic refresh every 5 minutes, use pg_cron extension:
--
-- 1. Install pg_cron extension:
--    CREATE EXTENSION IF NOT EXISTS pg_cron;
--
-- 2. Schedule the refresh job:
--    SELECT cron.schedule(
--        'refresh-dashboard-metrics',
--        '*/5 * * * *',  -- Every 5 minutes
--        'SELECT perlengkapan.refresh_dashboard_metrics();'
--    );
--
-- 3. Verify the job:
--    SELECT * FROM cron.job WHERE jobname = 'refresh-dashboard-metrics';
--
-- 4. Check job run history:
--    SELECT * FROM cron.job_run_details WHERE jobid = (
--        SELECT jobid FROM cron.job WHERE jobname = 'refresh-dashboard-metrics'
--    ) ORDER BY start_time DESC LIMIT 10;

-- Alternative: Use system cron if pg_cron is not available
-- Add to crontab:
-- */5 * * * * psql -U simpelv2 -d simpelv2 -c "SELECT perlengkapan.refresh_dashboard_metrics();"

-- ============================================================================
-- INITIAL REFRESH
-- ============================================================================

-- Perform initial refresh of materialized view
REFRESH MATERIALIZED VIEW perlengkapan.mv_dashboard_metrics;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $
BEGIN
    RAISE NOTICE '✅ Dashboard views created successfully';
    RAISE NOTICE '📊 Created: 1 materialized view (mv_dashboard_metrics)';
    RAISE NOTICE '👁️ Created: 6 regular views';
    RAISE NOTICE '  - v_gap_analysis (gap analysis with SIMAN data)';
    RAISE NOTICE '  - v_kebutuhan_summary_by_satker';
    RAISE NOTICE '  - v_pakaian_summary_by_satker';
    RAISE NOTICE '  - v_workflow_performance';
    RAISE NOTICE '  - v_asset_utilization_by_satker';
    RAISE NOTICE '  - v_trend_analysis_yoy';
    RAISE NOTICE '🔧 Created: 1 refresh function';
    RAISE NOTICE '⏰ Setup cron job to refresh every 5 minutes (see instructions in migration file)';
    RAISE NOTICE '🔍 Schema: perlengkapan';
END $;
