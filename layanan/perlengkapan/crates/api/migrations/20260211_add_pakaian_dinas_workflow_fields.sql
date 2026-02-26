-- Migration: Add workflow fields to pakaian dinas tables
-- Description: Add document fields and activity tracking for pakaian dinas workflow
-- Requirements: REQ-D011, REQ-N001, REQ-W001

-- Add document fields to pengajuan_pakaian_dinas table
ALTER TABLE perlengkapan.pengajuan_pakaian_dinas
ADD COLUMN IF NOT EXISTS document_id UUID,
ADD COLUMN IF NOT EXISTS document_url TEXT;

-- Create activity log table for pakaian dinas workflow
CREATE TABLE IF NOT EXISTS perlengkapan.pengajuan_pakaian_dinas_aktivitas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pengajuan_id UUID NOT NULL REFERENCES perlengkapan.pengajuan_pakaian_dinas(id) ON DELETE CASCADE,
    aktivitas_id INTEGER NOT NULL REFERENCES perlengkapan.ms_aktivitas_bmn(id),
    user_id UUID NOT NULL,
    catatan TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for activity log
CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_aktivitas_pengajuan
ON perlengkapan.pengajuan_pakaian_dinas_aktivitas(pengajuan_id);

CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_aktivitas_user
ON perlengkapan.pengajuan_pakaian_dinas_aktivitas(user_id);

CREATE INDEX IF NOT EXISTS idx_pakaian_dinas_aktivitas_created
ON perlengkapan.pengajuan_pakaian_dinas_aktivitas(created_at);

-- Add comment
COMMENT ON TABLE perlengkapan.pengajuan_pakaian_dinas_aktivitas IS
'Activity log for pakaian dinas workflow transitions';

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.document_id IS
'Reference to generated rekapitulasi document';

COMMENT ON COLUMN perlengkapan.pengajuan_pakaian_dinas.document_url IS
'URL to download rekapitulasi document';
