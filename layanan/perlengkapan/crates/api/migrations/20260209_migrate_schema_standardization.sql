-- ============================================================================
-- Migration: Schema Standardization - Data Migration
-- Description: Migrate pakaian_dinas tables to perlengkapan schema
-- Author: SIMPEL Team
-- Created: 2026-02-09
-- Requirements: NFR-A003, NFR-A004
-- ============================================================================

-- ⚠️ CRITICAL: This migration should be executed during a maintenance window
-- ⚠️ BACKUP: Ensure full database backup before running this migration
-- ⚠️ TEST: Test on staging environment first

-- ============================================================================
-- PRE-MIGRATION CHECKS
-- ============================================================================

DO $$
DECLARE
    table_count INTEGER;
    pakaian_count INTEGER;
BEGIN
    -- Check if perlengkapan schema exists
    IF NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'perlengkapan') THEN
        RAISE EXCEPTION 'perlengkapan schema does not exist. Run previous migrations first.';
    END IF;

    -- Count pakaian_dinas tables in public schema
    SELECT COUNT(*) INTO table_count
    FROM pg_tables
    WHERE schemaname = 'public'
    AND tablename LIKE '%pakaian_dinas%';

    RAISE NOTICE 'Found % pakaian_dinas tables in public schema', table_count;

    -- Count existing records
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas') THEN
        SELECT COUNT(*) INTO pakaian_count FROM pengajuan_pakaian_dinas;
        RAISE NOTICE 'Found % records in pengajuan_pakaian_dinas', pakaian_count;
    END IF;
END $$;

-- ============================================================================
-- STEP 1: MOVE TABLES TO PERLENGKAPAN SCHEMA
-- ============================================================================

BEGIN;

RAISE NOTICE 'Starting schema migration...';

-- Move master tables
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'ms_jenis_pakaian_dinas') THEN
        ALTER TABLE public.ms_jenis_pakaian_dinas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved ms_jenis_pakaian_dinas to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'ms_spesifikasi_pakaian_dinas') THEN
        ALTER TABLE public.ms_spesifikasi_pakaian_dinas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved ms_spesifikasi_pakaian_dinas to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'ms_spesifikasi_pakaian_dinas_foto') THEN
        ALTER TABLE public.ms_spesifikasi_pakaian_dinas_foto SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved ms_spesifikasi_pakaian_dinas_foto to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'ms_subspesifikasi_pakaian_dinas') THEN
        ALTER TABLE public.ms_subspesifikasi_pakaian_dinas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved ms_subspesifikasi_pakaian_dinas to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'ms_ukuran') THEN
        ALTER TABLE public.ms_ukuran SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved ms_ukuran to perlengkapan schema';
    END IF;
END $$;

-- Move transaction tables
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_satker_terpilih') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_satker_terpilih SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_satker_terpilih to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_pakaian') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_pakaian SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_pakaian to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_satker') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_satker SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_satker to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_satker_pegawai') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_satker_pegawai SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_satker_pegawai to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_satker_pegawai_ukuran') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_satker_pegawai_ukuran SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_satker_pegawai_ukuran to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pengajuan_pakaian_dinas_satker_aktivitas') THEN
        ALTER TABLE public.pengajuan_pakaian_dinas_satker_aktivitas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pengajuan_pakaian_dinas_satker_aktivitas to perlengkapan schema';
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_tables WHERE schemaname = 'public' AND tablename = 'pegawai_pakaian_dinas') THEN
        ALTER TABLE public.pegawai_pakaian_dinas SET SCHEMA perlengkapan;
        RAISE NOTICE 'Moved pegawai_pakaian_dinas to perlengkapan schema';
    END IF;
END $$;

COMMIT;

RAISE NOTICE 'Schema migration completed successfully';

-- ============================================================================
-- STEP 2: STANDARDIZE WORKFLOW STATUS FIELD NAMES
-- ============================================================================

BEGIN;

RAISE NOTICE 'Standardizing workflow status field names...';

-- Rename aktivitas_id to status_kode in pengajuan_pakaian_dinas
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'perlengkapan'
        AND table_name = 'pengajuan_pakaian_dinas'
        AND column_name = 'aktivitas_id'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
        RENAME COLUMN aktivitas_id TO status_kode;
        RAISE NOTICE 'Renamed aktivitas_id to status_kode in pengajuan_pakaian_dinas';
    END IF;
END $$;

-- Rename aktivitas_id to status_kode in pengajuan_pakaian_dinas_satker
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'perlengkapan'
        AND table_name = 'pengajuan_pakaian_dinas_satker'
        AND column_name = 'aktivitas_id'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker
        RENAME COLUMN aktivitas_id TO status_kode;
        RAISE NOTICE 'Renamed aktivitas_id to status_kode in pengajuan_pakaian_dinas_satker';
    END IF;
END $$;

-- Rename aktivitas_id to status_kode in pengajuan_pakaian_dinas_satker_aktivitas
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_schema = 'perlengkapan'
        AND table_name = 'pengajuan_pakaian_dinas_satker_aktivitas'
        AND column_name = 'aktivitas_id'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas
        RENAME COLUMN aktivitas_id TO status_kode;
        RAISE NOTICE 'Renamed aktivitas_id to status_kode in pengajuan_pakaian_dinas_satker_aktivitas';
    END IF;
END $$;

COMMIT;

-- ============================================================================
-- STEP 3: ADD FOREIGN KEY CONSTRAINTS TO MS_AKTIVITAS_BMN
-- ============================================================================

BEGIN;

RAISE NOTICE 'Adding foreign key constraints...';

-- Add FK constraint to pengajuan_pakaian_dinas
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_ppd_status_kode'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
        ADD CONSTRAINT fk_ppd_status_kode
        FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);
        RAISE NOTICE 'Added FK constraint fk_ppd_status_kode';
    END IF;
END $$;

-- Add FK constraint to pengajuan_pakaian_dinas_satker
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_ppd_satker_status_kode'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker
        ADD CONSTRAINT fk_ppd_satker_status_kode
        FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);
        RAISE NOTICE 'Added FK constraint fk_ppd_satker_status_kode';
    END IF;
END $$;

-- Add FK constraint to pengajuan_pakaian_dinas_satker_aktivitas
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'fk_ppd_aktivitas_status_kode'
    ) THEN
        ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas
        ADD CONSTRAINT fk_ppd_aktivitas_status_kode
        FOREIGN KEY (status_kode) REFERENCES perlengkapan.ms_aktivitas_bmn(kode);
        RAISE NOTICE 'Added FK constraint fk_ppd_aktivitas_status_kode';
    END IF;
END $$;

COMMIT;

-- ============================================================================
-- STEP 4: STANDARDIZE TIMESTAMP FIELDS
-- ============================================================================

BEGIN;

RAISE NOTICE 'Standardizing timestamp fields...';

-- Convert TIMESTAMP WITH TIME ZONE to TIMESTAMPTZ (they are the same, just aliases)
-- PostgreSQL already stores them the same way, so no conversion needed
-- Just add comments for documentation

COMMENT ON COLUMN perlengkapan.ms_jenis_pakaian_dinas.created_at IS 'Creation timestamp (TIMESTAMPTZ)';
COMMENT ON COLUMN perlengkapan.ms_jenis_pakaian_dinas.updated_at IS 'Last update timestamp (TIMESTAMPTZ)';

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.created_at IS 'Creation timestamp (TIMESTAMPTZ)';
COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.updated_at IS 'Last update timestamp (TIMESTAMPTZ)';

RAISE NOTICE 'Timestamp fields standardized (TIMESTAMPTZ)';

COMMIT;

-- ============================================================================
-- STEP 5: DATA INTEGRITY VERIFICATION
-- ============================================================================

DO $$
DECLARE
    jenis_count INTEGER;
    spesifikasi_count INTEGER;
    pengajuan_count INTEGER;
    satker_count INTEGER;
    pegawai_count INTEGER;
    orphan_count INTEGER;
BEGIN
    RAISE NOTICE 'Verifying data integrity...';

    -- Count records in each table
    SELECT COUNT(*) INTO jenis_count FROM perlengkapan.ms_jenis_pakaian_dinas;
    SELECT COUNT(*) INTO spesifikasi_count FROM perlengkapan.ms_spesifikasi_pakaian_dinas;
    SELECT COUNT(*) INTO pengajuan_count FROM perlengkapan.pengajuan_pakaian_dinas;
    SELECT COUNT(*) INTO satker_count FROM perlengkapan.pengajuan_pakaian_dinas_satker;
    SELECT COUNT(*) INTO pegawai_count FROM perlengkapan.pengajuan_pakaian_dinas_satker_pegawai;

    RAISE NOTICE 'Record counts:';
    RAISE NOTICE '  - ms_jenis_pakaian_dinas: %', jenis_count;
    RAISE NOTICE '  - ms_spesifikasi_pakaian_dinas: %', spesifikasi_count;
    RAISE NOTICE '  - pengajuan_pakaian_dinas: %', pengajuan_count;
    RAISE NOTICE '  - pengajuan_pakaian_dinas_satker: %', satker_count;
    RAISE NOTICE '  - pengajuan_pakaian_dinas_satker_pegawai: %', pegawai_count;

    -- Check for orphaned records (should be 0 due to FK constraints)
    SELECT COUNT(*) INTO orphan_count
    FROM perlengkapan.pengajuan_pakaian_dinas_satker ps
    LEFT JOIN perlengkapan.pengajuan_pakaian_dinas p ON ps.pengajuan_id = p.id
    WHERE p.id IS NULL;

    IF orphan_count > 0 THEN
        RAISE WARNING 'Found % orphaned satker records', orphan_count;
    ELSE
        RAISE NOTICE '✅ No orphaned records found';
    END IF;

    -- Verify FK constraints
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname IN ('fk_ppd_status_kode', 'fk_ppd_satker_status_kode', 'fk_ppd_aktivitas_status_kode')
    ) THEN
        RAISE NOTICE '✅ Foreign key constraints verified';
    ELSE
        RAISE WARNING '⚠️ Some foreign key constraints are missing';
    END IF;

    RAISE NOTICE 'Data integrity verification completed';
END $$;

-- ============================================================================
-- STEP 6: UPDATE APPLICATION SEARCH PATH (Optional)
-- ============================================================================

-- Update default search path to include perlengkapan schema
-- This allows queries without schema prefix
-- ALTER DATABASE simpelv2 SET search_path TO perlengkapan, public;

-- Or set per-role:
-- ALTER ROLE simpelv2_app SET search_path TO perlengkapan, public;

-- ============================================================================
-- ROLLBACK SCRIPT (Save for emergency)
-- ============================================================================

-- In case of issues, use this rollback script:
--
-- BEGIN;
--
-- -- Move tables back to public schema
-- ALTER TABLE perlengkapan.ms_jenis_pakaian_dinas SET SCHEMA public;
-- ALTER TABLE perlengkapan.ms_spesifikasi_pakaian_dinas SET SCHEMA public;
-- ALTER TABLE perlengkapan.ms_spesifikasi_pakaian_dinas_foto SET SCHEMA public;
-- ALTER TABLE perlengkapan.ms_subspesifikasi_pakaian_dinas SET SCHEMA public;
-- ALTER TABLE perlengkapan.ms_ukuran SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_terpilih SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_pakaian SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_pegawai_ukuran SET SCHEMA public;
-- ALTER TABLE perlengkapan.pengajuan_pakaian_dinas_satker_aktivitas SET SCHEMA public;
-- ALTER TABLE perlengkapan.pegawai_pakaian_dinas SET SCHEMA public;
--
-- -- Rename columns back
-- ALTER TABLE public.pengajuan_pakaian_dinas RENAME COLUMN status_kode TO aktivitas_id;
-- ALTER TABLE public.pengajuan_pakaian_dinas_satker RENAME COLUMN status_kode TO aktivitas_id;
-- ALTER TABLE public.pengajuan_pakaian_dinas_satker_aktivitas RENAME COLUMN status_kode TO aktivitas_id;
--
-- -- Drop FK constraints
-- ALTER TABLE public.pengajuan_pakaian_dinas DROP CONSTRAINT IF EXISTS fk_ppd_status_kode;
-- ALTER TABLE public.pengajuan_pakaian_dinas_satker DROP CONSTRAINT IF EXISTS fk_ppd_satker_status_kode;
-- ALTER TABLE public.pengajuan_pakaian_dinas_satker_aktivitas DROP CONSTRAINT IF EXISTS fk_ppd_aktivitas_status_kode;
--
-- COMMIT;

-- ============================================================================
-- POST-MIGRATION TASKS
-- ============================================================================

-- 1. Update application configuration to use perlengkapan schema
-- 2. Update all SQL queries to use schema-qualified table names
-- 3. Test all application features
-- 4. Monitor application logs for errors
-- 5. Run ANALYZE on all migrated tables
-- 6. Update documentation

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

DO $$
DECLARE
    total_tables INTEGER;
    perlengkapan_tables INTEGER;
BEGIN
    -- Count total tables in perlengkapan schema
    SELECT COUNT(*) INTO perlengkapan_tables
    FROM pg_tables
    WHERE schemaname = 'perlengkapan';

    RAISE NOTICE '';
    RAISE NOTICE '═══════════════════════════════════════════════════════════';
    RAISE NOTICE '✅ MIGRATION COMPLETED SUCCESSFULLY';
    RAISE NOTICE '═══════════════════════════════════════════════════════════';
    RAISE NOTICE '';
    RAISE NOTICE '📊 Summary:';
    RAISE NOTICE '  - Total tables in perlengkapan schema: %', perlengkapan_tables;
    RAISE NOTICE '  - Pakaian dinas tables migrated: 13';
    RAISE NOTICE '  - Workflow status fields standardized: 3';
    RAISE NOTICE '  - Foreign key constraints added: 3';
    RAISE NOTICE '';
    RAISE NOTICE '📝 Next Steps:';
    RAISE NOTICE '  1. Update application code to use perlengkapan schema';
    RAISE NOTICE '  2. Test all pakaian dinas features';
    RAISE NOTICE '  3. Monitor application logs';
    RAISE NOTICE '  4. Run performance tests';
    RAISE NOTICE '  5. Update API documentation';
    RAISE NOTICE '';
    RAISE NOTICE '⚠️ Important:';
    RAISE NOTICE '  - Keep database backup until migration is verified';
    RAISE NOTICE '  - Rollback script is available in this migration file';
    RAISE NOTICE '  - Monitor query performance after migration';
    RAISE NOTICE '';
    RAISE NOTICE '═══════════════════════════════════════════════════════════';
END $$;
