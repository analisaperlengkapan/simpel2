-- ============================================================================
-- Migration: Create Penghapusan BMN Tables
-- Description: Create tables for BMN disposal workflow
-- Requirements: REQ-W001, REQ-W004, REQ-D002
-- ============================================================================

-- Create penghapusan_bmn table
CREATE TABLE IF NOT EXISTS perlengkapan.penghapusan_bmn (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    satker_id UUID NOT NULL,
    asset_id UUID NOT NULL,
    kode_barang VARCHAR(50) NOT NULL,
    nama_barang VARCHAR(255) NOT NULL,
    nup VARCHAR(50) NOT NULL,
    tanggal_penghapusan DATE NOT NULL,
    alasan TEXT NOT NULL,
    metode_penghapusan VARCHAR(100) NOT NULL, -- DIJUAL, DIHIBAHKAN, DIMUSNAHKAN
    nilai_residu DECIMAL(15,2),
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT', -- DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED, CANCELLED
    document_id UUID,
    document_url TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for penghapusan_bmn
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_satker ON perlengkapan.penghapusan_bmn(satker_id);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_asset ON perlengkapan.penghapusan_bmn(asset_id);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_status ON perlengkapan.penghapusan_bmn(status);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_created_at ON perlengkapan.penghapusan_bmn(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_tahun ON perlengkapan.penghapusan_bmn(EXTRACT(YEAR FROM tanggal_penghapusan));

-- Create penghapusan_bmn_aktivitas table for workflow tracking
CREATE TABLE IF NOT EXISTS perlengkapan.penghapusan_bmn_aktivitas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    penghapusan_id UUID NOT NULL REFERENCES perlengkapan.penghapusan_bmn(id) ON DELETE CASCADE,
    aktivitas_id INTEGER NOT NULL REFERENCES perlengkapan.ms_aktivitas_bmn(id),
    user_id UUID NOT NULL,
    catatan TEXT,
    document_id UUID,
    document_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for penghapusan_bmn_aktivitas
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_aktivitas_penghapusan ON perlengkapan.penghapusan_bmn_aktivitas(penghapusan_id);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_aktivitas_aktivitas ON perlengkapan.penghapusan_bmn_aktivitas(aktivitas_id);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_aktivitas_user ON perlengkapan.penghapusan_bmn_aktivitas(user_id);
CREATE INDEX IF NOT EXISTS idx_penghapusan_bmn_aktivitas_created_at ON perlengkapan.penghapusan_bmn_aktivitas(created_at DESC);

-- Add comments
COMMENT ON TABLE perlengkapan.penghapusan_bmn IS 'BMN disposal records with workflow support';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.metode_penghapusan IS 'Disposal method: DIJUAL (sold), DIHIBAHKAN (donated), DIMUSNAHKAN (destroyed)';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.status IS 'Workflow status: DRAFT, SUBMITTED, REVIEWED, APPROVED, REJECTED, CANCELLED';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.document_id IS 'Generated SK Penghapusan document ID';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn.document_url IS 'URL to download SK Penghapusan document';

COMMENT ON TABLE perlengkapan.penghapusan_bmn_aktivitas IS 'Workflow activity log for penghapusan BMN';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn_aktivitas.document_id IS 'Document generated during this activity (e.g., SK Penghapusan)';
COMMENT ON COLUMN perlengkapan.penghapusan_bmn_aktivitas.document_url IS 'URL to download document generated during this activity';
