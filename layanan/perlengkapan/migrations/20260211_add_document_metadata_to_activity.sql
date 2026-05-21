-- Migration: Add document metadata columns to workflow activity table
-- Description: Add document_id and document_url columns to track generated documents
-- Requirements: REQ-D005, REQ-W011
-- Date: 2026-02-11

-- Add document metadata columns to pengajuan_kebutuhan_bmn_satker_aktivitas
ALTER TABLE perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas
ADD COLUMN IF NOT EXISTS document_id UUID,
ADD COLUMN IF NOT EXISTS document_url TEXT;

-- Add index for document_id lookups
CREATE INDEX IF NOT EXISTS idx_aktivitas_document_id
ON perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas(document_id)
WHERE document_id IS NOT NULL;

-- Add comment
COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas.document_id IS 'ID of generated document (SK, surat izin, etc.) from dokumen service';
COMMENT ON COLUMN perlengkapan.pengajuan_kebutuhan_bmn_satker_aktivitas.document_url IS 'Download URL for the generated document';
