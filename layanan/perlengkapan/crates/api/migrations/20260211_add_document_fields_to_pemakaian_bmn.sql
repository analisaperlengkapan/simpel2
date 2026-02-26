-- ============================================================================
-- Migration: Add document fields to izin_pemakaian_bmn
-- Description: Add document_id and document_url for storing permit document references
-- Requirements: REQ-P006, REQ-D002
-- ============================================================================

-- Add document fields to izin_pemakaian_bmn table
ALTER TABLE perlengkapan.izin_pemakaian_bmn
ADD COLUMN IF NOT EXISTS document_id UUID,
ADD COLUMN IF NOT EXISTS document_url TEXT;

-- Add index for document_id
CREATE INDEX IF NOT EXISTS idx_izin_document_id ON perlengkapan.izin_pemakaian_bmn(document_id);

-- Add comments
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.document_id IS 'Reference to generated permit document in dokumen service';
COMMENT ON COLUMN perlengkapan.izin_pemakaian_bmn.document_url IS 'Download URL for the permit document';
