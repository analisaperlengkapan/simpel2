-- ============================================================================
-- Database Validation Script for Phase 1: Database Refactoring & Standardization
-- Description: Validates all tables, indexes, and views created in Phase 1
-- Author: SIMPelv2 Team
-- Created: 2026-02-09
-- ============================================================================

\echo '========================================='
\echo 'Phase 1 Database Validation'
\echo '========================================='
\echo ''

-- ============================================================================
-- SECTION 1: Validate Integration Schema Tables
-- ============================================================================

\echo '1. Validating Integration Schema Tables...'
\echo ''

-- Check integrasi schema exists
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'integrasi')
    THEN '✅ integrasi schema exists'
    ELSE '❌ integrasi schema MISSING'
END as status;

-- Check SIMAN tables
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'siman_aset_tanah')
    THEN '✅ integrasi.siman_aset_tanah exists'
    ELSE '❌ integrasi.siman_aset_tanah MISSING'
END as status;

SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'siman_aset_gedung_bangunan')
    THEN '✅ integrasi.siman_aset_gedung_bangunan exists'
    ELSE '❌ integrasi.siman_aset_gedung_bangunan MISSING'
END as status;

SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'siman_aset_alat_besar')
    THEN '✅ integrasi.siman_aset_alat_besar exists'
    ELSE '❌ integrasi.siman_aset_alat_besar MISSING'
END as status;

SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'siman_aset_angkutan_bermotor')
    THEN '✅ integrasi.siman_aset_angkutan_bermotor exists'
    ELSE '❌ integrasi.siman_aset_angkutan_bermotor MISSING'
END as status;

-- Check MySIMKARI tables
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'mysimkari_pegawai')
    THEN '✅ integrasi.mysimkari_pegawai exists'
    ELSE '❌ integrasi.mysimkari_pegawai MISSING'
END as status;

-- Check API call log table
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'api_call_log')
    THEN '✅ integrasi.api_call_log exists'
    ELSE '❌ integrasi.api_call_log MISSING'
END as status;

-- Check sync status table
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'integrasi' AND table_name = 'sync_status')
    THEN '✅ integrasi.sync_status exists'
    ELSE '❌ integrasi.sync_status MISSING'
END as status;

\echo ''

-- ============================================================================
-- SECTION 2: Validate New Entity Tables
-- ============================================================================

\echo '2. Validating New Entity Tables...'
\echo ''

-- Check roadmap_sarpras
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'roadmap_sarpras')
    THEN '✅ perlengkapan.roadmap_sarpras exists'
    ELSE '❌ perlengkapan.roadmap_sarpras MISSING'
END as status;

-- Check mapping_kodefikasi
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'mapping_kodefikasi')
    THEN '✅ perlengkapan.mapping_kodefikasi exists'
    ELSE '❌ perlengkapan.mapping_kodefikasi MISSING'
END as status;

-- Check riwayat_pemenuhan
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'riwayat_pemenuhan')
    THEN '✅ perlengkapan.riwayat_pemenuhan exists'
    ELSE '❌ perlengkapan.riwayat_pemenuhan MISSING'
END as status;

-- Check parallel_approvals
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'parallel_approvals')
    THEN '✅ perlengkapan.parallel_approvals exists'
    ELSE '❌ perlengkapan.parallel_approvals MISSING'
END as status;

-- Check parallel_approval_votes
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'parallel_approval_votes')
    THEN '✅ perlengkapan.parallel_approval_votes exists'
    ELSE '❌ perlengkapan.parallel_approval_votes MISSING'
END as status;

-- Check izin_pemakaian_bmn
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'izin_pemakaian_bmn')
    THEN '✅ perlengkapan.izin_pemakaian_bmn exists'
    ELSE '❌ perlengkapan.izin_pemakaian_bmn MISSING'
END as status;

-- Check izin_pemakaian_bmn_aktivitas
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'perlengkapan' AND table_name = 'izin_pemakaian_bmn_aktivitas')
    THEN '✅ perlengkapan.izin_pemakaian_bmn_aktivitas exists'
    ELSE '❌ perlengkapan.izin_pemakaian_bmn_aktivitas MISSING'
END as status;

\echo ''

-- ============================================================================
-- SECTION 3: Validate Dashboard Views
-- ============================================================================

\echo '3. Validating Dashboard Views...'
\echo ''

-- Check v_gap_analysis
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.views WHERE table_schema = 'perlengkapan' AND table_name = 'v_gap_analysis')
    THEN '✅ perlengkapan.v_gap_analysis exists'
    ELSE '❌ perlengkapan.v_gap_analysis MISSING'
END as status;

-- Check mv_dashboard_metrics (materialized view)
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM pg_matviews WHERE schemaname = 'perlengkapan' AND matviewname = 'mv_dashboard_metrics')
    THEN '✅ perlengkapan.mv_dashboard_metrics exists'
    ELSE '❌ perlengkapan.mv_dashboard_metrics MISSING'
END as status;

-- Check v_kebutuhan_summary_by_satker
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.views WHERE table_schema = 'perlengkapan' AND table_name = 'v_kebutuhan_summary_by_satker')
    THEN '✅ perlengkapan.v_kebutuhan_summary_by_satker exists'
    ELSE '❌ perlengkapan.v_kebutuhan_summary_by_satker MISSING'
END as status;

-- Check v_workflow_performance
SELECT CASE
    WHEN EXISTS (SELECT 1 FROM information_schema.views WHERE table_schema = 'perlengkapan' AND table_name = 'v_workflow_performance')
    THEN '✅ perlengkapan.v_workflow_performance exists'
    ELSE '❌ perlengkapan.v_workflow_performance MISSING'
END as status;

\echo ''

-- ============================================================================
-- SECTION 4: Validate Performance Indexes
-- ============================================================================

\echo '4. Validating Performance Indexes...'
\echo ''

-- Count indexes in perlengkapan schema
SELECT
    '✅ Total indexes in perlengkapan schema: ' || COUNT(*) as status
FROM pg_indexes
WHERE schemaname = 'perlengkapan';

-- Count indexes in integrasi schema
SELECT
    '✅ Total indexes in integrasi schema: ' || COUNT(*) as status
FROM pg_indexes
WHERE schemaname = 'integrasi';

-- Check for GIN indexes (JSONB and full-text search)
SELECT
    '✅ GIN indexes (JSONB/full-text): ' || COUNT(*) as status
FROM pg_indexes
WHERE schemaname IN ('perlengkapan', 'integrasi')
AND indexdef LIKE '%USING gin%';

-- Check for composite indexes
SELECT
    '✅ Composite indexes: ' || COUNT(*) as status
FROM pg_indexes
WHERE schemaname IN ('perlengkapan', 'integrasi')
AND indexdef LIKE '%,%';

\echo ''

-- ============================================================================
-- SECTION 5: Validate Key Constraints
-- ============================================================================

\echo '5. Validating Key Constraints...'
\echo ''

-- Check foreign key constraints on new tables
SELECT
    '✅ Foreign key constraints on roadmap_sarpras: ' || COUNT(*) as status
FROM information_schema.table_constraints
WHERE table_schema = 'perlengkapan'
AND table_name = 'roadmap_sarpras'
AND constraint_type = 'FOREIGN KEY';

SELECT
    '✅ Foreign key constraints on riwayat_pemenuhan: ' || COUNT(*) as status
FROM information_schema.table_constraints
WHERE table_schema = 'perlengkapan'
AND table_name = 'riwayat_pemenuhan'
AND constraint_type = 'FOREIGN KEY';

SELECT
    '✅ Foreign key constraints on parallel_approval_votes: ' || COUNT(*) as status
FROM information_schema.table_constraints
WHERE table_schema = 'perlengkapan'
AND table_name = 'parallel_approval_votes'
AND constraint_type = 'FOREIGN KEY';

SELECT
    '✅ Foreign key constraints on izin_pemakaian_bmn: ' || COUNT(*) as status
FROM information_schema.table_constraints
WHERE table_schema = 'perlengkapan'
AND table_name = 'izin_pemakaian_bmn'
AND constraint_type = 'FOREIGN KEY';

-- Check unique constraints
SELECT
    '✅ Unique constraints on izin_pemakaian_bmn: ' || COUNT(*) as status
FROM information_schema.table_constraints
WHERE table_schema = 'perlengkapan'
AND table_name = 'izin_pemakaian_bmn'
AND constraint_type = 'UNIQUE';

\echo ''

-- ============================================================================
-- SECTION 6: Validate Triggers
-- ============================================================================

\echo '6. Validating Triggers...'
\echo ''

-- Check updated_at triggers
SELECT
    '✅ Triggers on new entity tables: ' || COUNT(*) as status
FROM information_schema.triggers
WHERE event_object_schema = 'perlengkapan'
AND event_object_table IN (
    'roadmap_sarpras',
    'mapping_kodefikasi',
    'riwayat_pemenuhan',
    'parallel_approvals',
    'izin_pemakaian_bmn'
);

\echo ''

-- ============================================================================
-- SECTION 7: Validate Functions
-- ============================================================================

\echo '7. Validating Functions...'
\echo ''

-- Check refresh_dashboard_metrics function
SELECT CASE
    WHEN EXISTS (
        SELECT 1 FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        WHERE n.nspname = 'perlengkapan'
        AND p.proname = 'refresh_dashboard_metrics'
    )
    THEN '✅ perlengkapan.refresh_dashboard_metrics() exists'
    ELSE '❌ perlengkapan.refresh_dashboard_metrics() MISSING'
END as status;

-- Check update_entity_updated_at function
SELECT CASE
    WHEN EXISTS (
        SELECT 1 FROM pg_proc p
        JOIN pg_namespace n ON p.pronamespace = n.oid
        WHERE n.nspname = 'perlengkapan'
        AND p.proname = 'update_entity_updated_at'
    )
    THEN '✅ perlengkapan.update_entity_updated_at() exists'
    ELSE '❌ perlengkapan.update_entity_updated_at() MISSING'
END as status;

\echo ''

-- ============================================================================
-- SECTION 8: Summary Statistics
-- ============================================================================

\echo '8. Summary Statistics...'
\echo ''

-- Total tables in perlengkapan schema
SELECT
    '📊 Total tables in perlengkapan schema: ' || COUNT(*) as summary
FROM information_schema.tables
WHERE table_schema = 'perlengkapan'
AND table_type = 'BASE TABLE';

-- Total tables in integrasi schema
SELECT
    '📊 Total tables in integrasi schema: ' || COUNT(*) as summary
FROM information_schema.tables
WHERE table_schema = 'integrasi'
AND table_type = 'BASE TABLE';

-- Total views in perlengkapan schema
SELECT
    '📊 Total views in perlengkapan schema: ' || COUNT(*) as summary
FROM information_schema.views
WHERE table_schema = 'perlengkapan';

-- Total materialized views in perlengkapan schema
SELECT
    '📊 Total materialized views in perlengkapan schema: ' || COUNT(*) as summary
FROM pg_matviews
WHERE schemaname = 'perlengkapan';

\echo ''
\echo '========================================='
\echo 'Phase 1 Validation Complete!'
\echo '========================================='
\echo ''
\echo 'If all checks show ✅, Phase 1 is successfully completed.'
\echo 'If any checks show ❌, review the migration files and re-run them.'
\echo ''
