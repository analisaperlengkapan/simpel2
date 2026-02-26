-- Migration: Create export_jobs table for async export functionality
-- Date: 2026-02-10
-- Purpose: Support asynchronous export of large datasets

-- Create export_jobs table
CREATE TABLE IF NOT EXISTS perlengkapan.export_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(50) NOT NULL,
    filters JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'queued',
    progress REAL,
    document_id UUID,
    error_message TEXT,
    created_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    CONSTRAINT chk_status CHECK (status IN ('queued', 'processing', 'completed', 'failed')),
    CONSTRAINT chk_progress CHECK (progress IS NULL OR (progress >= 0 AND progress <= 100))
);

-- Create indexes
CREATE INDEX idx_export_jobs_status ON perlengkapan.export_jobs(status);
CREATE INDEX idx_export_jobs_created_by ON perlengkapan.export_jobs(created_by);
CREATE INDEX idx_export_jobs_created_at ON perlengkapan.export_jobs(created_at DESC);

-- Add comments
COMMENT ON TABLE perlengkapan.export_jobs IS 'Async export job queue for large datasets';
COMMENT ON COLUMN perlengkapan.export_jobs.entity_type IS 'Type of entity being exported (kebutuhan_bmn, pakaian_dinas, etc.)';
COMMENT ON COLUMN perlengkapan.export_jobs.filters IS 'JSON-encoded filters applied to the export';
COMMENT ON COLUMN perlengkapan.export_jobs.status IS 'Job status: queued, processing, completed, failed';
COMMENT ON COLUMN perlengkapan.export_jobs.progress IS 'Export progress percentage (0-100)';
COMMENT ON COLUMN perlengkapan.export_jobs.document_id IS 'Reference to generated document in dokumen service';
COMMENT ON COLUMN perlengkapan.export_jobs.error_message IS 'Error message if job failed';
