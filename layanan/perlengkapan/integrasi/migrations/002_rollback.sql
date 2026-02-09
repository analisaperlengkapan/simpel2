-- ============================================================================
-- ROLLBACK SCRIPT - SIMPelv2 Layanan Integrasi
-- ============================================================================
-- Purpose: Clean rollback of all integration service database objects
-- Usage: psql -U username -d database_name -f migrations/002_rollback.sql
-- ============================================================================

SET search_path TO integrasi, public;

-- Drop views first (dependent on tables)
DROP VIEW IF EXISTS v_api_stats_by_module CASCADE;
DROP VIEW IF EXISTS v_token_health CASCADE;
DROP VIEW IF EXISTS v_recent_failed_calls CASCADE;

-- Drop triggers
DROP TRIGGER IF EXISTS update_api_tokens_updated_at ON api_tokens;
DROP TRIGGER IF EXISTS update_siman_angkutan_updated_at ON siman_aset_angkutan_bermotor;
DROP TRIGGER IF EXISTS update_siman_alat_besar_updated_at ON siman_aset_alat_besar;
DROP TRIGGER IF EXISTS update_siman_gedung_updated_at ON siman_aset_gedung_bangunan;
DROP TRIGGER IF EXISTS update_siman_tanah_updated_at ON siman_aset_tanah;
DROP TRIGGER IF EXISTS update_mysimkari_pegawai_updated_at ON mysimkari_pegawai;
DROP TRIGGER IF EXISTS update_mysimkari_satker_updated_at ON mysimkari_satker;
DROP TRIGGER IF EXISTS update_adm_ref_jns_spp_updated_at ON adm_ref_jns_spp;
DROP TRIGGER IF EXISTS update_adm_ref_bank_updated_at ON adm_ref_bank;
DROP TRIGGER IF EXISTS update_adm_ref_admin_updated_at ON adm_ref_admin;

-- Drop function
DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- Drop SIMAN tables
DROP TABLE IF EXISTS siman_aset_angkutan_bermotor CASCADE;
DROP TABLE IF EXISTS siman_aset_alat_besar CASCADE;
DROP TABLE IF EXISTS siman_aset_gedung_bangunan CASCADE;
DROP TABLE IF EXISTS siman_aset_tanah CASCADE;

-- Drop MySIMKARI tables
DROP TABLE IF EXISTS mysimkari_pegawai CASCADE;
DROP TABLE IF EXISTS mysimkari_satker CASCADE;

-- Drop MonSAKTI tables
DROP TABLE IF EXISTS adm_ref_jns_spp CASCADE;
DROP TABLE IF EXISTS adm_ref_bank CASCADE;
DROP TABLE IF EXISTS adm_ref_admin CASCADE;

-- Drop audit/logging tables
DROP TABLE IF EXISTS api_tokens CASCADE;
DROP TABLE IF EXISTS token_reset_log CASCADE;
DROP TABLE IF EXISTS data_sync_log CASCADE;
DROP TABLE IF EXISTS batch_processing_log CASCADE;
DROP TABLE IF EXISTS api_call_log CASCADE;

-- Drop schema (optional - uncomment if you want to completely remove the schema)
-- DROP SCHEMA IF EXISTS integrasi CASCADE;

DO $$
BEGIN
    RAISE NOTICE '✅ Integration service schema rolled back successfully';
    RAISE NOTICE '⚠️  All tables, views, triggers, and functions have been dropped';
END $$;
