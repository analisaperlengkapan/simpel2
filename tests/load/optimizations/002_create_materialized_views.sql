-- Performance Optimization: Materialized Views for Dashboards
-- Based on load test analysis
-- Requirements: NFR-P004 (Dashboard load time ≤ 5 seconds)

-- ============================================
-- Dashboard Metrics Materialized View
-- ============================================

CREATE MATERIALIZED VIEW IF NOT EXISTS perlengkapan.mv_dashboard_metrics AS
SELECT
    tahun_anggaran,
    COUNT(*) as total_kebutuhan,
    SUM(jumlah_kebutuhan) as total_jumlah,
    COUNT(DISTINCT satker_id) as total_satker,
    COUNT(CASE WHEN status = 'DRAFT' THEN 1 END) as draft_count,
    COUNT(CASE WHEN status = 'SUBMITTED' THEN 1 END) as submitted_count,
    COUNT(CASE WHEN status = 'REVIEWED' THEN 1 END) as reviewed_count,
    COUNT(CASE WHEN status = 'APPROVED' THEN 1 END) as approved_count,
    COUNT(CASE WHEN status = 'REJECTED' THEN 1 END) as rejected_count,
    MAX(updated_at) as last_updated
FROM perlengkapan.kebutuhan_bmn
GROUP BY tahun_anggaran;

-- Create unique index for concurrent refresh
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_dashboard_metrics_tahun
ON perlengkapan.mv_dashboard_metrics(tahun_anggaran);

-- ============================================
-- Gap Analysis Materialized View
-- ============================================

CREATE MATERIALIZED VIEW IF NOT EXISTS perlengkapan.mv_gap_analysis AS
SELECT
    k.satker_id,
    k.kode_barang,
    k.nama_barang,
    k.tahun_anggaran,
    k.jumlah_kebutuhan AS standard_quantity,
    COALESCE(COUNT(CASE WHEN sa.kondisi = 'BAIK' THEN 1 END), 0) AS existing_good_quantity,
    k.jumlah_kebutuhan - COALESCE(COUNT(CASE WHEN sa.kondisi = 'BAIK' THEN 1 END), 0) AS gap,
    MAX(k.updated_at) as last_updated
FROM perlengkapan.kebutuhan_bmn k
LEFT JOIN integrasi.siman_aset_tanah sa
    ON k.kode_barang = sa.kode_barang
    AND k.satker_id = sa.satker_id
GROUP BY k.satker_id, k.kode_barang, k.nama_barang, k.tahun_anggaran, k.jumlah_kebutuhan;

-- Create unique index for concurrent refresh
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_gap_analysis_unique
ON perlengkapan.mv_gap_analysis(satker_id, kode_barang, tahun_anggaran);

-- Additional indexes for filtering
CREATE INDEX IF NOT EXISTS idx_mv_gap_analysis_tahun
ON perlengkapan.mv_gap_analysis(tahun_anggaran);

CREATE INDEX IF NOT EXISTS idx_mv_gap_analysis_gap
ON perlengkapan.mv_gap_analysis(gap DESC)
WHERE gap > 0;

-- ============================================
-- Workflow Metrics Materialized View
-- ============================================

CREATE MATERIALIZED VIEW IF NOT EXISTS perlengkapan.mv_workflow_metrics AS
SELECT
    DATE_TRUNC('day', a.created_at) as date,
    a.aktivitas_id,
    ma.nama as aktivitas_nama,
    COUNT(*) as total_transitions,
    AVG(EXTRACT(EPOCH FROM (
        LEAD(a.created_at) OVER (PARTITION BY a.pengajuan_id ORDER BY a.created_at) - a.created_at
    ))) as avg_duration_seconds,
    COUNT(CASE WHEN EXTRACT(EPOCH FROM (
        LEAD(a.created_at) OVER (PARTITION BY a.pengajuan_id ORDER BY a.created_at) - a.created_at
    )) > 86400 THEN 1 END) as sla_breaches  -- > 24 hours
FROM perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas a
JOIN perlengkapan.ms_aktivitas_bmn ma ON a.aktivitas_id = ma.id
WHERE a.created_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY DATE_TRUNC('day', a.created_at), a.aktivitas_id, ma.nama;

-- Create unique index for concurrent refresh
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_workflow_metrics_unique
ON perlengkapan.mv_workflow_metrics(date, aktivitas_id);

-- ============================================
-- Satker Summary Materialized View
-- ============================================

CREATE MATERIALIZED VIEW IF NOT EXISTS perlengkapan.mv_satker_summary AS
SELECT
    s.id as satker_id,
    s.nama as satker_nama,
    s.tipe as satker_tipe,
    COUNT(k.id) as total_kebutuhan,
    SUM(k.jumlah_kebutuhan) as total_jumlah,
    COUNT(CASE WHEN k.status = 'APPROVED' THEN 1 END) as approved_count,
    MAX(k.updated_at) as last_activity
FROM authenc.satkers s
LEFT JOIN perlengkapan.kebutuhan_bmn k ON s.id = k.satker_id
WHERE k.tahun_anggaran = EXTRACT(YEAR FROM CURRENT_DATE)
GROUP BY s.id, s.nama, s.tipe;

-- Create unique index for concurrent refresh
CREATE UNIQUE INDEX IF NOT EXISTS idx_mv_satker_summary_satker
ON perlengkapan.mv_satker_summary(satker_id);

-- ============================================
-- Refresh Functions
-- ============================================

-- Function to refresh all materialized views
CREATE OR REPLACE FUNCTION perlengkapan.refresh_all_dashboard_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_dashboard_metrics;
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_gap_analysis;
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_workflow_metrics;
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_satker_summary;
END;
$$ LANGUAGE plpgsql;

-- Function to refresh specific view
CREATE OR REPLACE FUNCTION perlengkapan.refresh_dashboard_metrics()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_dashboard_metrics;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION perlengkapan.refresh_gap_analysis()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY perlengkapan.mv_gap_analysis;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- Scheduled Refresh (using pg_cron if available)
-- ============================================

-- Note: Requires pg_cron extension
-- CREATE EXTENSION IF NOT EXISTS pg_cron;

-- Refresh every 5 minutes
-- SELECT cron.schedule('refresh-dashboard-views', '*/5 * * * *',
--     'SELECT perlengkapan.refresh_all_dashboard_views()');

-- Alternative: Create a trigger to refresh on data changes
-- (Not recommended for high-frequency updates)

-- ============================================
-- Initial Refresh
-- ============================================

-- Refresh all views initially
SELECT perlengkapan.refresh_all_dashboard_views();

-- ============================================
-- Verification
-- ============================================

-- Check materialized view sizes
SELECT
    schemaname,
    matviewname,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||matviewname)) as size,
    last_refresh
FROM pg_matviews
WHERE schemaname = 'perlengkapan'
ORDER BY pg_total_relation_size(schemaname||'.'||matviewname) DESC;

-- Check row counts
SELECT 'mv_dashboard_metrics' as view_name, COUNT(*) as row_count
FROM perlengkapan.mv_dashboard_metrics
UNION ALL
SELECT 'mv_gap_analysis', COUNT(*)
FROM perlengkapan.mv_gap_analysis
UNION ALL
SELECT 'mv_workflow_metrics', COUNT(*)
FROM perlengkapan.mv_workflow_metrics
UNION ALL
SELECT 'mv_satker_summary', COUNT(*)
FROM perlengkapan.mv_satker_summary;
